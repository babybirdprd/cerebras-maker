// Cerebras-MAKER: Rhai Code Mode Runtime
// PRD Section 4: The Logic Layer - Sandboxed Scripting Runtime
// Enhanced with RLM (Recursive Language Model) capabilities

use super::atom::{AtomResult, AtomType, SpawnFlags};
use super::rlm::{RLMConfig, RLMOperation, RLMTrajectoryStep, ContextType, SharedRLMContextStore};
use super::shadow_git::ShadowGit;
use super::voting::{ConsensusConfig, ConsensusResult, run_consensus as voting_run_consensus};
use crate::agents::AtomInput;
use crate::grits;
use crate::llm::LlmConfig;
use rhai::{Dynamic, Engine, EvalAltResult, Scope, AST};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// The Code Mode Runtime - executes Rhai scripts with MAKER API
/// Enhanced with RLM (Recursive Language Model) capabilities for handling arbitrarily long contexts
pub struct CodeModeRuntime {
    engine: Engine,
    #[allow(dead_code)] // Stored for potential future runtime reconfigurations
    shadow_git: Arc<Mutex<ShadowGit>>,
    workspace_path: String,
    execution_log: Arc<Mutex<Vec<ExecutionEvent>>>,
    #[allow(dead_code)] // Stored for potential future runtime reconfigurations
    llm_config: Arc<LlmConfig>,
    /// RLM Context Store - holds large contexts as environment variables
    rlm_context_store: SharedRLMContextStore,
    /// RLM Configuration
    rlm_config: RLMConfig,
    /// RLM Trajectory for visualization
    rlm_trajectory: Arc<Mutex<Vec<RLMTrajectoryStep>>>,
}

/// Events emitted during execution for the Cockpit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEvent {
    pub timestamp_ms: u64,
    pub event_type: ExecutionEventType,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionEventType {
    ScriptStart,
    ScriptEnd,
    AtomSpawned,
    AtomCompleted,
    ConsensusStart,
    ConsensusVote,
    ConsensusEnd,
    RedFlagDetected,
    Snapshot,
    Rollback,
    Error,
    // RLM-specific events
    RLMStart,
    RLMPeek,
    RLMChunk,
    RLMSubQuery,
    RLMSubResult,
    RLMRegexFilter,
    RLMLoadContext,
    RLMFinal,
}

impl CodeModeRuntime {
    /// Create a new Code Mode Runtime
    pub fn new(workspace_path: &str) -> Result<Self, Box<EvalAltResult>> {
        Self::with_config(workspace_path, LlmConfig::cerebras())
    }

    /// Create a new Code Mode Runtime with custom LLM config
    pub fn with_config(workspace_path: &str, llm_config: LlmConfig) -> Result<Self, Box<EvalAltResult>> {
        Self::with_full_config(workspace_path, llm_config, RLMConfig::default())
    }

    /// Create a new Code Mode Runtime with custom LLM and RLM config
    pub fn with_full_config(workspace_path: &str, llm_config: LlmConfig, rlm_config: RLMConfig) -> Result<Self, Box<EvalAltResult>> {
        let mut engine = Engine::new();
        let shadow_git = Arc::new(Mutex::new(ShadowGit::new(workspace_path)));
        let execution_log = Arc::new(Mutex::new(Vec::new()));
        let llm_config = Arc::new(llm_config);
        let rlm_context_store = super::rlm::create_shared_store();
        let rlm_trajectory = Arc::new(Mutex::new(Vec::new()));

        // Initialize the atom worker pool for safe async-to-sync bridging
        // This creates a dedicated runtime thread that handles all atom executions
        super::atom_bridge::init_atom_pool(llm_config.clone());

        // Register MAKER API functions (including RLM functions)
        Self::register_api(
            &mut engine,
            shadow_git.clone(),
            execution_log.clone(),
            workspace_path.to_string(),
            llm_config.clone(),
            rlm_context_store.clone(),
            rlm_trajectory.clone(),
            rlm_config.clone(),
        );

        Ok(Self {
            engine,
            shadow_git,
            workspace_path: workspace_path.to_string(),
            execution_log,
            llm_config,
            rlm_context_store,
            rlm_config,
            rlm_trajectory,
        })
    }

