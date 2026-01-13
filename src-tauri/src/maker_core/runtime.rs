// Cerebras-MAKER: Rhai Code Mode Runtime
// PRD Section 4: The Logic Layer - Sandboxed Scripting Runtime

use super::atom::{AtomResult, AtomType, SpawnFlags};
use super::shadow_git::ShadowGit;
use super::voting::{ConsensusConfig, ConsensusResult};
use crate::grits;
use rhai::{Dynamic, Engine, EvalAltResult, Scope, AST};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// The Code Mode Runtime - executes Rhai scripts with MAKER API
pub struct CodeModeRuntime {
    engine: Engine,
    shadow_git: Arc<Mutex<ShadowGit>>,
    workspace_path: String,
    execution_log: Arc<Mutex<Vec<ExecutionEvent>>>,
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
}

impl CodeModeRuntime {
    /// Create a new Code Mode Runtime
    pub fn new(workspace_path: &str) -> Result<Self, Box<EvalAltResult>> {
        let mut engine = Engine::new();
        let shadow_git = Arc::new(Mutex::new(ShadowGit::new(workspace_path)));
        let execution_log = Arc::new(Mutex::new(Vec::new()));

        // Register MAKER API functions
        Self::register_api(&mut engine, shadow_git.clone(), execution_log.clone());

        Ok(Self {
            engine,
            shadow_git,
            workspace_path: workspace_path.to_string(),
            execution_log,
        })
    }

    /// Register the MAKER API into the Rhai engine
    fn register_api(
        engine: &mut Engine,
        shadow_git: Arc<Mutex<ShadowGit>>,
        log: Arc<Mutex<Vec<ExecutionEvent>>>,
    ) {
        // Register AtomType enum
        engine.register_type_with_name::<AtomType>("AtomType");
        engine.register_static_module("AtomType", Self::create_atom_type_module().into());

        // Register spawn_atom function
        engine.register_fn("spawn_atom", move |atom_type: AtomType, prompt: &str| -> Dynamic {
            // Placeholder - actual implementation requires async
            let result = AtomResult::success(atom_type, format!("Mock result for: {}", prompt), 100, 50);
            rhai::serde::to_dynamic(&result).unwrap_or(Dynamic::UNIT)
        });

        // Register spawn_atom_with_flags
        engine.register_fn("spawn_atom_with_flags", 
            move |atom_type: AtomType, prompt: &str, flags: Dynamic| -> Dynamic {
                let _flags: SpawnFlags = rhai::serde::from_dynamic(&flags).unwrap_or_default();
                let result = AtomResult::success(atom_type, format!("Mock result for: {}", prompt), 100, 50);
                rhai::serde::to_dynamic(&result).unwrap_or(Dynamic::UNIT)
            }
        );

        // Register run_consensus
        engine.register_fn("run_consensus", 
            move |atom_type: AtomType, task: &str, k_threshold: i64| -> Dynamic {
                let config = ConsensusConfig {
                    k_threshold: k_threshold as usize,
                    ..Default::default()
                };
                let result = ConsensusResult::failure(
                    "Mock consensus - use async runtime".to_string(),
                    HashMap::new(), 0, 0, 0
                );
                rhai::serde::to_dynamic(&result).unwrap_or(Dynamic::UNIT)
            }
        );

        // Register check_red_flags
        engine.register_fn("check_red_flags", move |code: &str| -> bool {
            // Uses grits-core red flag checking
            if let Some(graph) = grits::get_cached_graph() {
                let result = grits::red_flag_check(&graph, 0);
                result.introduced_cycle
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
    }

    /// Create the AtomType module for Rhai
    fn create_atom_type_module() -> rhai::Module {
        let mut module = rhai::Module::new();
        module.set_var("Search", AtomType::Search);
        module.set_var("Coder", AtomType::Coder);
        module.set_var("Reviewer", AtomType::Reviewer);
        module.set_var("Planner", AtomType::Planner);
        module.set_var("Validator", AtomType::Validator);
        module
    }

    /// Compile a Rhai script
    pub fn compile(&self, script: &str) -> Result<AST, Box<EvalAltResult>> {
        Ok(self.engine.compile(script)?)
    }

    /// Execute a Rhai script
    pub fn execute_script(&self, script: &str) -> Result<Dynamic, Box<EvalAltResult>> {
        let mut scope = Scope::new();

        // Add workspace path to scope
        scope.push("WORKSPACE", self.workspace_path.clone());

        // Log script start
        if let Ok(mut log) = self.execution_log.lock() {
            log.push(ExecutionEvent {
                timestamp_ms: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                event_type: ExecutionEventType::ScriptStart,
                message: "Script execution started".to_string(),
                data: None,
            });
        }

        let result = self.engine.eval_with_scope::<Dynamic>(&mut scope, script);

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
                message: if result.is_ok() { "Script completed" } else { "Script failed" }.to_string(),
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
}