    /// Register the MAKER API into the Rhai engine
    fn register_api(
        engine: &mut Engine,
        shadow_git: Arc<Mutex<ShadowGit>>,
        log: Arc<Mutex<Vec<ExecutionEvent>>>,
        workspace_path: String,
        llm_config: Arc<LlmConfig>,
        rlm_store: SharedRLMContextStore,
        rlm_trajectory: Arc<Mutex<Vec<RLMTrajectoryStep>>>,
        rlm_config: RLMConfig,
    ) {
        // Register AtomType enum
        engine.register_type_with_name::<AtomType>("AtomType");
        engine.register_static_module("AtomType", Self::create_atom_type_module().into());

        // Register spawn_atom function - bridges to async AtomExecutor
        let config_spawn = llm_config.clone();
        let ws_spawn = workspace_path.clone();
        let log_spawn = log.clone();
        engine.register_fn("spawn_atom", move |atom_type: AtomType, prompt: &str| -> Dynamic {
            Self::execute_spawn_atom(
                atom_type.clone(),
                prompt,
                SpawnFlags::default(),
                &config_spawn,
                &ws_spawn,
                &log_spawn,
            )
        });

        // Register spawn_atom_with_flags
        let config_flags = llm_config.clone();
        let ws_flags = workspace_path.clone();
        let log_flags = log.clone();
        engine.register_fn("spawn_atom_with_flags",
            move |atom_type: AtomType, prompt: &str, flags: Dynamic| -> Dynamic {
                let spawn_flags: SpawnFlags = rhai::serde::from_dynamic(&flags).unwrap_or_default();
                Self::execute_spawn_atom(
                    atom_type.clone(),
                    prompt,
                    spawn_flags,
                    &config_flags,
                    &ws_flags,
                    &log_flags,
                )
            }
        );

        // Register run_consensus - bridges to async consensus voting
        let config_consensus = llm_config.clone();
        let ws_consensus = workspace_path.clone();
        let log_consensus = log.clone();
        engine.register_fn("run_consensus",
            move |atom_type: AtomType, task: &str, k_threshold: i64| -> Dynamic {
                Self::execute_consensus(
                    atom_type.clone(),
                    task,
                    k_threshold as usize,
                    &config_consensus,
                    &ws_consensus,
                    &log_consensus,
                )
            }
        );

        // Register check_red_flags
        engine.register_fn("check_red_flags", move |_code: &str| -> bool {
            // Uses grits-core red flag checking
            if let Some(graph) = grits::get_cached_graph() {
                let workspace_path = grits::get_cached_workspace_path();
                let result = grits::red_flag_check(&graph, 0, workspace_path.as_deref());
                result.introduced_cycle || result.has_layer_violations
            } else {
                false
            }
        });

        // Register snapshot
        let sg_snapshot = shadow_git.clone();
        engine.register_fn("snapshot", move |message: &str| -> bool {
            if let Ok(mut sg) = sg_snapshot.lock() {
                sg.snapshot(message).is_ok()
            } else {
                false
            }
        });

        // Register rollback
        let sg_rollback = shadow_git.clone();
        engine.register_fn("rollback", move || -> bool {
            if let Ok(mut sg) = sg_rollback.lock() {
                sg.rollback().is_ok()
            } else {
                false
            }
        });

        // Register log function
        let log_clone = log.clone();
        engine.register_fn("log", move |message: &str| {
            if let Ok(mut log) = log_clone.lock() {
                log.push(ExecutionEvent {
                    timestamp_ms: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64,
                    event_type: ExecutionEventType::ScriptEnd,
                    message: message.to_string(),
                    data: None,
                });
            }
        });

        // =======================================================================
        // RLM (Recursive Language Model) API Functions
        // Based on: "Recursive Language Models: Scaling Context with Recursive Prompt Decomposition"
        // =======================================================================

        // Register load_context_var - stores large content as environment variable
        let store_load = rlm_store.clone();
        let traj_load = rlm_trajectory.clone();
        let log_load = log.clone();
        engine.register_fn("load_context_var", move |name: &str, content: &str| -> bool {
            if let Ok(mut store) = store_load.lock() {
                store.load_variable(name, content.to_string(), ContextType::String);

                // Log the operation
                Self::log_rlm_trajectory(&traj_load, RLMOperation::LoadContext {
                    var_name: name.to_string(),
                    length: content.len(),
                });
                Self::log_event(&log_load, ExecutionEventType::RLMLoadContext,
                    &format!("Loaded context '{}' ({} chars)", name, content.len()), None);
                true
            } else {
                false
            }
        });

        // Register peek_context - view a slice without tokenizing
        let store_peek = rlm_store.clone();
        let traj_peek = rlm_trajectory.clone();
        let log_peek = log.clone();
        engine.register_fn("peek_context", move |var_name: &str, start: i64, end: i64| -> Dynamic {
            if let Ok(store) = store_peek.lock() {
                let start_usize = start.max(0) as usize;
                let end_usize = end.max(0) as usize;

                match store.peek(var_name, start_usize, end_usize) {
                    Some(slice) => {
                        Self::log_rlm_trajectory(&traj_peek, RLMOperation::Peek {
                            var_name: var_name.to_string(),
                            start: start_usize,
                            end: end_usize,
                        });
                        Self::log_event(&log_peek, ExecutionEventType::RLMPeek,
                            &format!("Peeked '{}' [{}-{}]", var_name, start, end), None);
                        Dynamic::from(slice)
                    }
                    None => Dynamic::UNIT,
                }
            } else {
                Dynamic::UNIT
            }
        });

        // Register context_length - get length without accessing content
        let store_len = rlm_store.clone();
        engine.register_fn("context_length", move |var_name: &str| -> i64 {
            if let Ok(store) = store_len.lock() {
                store.length(var_name).map(|l| l as i64).unwrap_or(-1)
            } else {
                -1
            }
        });

        // Register chunk_context - split context into processable chunks
        let store_chunk = rlm_store.clone();
        let traj_chunk = rlm_trajectory.clone();
        let log_chunk = log.clone();
        let default_chunk_size = rlm_config.default_chunk_size;
        engine.register_fn("chunk_context", move |var_name: &str, chunk_size: i64| -> rhai::Array {
            if let Ok(mut store) = store_chunk.lock() {
                let size = if chunk_size <= 0 { default_chunk_size } else { chunk_size as usize };
                let chunks = store.chunk(var_name, size);
                let num_chunks = chunks.len();

                Self::log_rlm_trajectory(&traj_chunk, RLMOperation::Chunk {
                    var_name: var_name.to_string(),
                    num_chunks,
                });
                Self::log_event(&log_chunk, ExecutionEventType::RLMChunk,
                    &format!("Chunked '{}' into {} chunks", var_name, num_chunks), None);

                chunks.into_iter().map(Dynamic::from).collect()
            } else {
                rhai::Array::new()
            }
        });

        // Register regex_filter - code-based filtering using regex
        let store_regex = rlm_store.clone();
        let traj_regex = rlm_trajectory.clone();
        let log_regex = log.clone();
        engine.register_fn("regex_filter", move |var_name: &str, pattern: &str| -> rhai::Array {
            if let Ok(store) = store_regex.lock() {
                match store.regex_filter(var_name, pattern) {
                    Ok(matches) => {
                        let num_matches = matches.len();
                        Self::log_rlm_trajectory(&traj_regex, RLMOperation::RegexFilter {
                            var_name: var_name.to_string(),
                            pattern: pattern.to_string(),
                            matches: num_matches,
                        });
                        Self::log_event(&log_regex, ExecutionEventType::RLMRegexFilter,
                            &format!("Regex '{}' on '{}': {} matches", pattern, var_name, num_matches), None);
                        matches.into_iter().map(Dynamic::from).collect()
                    }
                    Err(e) => {
                        Self::log_event(&log_regex, ExecutionEventType::Error,
                            &format!("Regex error: {}", e), None);
                        rhai::Array::new()
                    }
                }
            } else {
                rhai::Array::new()
            }
        });

        // Register llm_query - recursive sub-LM call (the core RLM capability)
        let config_query = llm_config.clone();
        let traj_query = rlm_trajectory.clone();
        let log_query = log.clone();
        engine.register_fn("llm_query", move |prompt: &str| -> Dynamic {
            Self::log_rlm_trajectory(&traj_query, RLMOperation::SubQuery {
                prompt_preview: prompt.chars().take(100).collect::<String>() + "...",
                depth: 1,
            });
            Self::log_event(&log_query, ExecutionEventType::RLMSubQuery,
                &format!("Sub-LM query: {}...", &prompt.chars().take(50).collect::<String>()), None);

            // Execute the sub-LM call using a standard atom
            let result = Self::execute_spawn_atom(
                AtomType::Planner, // Use Planner for general reasoning sub-calls
                prompt,
                SpawnFlags { temperature: 0.1, ..Default::default() },
                &config_query,
                "",
                &log_query,
            );

            // Log the result
            if let Ok(result_str) = serde_json::to_string(&result) {
                Self::log_rlm_trajectory(&traj_query, RLMOperation::SubResult {
                    result_preview: result_str.chars().take(100).collect::<String>() + "...",
                });
            }

            result
        });

        // Register spawn_rlm - RLM-aware atom spawning with context variable
        let config_rlm = llm_config.clone();
        let store_rlm = rlm_store.clone();
        let log_rlm = log.clone();
        let traj_rlm = rlm_trajectory.clone();
        engine.register_fn("spawn_rlm", move |atom_type: AtomType, task: &str, context_var: &str| -> Dynamic {
            // Get context info without including full content
            let (context_len, context_preview) = if let Ok(store) = store_rlm.lock() {
                let len = store.length(context_var).unwrap_or(0);
                let preview = store.peek(context_var, 0, 500).unwrap_or_default();
                (len, preview)
            } else {
                (0, String::new())
            };

            Self::log_rlm_trajectory(&traj_rlm, RLMOperation::Start);
            Self::log_event(&log_rlm, ExecutionEventType::RLMStart,
                &format!("RLM spawn {:?} with context '{}' ({} chars)", atom_type, context_var, context_len), None);

            // Build the RLM-enhanced prompt that includes context metadata
            let rlm_prompt = format!(
                "You have access to a context variable '{}' with {} characters.\n\
                Context preview (first 500 chars):\n```\n{}\n```\n\n\
                Your task: {}\n\n\
                Use peek_context(), chunk_context(), regex_filter(), and llm_query() as needed.",
                context_var, context_len, context_preview, task
            );

            Self::execute_spawn_atom(
                atom_type,
                &rlm_prompt,
                SpawnFlags { temperature: 0.1, max_tokens: Some(4000), ..Default::default() },
                &config_rlm,
                "",
                &log_rlm,
            )
        });

        // Register has_context - check if a context variable exists
        let store_has = rlm_store.clone();
        engine.register_fn("has_context", move |var_name: &str| -> bool {
            if let Ok(store) = store_has.lock() {
                store.contains(var_name)
            } else {
                false
            }
        });

        // Register clear_context - remove a context variable
        let store_clear = rlm_store.clone();
        engine.register_fn("clear_context", move |var_name: &str| -> bool {
            if let Ok(mut store) = store_clear.lock() {
                store.remove(var_name).is_some()
            } else {
                false
            }
        });

        // Register list_contexts - list all context variable names
        let store_list = rlm_store.clone();
        engine.register_fn("list_contexts", move || -> rhai::Array {
            if let Ok(store) = store_list.lock() {
                store.list_variables().into_iter().map(Dynamic::from).collect()
            } else {
                rhai::Array::new()
            }
        });

        // =======================================================================
        // Web Research API Functions (crawl4ai integration)
        // Enables Rhai scripts to perform web research as part of agent workflows
        // =======================================================================

        // Initialize web research worker pool
        super::web_research_bridge::init_web_research_worker();

        // Register crawl_url - crawl a single URL and return content
        let log_crawl = log.clone();
        engine.register_fn("crawl_url", move |url: &str| -> Dynamic {
            Self::log_event(&log_crawl, ExecutionEventType::AtomSpawned,
                &format!("Crawling URL: {}", url), None);

            match super::web_research_bridge::crawl_url_sync(url.to_string(), true) {
                Ok(result) => {
                    Self::log_event(&log_crawl, ExecutionEventType::AtomCompleted,
                        "URL crawl completed", serde_json::to_value(&result).ok());
                    rhai::serde::to_dynamic(&result).unwrap_or(Dynamic::UNIT)
                }
                Err(e) => {
                    Self::log_event(&log_crawl, ExecutionEventType::Error,
                        &format!("Crawl failed: {}", e), None);
                    Dynamic::UNIT
                }
            }
        });

        // Register research_docs - research multiple documentation URLs
        let log_research = log.clone();
        engine.register_fn("research_docs", move |urls: rhai::Array| -> Dynamic {
            let url_strings: Vec<String> = urls.into_iter()
                .filter_map(|v| v.into_string().ok())
                .collect();

            Self::log_event(&log_research, ExecutionEventType::AtomSpawned,
                &format!("Researching {} documentation URLs", url_strings.len()), None);

            match super::web_research_bridge::research_docs_sync(url_strings) {
                Ok(result) => {
                    Self::log_event(&log_research, ExecutionEventType::AtomCompleted,
                        "Documentation research completed", serde_json::to_value(&result).ok());
                    rhai::serde::to_dynamic(&result).unwrap_or(Dynamic::UNIT)
                }
                Err(e) => {
                    Self::log_event(&log_research, ExecutionEventType::Error,
                        &format!("Research failed: {}", e), None);
                    Dynamic::UNIT
                }
            }
        });

        // Register extract_content - extract structured content using CSS selectors
        let log_extract = log.clone();
        engine.register_fn("extract_content", move |url: &str, selector: &str| -> Dynamic {
            Self::log_event(&log_extract, ExecutionEventType::AtomSpawned,
                &format!("Extracting content from {} with selector: {}", url, selector), None);

            // Build a simple CSS extraction schema
            let schema = serde_json::json!({
                "baseSelector": "body",
                "fields": [
                    {"name": "content", "selector": selector, "type": "text"}
                ]
            });

            match super::web_research_bridge::extract_content_sync(
                url.to_string(),
                "css".to_string(),
                schema,
            ) {
                Ok(result) => {
                    Self::log_event(&log_extract, ExecutionEventType::AtomCompleted,
                        "Content extraction completed", serde_json::to_value(&result).ok());
                    rhai::serde::to_dynamic(&result).unwrap_or(Dynamic::UNIT)
                }
                Err(e) => {
                    Self::log_event(&log_extract, ExecutionEventType::Error,
                        &format!("Extraction failed: {}", e), None);
                    Dynamic::UNIT
                }
            }
        });
    }

    /// Create the AtomType module for Rhai
    fn create_atom_type_module() -> rhai::Module {
        let mut module = rhai::Module::new();
        module.set_var("Search", AtomType::Search);
        module.set_var("Coder", AtomType::Coder);
        module.set_var("Reviewer", AtomType::Reviewer);
        module.set_var("Planner", AtomType::Planner);
        module.set_var("Validator", AtomType::Validator);
        module.set_var("Tester", AtomType::Tester);
        module.set_var("Architect", AtomType::Architect);
        module.set_var("GritsAnalyzer", AtomType::GritsAnalyzer);
        module.set_var("RLMProcessor", AtomType::RLMProcessor);
        module.set_var("WebResearcher", AtomType::WebResearcher);
        module
    }

    /// Compile a Rhai script
    pub fn compile(&self, script: &str) -> Result<AST, Box<EvalAltResult>> {
        Ok(self.engine.compile(script)?)
    }

    /// P3-2: Execute a Rhai script with automatic snapshot and rollback on failure
    /// PRD 5.1: "Before any Rhai script touches disk, gitoxide creates a blob"
    pub fn execute_script(&self, script: &str) -> Result<Dynamic, Box<EvalAltResult>> {
        self.execute_script_with_recovery(script, true)
    }

    /// Execute a Rhai script with optional automatic recovery
    pub fn execute_script_with_recovery(&self, script: &str, auto_recover: bool) -> Result<Dynamic, Box<EvalAltResult>> {
        let mut scope = Scope::new();

        // Add workspace path to scope
        scope.push("WORKSPACE", self.workspace_path.clone());

        // P3-2: Create pre-execution snapshot for automatic recovery
        let snapshot_id = if auto_recover {
            match self.shadow_git.lock() {
                Ok(mut git) => {
                    match git.snapshot("Pre-script execution snapshot") {
                        Ok(snapshot) => {
                            Self::log_event(&self.execution_log, ExecutionEventType::Snapshot,
                                &format!("Created pre-execution snapshot: {}", snapshot.id),
                                Some(serde_json::json!({"snapshot_id": snapshot.id})));
                            Some(snapshot.id)
                        }
                        Err(e) => {
                            Self::log_event(&self.execution_log, ExecutionEventType::Error,
                                &format!("Failed to create snapshot: {}", e), None);
                            None
                        }
                    }
                }
                Err(_) => None
            }
        } else {
            None
        };

        // Log script start
        if let Ok(mut log) = self.execution_log.lock() {
            log.push(ExecutionEvent {
                timestamp_ms: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                event_type: ExecutionEventType::ScriptStart,
                message: "Script execution started".to_string(),
                data: snapshot_id.as_ref().map(|id| serde_json::json!({"snapshot_id": id})),
            });
        }

        let result = self.engine.eval_with_scope::<Dynamic>(&mut scope, script);

        // P3-2: Automatic rollback on failure
        if result.is_err() && auto_recover {
            if let Some(ref snap_id) = snapshot_id {
                if let Ok(mut git) = self.shadow_git.lock() {
                    match git.rollback_to(snap_id) {
                        Ok(()) => {
                            Self::log_event(&self.execution_log, ExecutionEventType::Rollback,
                                &format!("Auto-rollback to snapshot {}", snap_id),
                                Some(serde_json::json!({"snapshot_id": snap_id, "reason": "script_failure"})));
                        }
                        Err(e) => {
                            Self::log_event(&self.execution_log, ExecutionEventType::Error,
                                &format!("Auto-rollback failed: {}", e), None);
                        }
                    }
                }
            }
        }

        // Log script end
        if let Ok(mut log) = self.execution_log.lock() {
            let event_type = if result.is_ok() {
                ExecutionEventType::ScriptEnd
            } else {
                ExecutionEventType::Error
            };
            log.push(ExecutionEvent {
                timestamp_ms: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                event_type,
                message: if result.is_ok() {
                    "Script completed successfully".to_string()
                } else {
                    format!("Script failed{}", if auto_recover && snapshot_id.is_some() { " (auto-rolled back)" } else { "" })
                },
                data: None,
            });
        }

        result
    }

    /// Execute a compiled AST
    pub fn execute_ast(&self, ast: &AST) -> Result<Dynamic, Box<EvalAltResult>> {
        let mut scope = Scope::new();
        scope.push("WORKSPACE", self.workspace_path.clone());
        self.engine.eval_ast_with_scope::<Dynamic>(&mut scope, ast)
    }

    /// Get the execution log
    pub fn get_execution_log(&self) -> Vec<ExecutionEvent> {
        self.execution_log.lock().map(|log| log.clone()).unwrap_or_default()
    }

    /// Clear the execution log
    pub fn clear_log(&self) {
        if let Ok(mut log) = self.execution_log.lock() {
            log.clear();
        }
    }

    /// Load and execute the voting.rhai preamble
    pub fn load_preamble(&mut self) -> Result<(), Box<EvalAltResult>> {
        let preamble = include_str!("../scripts/voting.rhai");
        let ast = self.engine.compile(preamble)?;
        let mut scope = Scope::new();
        self.engine.run_ast_with_scope(&mut scope, &ast)?;
        Ok(())
    }

    /// Execute spawn_atom by bridging to async AtomExecutor
    /// Uses the dedicated AtomWorkerPool for safe async-to-sync bridging
    fn execute_spawn_atom(
        atom_type: AtomType,
        prompt: &str,
        flags: SpawnFlags,
        _llm_config: &LlmConfig,
        _workspace_path: &str,
        log: &Arc<Mutex<Vec<ExecutionEvent>>>,
    ) -> Dynamic {
        // Log atom spawned
        Self::log_event(log, ExecutionEventType::AtomSpawned,
            &format!("Spawning {:?} atom", atom_type), None);

        // Build the atom input
        let input = AtomInput::new(atom_type.clone(), prompt)
            .with_flags(flags);

        // Use the worker pool bridge for safe async-to-sync execution
        // This avoids creating a new runtime per call and handles async context properly
        let result = super::atom_bridge::execute_atom_sync(input);

        match result {
            Ok(atom_result) => {
                // Log completion
                Self::log_event(log, ExecutionEventType::AtomCompleted,
                    &format!("{:?} atom completed", atom_type),
                    serde_json::to_value(&atom_result).ok());

                rhai::serde::to_dynamic(&atom_result).unwrap_or(Dynamic::UNIT)
            }
            Err(e) => {
                Self::log_event(log, ExecutionEventType::Error,
                    &format!("Atom failed: {}", e), None);

                let error_result = AtomResult::failure(atom_type, e.clone(), vec![e]);
                rhai::serde::to_dynamic(&error_result).unwrap_or(Dynamic::UNIT)
            }
        }
    }

    /// Execute consensus voting by bridging to async voting
    fn execute_consensus(
        atom_type: AtomType,
        task: &str,
        k_threshold: usize,
        llm_config: &LlmConfig,
        workspace_path: &str,
        log: &Arc<Mutex<Vec<ExecutionEvent>>>,
    ) -> Dynamic {
        // Log consensus start
        Self::log_event(log, ExecutionEventType::ConsensusStart,
            &format!("Starting k={} consensus for {:?}", k_threshold, atom_type), None);

        let config = ConsensusConfig {
            k_threshold,
            ..Default::default()
        };

        // Bridge async to sync
        let result = match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                let llm = llm_config.clone();
                let ws = workspace_path.to_string();
                let task_str = task.to_string();
                let at = atom_type.clone();

                std::thread::scope(|s| {
                    s.spawn(move || {
                        handle.block_on(voting_run_consensus(at, &task_str, config, &llm, &ws))
                    }).join().unwrap_or_else(|_| ConsensusResult::failure(
                        "Thread panicked".to_string(), HashMap::new(), 0, 0, 0
                    ))
                })
            }
            Err(_) => {
                match tokio::runtime::Runtime::new() {
                    Ok(rt) => {
                        let llm = llm_config.clone();
                        let ws = workspace_path.to_string();
                        rt.block_on(voting_run_consensus(atom_type.clone(), &ws, config, &llm, &ws))
                    }
                    Err(e) => ConsensusResult::failure(
                        format!("Failed to create runtime: {}", e),
                        HashMap::new(), 0, 0, 0
                    ),
                }
            }
        };

        // Log consensus end
        Self::log_event(log, ExecutionEventType::ConsensusEnd,
            &format!("Consensus {} (winner: {:?})",
                if result.reached { "reached" } else { "failed" },
                result.winner),
            serde_json::to_value(&result).ok());

        rhai::serde::to_dynamic(&result).unwrap_or(Dynamic::UNIT)
    }

    /// Helper to log execution events
    fn log_event(
        log: &Arc<Mutex<Vec<ExecutionEvent>>>,
        event_type: ExecutionEventType,
        message: &str,
        data: Option<serde_json::Value>,
    ) {
        if let Ok(mut log) = log.lock() {
            log.push(ExecutionEvent {
                timestamp_ms: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                event_type,
                message: message.to_string(),
                data,
            });
        }
    }

    /// Helper to log RLM trajectory steps for visualization
    fn log_rlm_trajectory(
        trajectory: &Arc<Mutex<Vec<RLMTrajectoryStep>>>,
        operation: RLMOperation,
    ) {
        if let Ok(mut traj) = trajectory.lock() {
            let step = traj.len();
            traj.push(RLMTrajectoryStep {
                step,
                operation: operation.clone(),
                description: match &operation {
                    RLMOperation::Start => "RLM execution started".to_string(),
                    RLMOperation::Peek { var_name, start, end } =>
                        format!("Peeked '{}' chars {}-{}", var_name, start, end),
                    RLMOperation::Chunk { var_name, num_chunks } =>
                        format!("Chunked '{}' into {} pieces", var_name, num_chunks),
                    RLMOperation::SubQuery { prompt_preview, depth } =>
                        format!("Sub-query (depth {}): {}", depth, prompt_preview),
                    RLMOperation::SubResult { result_preview } =>
                        format!("Sub-result: {}", result_preview),
                    RLMOperation::RegexFilter { var_name, pattern, matches } =>
                        format!("Regex '{}' on '{}': {} matches", pattern, var_name, matches),
                    RLMOperation::LoadContext { var_name, length } =>
                        format!("Loaded '{}' ({} chars)", var_name, length),
                    RLMOperation::Final { answer_preview } =>
                        format!("Final answer: {}", answer_preview),
                    RLMOperation::Error { message } =>
                        format!("Error: {}", message),
                },
                data: serde_json::to_value(&operation).ok(),
                timestamp_ms: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
            });
        }
    }

    /// Get the RLM trajectory for visualization
    pub fn get_rlm_trajectory(&self) -> Vec<RLMTrajectoryStep> {
        self.rlm_trajectory.lock().map(|t| t.clone()).unwrap_or_default()
    }

    /// Clear the RLM trajectory
    pub fn clear_rlm_trajectory(&self) {
        if let Ok(mut traj) = self.rlm_trajectory.lock() {
            traj.clear();
        }
    }

    /// Get the RLM context store (for external access)
    pub fn get_rlm_store(&self) -> SharedRLMContextStore {
        self.rlm_context_store.clone()
    }

    /// Get RLM configuration
    pub fn get_rlm_config(&self) -> &RLMConfig {
        &self.rlm_config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn create_test_runtime() -> CodeModeRuntime {
        let temp_dir = env::temp_dir().join("cerebras_maker_test");
        std::fs::create_dir_all(&temp_dir).ok();
        CodeModeRuntime::new(temp_dir.to_str().unwrap()).unwrap()
    }

    #[test]
    fn test_runtime_creation() {
        let runtime = create_test_runtime();
        assert!(runtime.get_rlm_config().max_depth > 0);
    }

    #[test]
    fn test_rlm_store_access() {
        let runtime = create_test_runtime();
        let store = runtime.get_rlm_store();

        {
            let mut guard = store.lock().unwrap();
            guard.load_variable("test", "content".to_string(), ContextType::String);
        }

        {
            let guard = store.lock().unwrap();
            assert!(guard.contains("test"));
        }
    }

    #[test]
    fn test_rlm_trajectory_operations() {
        let runtime = create_test_runtime();

        // Initially empty
        assert!(runtime.get_rlm_trajectory().is_empty());

        // Clear should work on empty
        runtime.clear_rlm_trajectory();
        assert!(runtime.get_rlm_trajectory().is_empty());
    }

    #[test]
    fn test_rhai_load_context_var() {
        let runtime = create_test_runtime();

        let script = r#"
            let result = load_context_var("my_var", "Hello, World!");
            result
        "#;

        let result = runtime.execute_script(script);
        assert!(result.is_ok());

        // Verify the variable was stored
        let store = runtime.get_rlm_store();
        let guard = store.lock().unwrap();
        assert!(guard.contains("my_var"));
        assert_eq!(guard.length("my_var"), Some(13));
    }

    #[test]
    fn test_rhai_peek_context() {
        let runtime = create_test_runtime();

        let script = r#"
            load_context_var("data", "0123456789");
            let slice = peek_context("data", 2, 7);
            slice
        "#;

        let result = runtime.execute_script(script);
        assert!(result.is_ok());

        let value = result.unwrap();
        assert_eq!(value.into_string().unwrap(), "23456");
    }

    #[test]
    fn test_rhai_context_length() {
        let runtime = create_test_runtime();

        let script = r#"
            load_context_var("content", "Hello, World!");
            let len = context_length("content");
            len
        "#;

        let result = runtime.execute_script(script);
        assert!(result.is_ok());

        let value = result.unwrap();
        assert_eq!(value.as_int().unwrap(), 13);
    }

    #[test]
    fn test_rhai_context_length_nonexistent() {
        let runtime = create_test_runtime();

        let script = r#"
            context_length("nonexistent")
        "#;

        let result = runtime.execute_script(script);
        assert!(result.is_ok());

        let value = result.unwrap();
        assert_eq!(value.as_int().unwrap(), -1);
    }

    #[test]
    fn test_rhai_chunk_context() {
        let runtime = create_test_runtime();

        let script = r#"
            load_context_var("data", "ABCDEFGHIJ");
            let chunks = chunk_context("data", 3);
            chunks.len()
        "#;

        let result = runtime.execute_script(script);
        assert!(result.is_ok());

        let value = result.unwrap();
        assert_eq!(value.as_int().unwrap(), 4);
    }

    #[test]
    fn test_rhai_regex_filter() {
        let runtime = create_test_runtime();

        let script = r#"
            load_context_var("code", "fn main() {}\nlet x = 1;\nfn helper() {}");
            let matches = regex_filter("code", "^fn ");
            matches.len()
        "#;

        let result = runtime.execute_script(script);
        assert!(result.is_ok());

        let value = result.unwrap();
        assert_eq!(value.as_int().unwrap(), 2);
    }

    #[test]
    fn test_rhai_has_context() {
        let runtime = create_test_runtime();

        let script = r#"
            let before = has_context("test_var");
            load_context_var("test_var", "data");
            let after = has_context("test_var");
            [before, after]
        "#;

        let result = runtime.execute_script(script);
        assert!(result.is_ok());

        let value = result.unwrap();
        let arr = value.into_array().unwrap();
        assert!(!arr[0].as_bool().unwrap());
        assert!(arr[1].as_bool().unwrap());
    }

    #[test]
    fn test_rhai_clear_context() {
        let runtime = create_test_runtime();

        let script = r#"
            load_context_var("to_clear", "data");
            let before = has_context("to_clear");
            clear_context("to_clear");
            let after = has_context("to_clear");
            [before, after]
        "#;

        let result = runtime.execute_script(script);
        assert!(result.is_ok());

        let value = result.unwrap();
        let arr = value.into_array().unwrap();
        assert!(arr[0].as_bool().unwrap());
        assert!(!arr[1].as_bool().unwrap());
    }

    #[test]
    fn test_rhai_list_contexts() {
        let runtime = create_test_runtime();

        let script = r#"
            load_context_var("var1", "a");
            load_context_var("var2", "b");
            load_context_var("var3", "c");
            let contexts = list_contexts();
            contexts.len()
        "#;

        let result = runtime.execute_script(script);
        assert!(result.is_ok());

        let value = result.unwrap();
        assert_eq!(value.as_int().unwrap(), 3);
    }

    #[test]
    fn test_rlm_trajectory_logging() {
        let runtime = create_test_runtime();

        let script = r#"
            load_context_var("test", "content");
            peek_context("test", 0, 5);
            chunk_context("test", 2);
        "#;

        let _ = runtime.execute_script(script);

        // Check that trajectory was logged
        let trajectory = runtime.get_rlm_trajectory();
        assert!(trajectory.len() >= 3); // At least load, peek, chunk
    }

    #[test]
    fn test_execution_log_rlm_events() {
        let runtime = create_test_runtime();

        let script = r#"
            load_context_var("test", "content");
        "#;

        let _ = runtime.execute_script(script);

        // Check execution log contains RLM events
        let log = runtime.get_execution_log();
        let has_rlm_event = log.iter().any(|e| matches!(e.event_type, ExecutionEventType::RLMLoadContext));
        assert!(has_rlm_event);
    }

    #[test]
    fn test_rlm_config_custom() {
        let temp_dir = env::temp_dir().join("cerebras_maker_test_custom");
        std::fs::create_dir_all(&temp_dir).ok();
        let custom_config = RLMConfig {
            max_depth: 3,
            max_iterations: 50,
            default_chunk_size: 100_000,
            rlm_threshold: 75_000,
            use_sub_model: true,
            sub_model_key: Some("gpt-4".to_string()),
        };

        let runtime = CodeModeRuntime::with_full_config(
            temp_dir.to_str().unwrap(),
            LlmConfig::cerebras(),
            custom_config,
        ).unwrap();

        let config = runtime.get_rlm_config();
        assert_eq!(config.max_depth, 3);
        assert_eq!(config.max_iterations, 50);
        assert_eq!(config.default_chunk_size, 100_000);
        assert!(config.use_sub_model);
    }

    #[test]
    fn test_atom_type_module_includes_rlm_processor() {
        let runtime = create_test_runtime();

        // This tests that RLMProcessor is available in the AtomType module
        let script = r#"
            let atom = AtomType::RLMProcessor;
            true
        "#;

        let result = runtime.execute_script(script);
        assert!(result.is_ok());
    }
}
