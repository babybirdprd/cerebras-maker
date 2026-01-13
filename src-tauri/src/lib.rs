// Cerebras-MAKER: Autonomous Coding System
// PRD: "Massively Decomposed, Topologically Aware, 100% Reliable"

use std::path::Path;
use std::sync::Mutex;

// Re-export grits-core modules for use throughout the application
pub use grits_core::context::MiniCodebase;
pub use grits_core::topology::analysis::TopologicalAnalysis;
pub use grits_core::topology::{DependencyEdge, Symbol, SymbolGraph};

// MAKER Core module - Rhai runtime, voting, shadow git
pub mod maker_core;

// Agent System module - Interrogator, Architect, Orchestrator
pub mod agents;

// LLM abstraction layer - unified provider interface
pub mod llm;

// Script Generator framework - plugin architecture for Rhai generation
pub mod generators;

// Project template system
pub mod templates;

// Re-export maker_core types for convenience
pub use maker_core::{AtomType, AtomResult, CodeModeRuntime, ShadowGit, ConsensusConfig, ConsensusResult};

// Re-export agent types
pub use agents::{Agent, Interrogator, Architect, Orchestrator, AgentContext, AgentOutput, AtomExecutor, AtomInput, AtomOutput, CodeChange, ReviewResult, ValidationResult};

// Re-export LLM types
pub use llm::{LlmProvider, LlmConfig, Message, Role};

// Re-export generator types
pub use generators::{ScriptGenerator, GeneratorRegistry, GenerationResult, RhaiScriptGenerator, TaskScriptGenerator};

// Global runtime instance (lazy initialized)
static RUNTIME: Mutex<Option<CodeModeRuntime>> = Mutex::new(None);

// Global ShadowGit instance for transactional file operations
static SHADOW_GIT: Mutex<Option<ShadowGit>> = Mutex::new(None);

/// Module for grits-core integration functionality
pub mod grits {
    use super::*;
    use grits_core::topology::scanner::DirectoryScanner;
    use std::sync::Mutex;

    /// Cached SymbolGraph for the workspace (thread-safe)
    static WORKSPACE_GRAPH: Mutex<Option<SymbolGraph>> = Mutex::new(None);

    /// Build a SymbolGraph from a workspace directory
    /// PRD 3.1: "Load the Semantic Graph (Cached in RAM)"
    pub fn load_workspace_graph(workspace_path: &str) -> Result<SymbolGraph, String> {
        let path = Path::new(workspace_path);
        if !path.exists() {
            return Err(format!("Workspace path does not exist: {}", workspace_path));
        }

        let scanner = DirectoryScanner::new();
        let graph = scanner
            .scan(path)
            .map_err(|e| format!("Failed to scan workspace: {}", e))?;

        // Cache the graph
        if let Ok(mut cached) = WORKSPACE_GRAPH.lock() {
            *cached = Some(graph.clone());
        }

        Ok(graph)
    }

    /// Get the cached workspace graph
    pub fn get_cached_graph() -> Option<SymbolGraph> {
        WORKSPACE_GRAPH.lock().ok().and_then(|g| g.clone())
    }

    /// Assemble a MiniCodebase for context engineering
    /// PRD 3.1: "Semantic Tree-Shaking" - Extract only topologically-relevant code
    pub fn assemble_context(
        graph: &SymbolGraph,
        seed_symbols: Vec<String>,
        depth: usize,
        strength_threshold: f32,
        issue_id: Option<String>,
    ) -> MiniCodebase {
        MiniCodebase::assemble(graph, seed_symbols, depth, strength_threshold, issue_id)
    }

    /// Perform red-flag check for architectural violations
    /// PRD 3.2: "Architectural Red-Flagging" - Detect cycle membership via triangles
    pub fn red_flag_check(graph: &SymbolGraph, previous_betti_1: usize) -> RedFlagResult {
        let analysis = TopologicalAnalysis::analyze(graph);

        RedFlagResult {
            introduced_cycle: analysis.betti_1 > previous_betti_1,
            betti_1: analysis.betti_1,
            betti_0: analysis.betti_0,
            triangle_count: analysis.triangle_count,
            solid_score: analysis.solid_score().normalized,
            cycles_detected: analysis
                .triangles
                .iter()
                .map(|t| t.nodes.to_vec())
                .collect(),
        }
    }

    /// Result of architectural red-flag check
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct RedFlagResult {
        pub introduced_cycle: bool,
        pub betti_1: usize,
        pub betti_0: usize,
        pub triangle_count: usize,
        pub solid_score: f32,
        pub cycles_detected: Vec<Vec<String>>,
    }
}

// Tauri commands for frontend integration

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Load the semantic graph from a workspace directory
#[tauri::command]
fn load_symbol_graph(workspace_path: String) -> Result<serde_json::Value, String> {
    let graph = grits::load_workspace_graph(&workspace_path)?;
    serde_json::to_value(&graph).map_err(|e| e.to_string())
}

/// Assemble a mini codebase for an issue
#[tauri::command]
fn assemble_mini_codebase(
    seed_symbols: Vec<String>,
    depth: usize,
    strength_threshold: f32,
    issue_id: Option<String>,
) -> Result<serde_json::Value, String> {
    let graph = grits::get_cached_graph().ok_or("No cached graph. Call load_symbol_graph first.")?;
    let mini = grits::assemble_context(&graph, seed_symbols, depth, strength_threshold, issue_id);
    serde_json::to_value(&mini).map_err(|e| e.to_string())
}

/// Check for architectural red flags in the current graph
#[tauri::command]
fn check_red_flags(previous_betti_1: usize) -> Result<grits::RedFlagResult, String> {
    let graph = grits::get_cached_graph().ok_or("No cached graph. Call load_symbol_graph first.")?;
    Ok(grits::red_flag_check(&graph, previous_betti_1))
}

/// Analyze topology and return full analysis
#[tauri::command]
fn analyze_topology() -> Result<serde_json::Value, String> {
    let graph = grits::get_cached_graph().ok_or("No cached graph. Call load_symbol_graph first.")?;
    let analysis = TopologicalAnalysis::analyze(&graph);
    serde_json::to_value(&analysis).map_err(|e| e.to_string())
}

// ============================================================================
// MAKER Runtime Commands (PRD Section 4)
// ============================================================================

/// Initialize the MAKER runtime for a workspace
#[tauri::command]
fn init_runtime(workspace_path: String) -> Result<String, String> {
    let runtime = CodeModeRuntime::new(&workspace_path)
        .map_err(|e| format!("Failed to initialize runtime: {}", e))?;

    if let Ok(mut global) = RUNTIME.lock() {
        *global = Some(runtime);
    }

    // Also initialize the ShadowGit for the workspace
    let mut shadow_git = ShadowGit::new(&workspace_path);
    shadow_git.init().map_err(|e| format!("Failed to initialize shadow git: {}", e))?;

    if let Ok(mut sg) = SHADOW_GIT.lock() {
        *sg = Some(shadow_git);
    }

    Ok("Runtime initialized".to_string())
}

/// Execute a Rhai script
#[tauri::command]
fn execute_script(script: String) -> Result<serde_json::Value, String> {
    let runtime = RUNTIME.lock()
        .map_err(|_| "Failed to acquire runtime lock")?;

    let runtime = runtime.as_ref()
        .ok_or("Runtime not initialized. Call init_runtime first.")?;

    let result = runtime.execute_script(&script)
        .map_err(|e| format!("Script execution failed: {}", e))?;

    serde_json::to_value(&result.to_string()).map_err(|e| e.to_string())
}

/// Get the execution log (for Cockpit)
#[tauri::command]
fn get_execution_log() -> Result<serde_json::Value, String> {
    let runtime = RUNTIME.lock()
        .map_err(|_| "Failed to acquire runtime lock")?;

    let runtime = runtime.as_ref()
        .ok_or("Runtime not initialized")?;

    let log = runtime.get_execution_log();
    serde_json::to_value(&log).map_err(|e| e.to_string())
}

// ============================================================================
// Shadow Git Commands - Transactional File System
// ============================================================================

/// Create a snapshot (Shadow Git)
/// PRD 5.1: "Before any Rhai script touches disk, gitoxide creates a blob"
#[tauri::command]
fn create_snapshot(message: String) -> Result<serde_json::Value, String> {
    let mut sg = SHADOW_GIT.lock()
        .map_err(|_| "Failed to acquire shadow git lock")?;

    let shadow_git = sg.as_mut()
        .ok_or("Shadow Git not initialized. Call init_runtime first.")?;

    let snapshot = shadow_git.snapshot(&message)
        .map_err(|e| format!("Failed to create snapshot: {}", e))?;

    Ok(serde_json::json!({
        "id": snapshot.id,
        "message": snapshot.message,
        "timestamp_ms": snapshot.timestamp_ms,
        "commit_hash": snapshot.commit_hash
    }))
}

/// Rollback to the previous snapshot
/// PRD 5.1: "gitoxide reverts the index to the snapshot instantly"
#[tauri::command]
fn rollback_snapshot() -> Result<String, String> {
    let mut sg = SHADOW_GIT.lock()
        .map_err(|_| "Failed to acquire shadow git lock")?;

    let shadow_git = sg.as_mut()
        .ok_or("Shadow Git not initialized. Call init_runtime first.")?;

    shadow_git.rollback()
        .map_err(|e| format!("Failed to rollback: {}", e))?;

    Ok("Rolled back to previous snapshot".to_string())
}

/// Rollback to a specific snapshot by ID
#[tauri::command]
fn rollback_to_snapshot(snapshot_id: String) -> Result<String, String> {
    let mut sg = SHADOW_GIT.lock()
        .map_err(|_| "Failed to acquire shadow git lock")?;

    let shadow_git = sg.as_mut()
        .ok_or("Shadow Git not initialized. Call init_runtime first.")?;

    shadow_git.rollback_to(&snapshot_id)
        .map_err(|e| format!("Failed to rollback to snapshot: {}", e))?;

    Ok(format!("Rolled back to snapshot: {}", snapshot_id))
}

/// Squash all snapshots into a single commit
/// PRD 5.1: "Only when PLAN.md is marked Complete does Shadow Repo squash"
#[tauri::command]
fn squash_snapshots(final_message: String) -> Result<serde_json::Value, String> {
    let mut sg = SHADOW_GIT.lock()
        .map_err(|_| "Failed to acquire shadow git lock")?;

    let shadow_git = sg.as_mut()
        .ok_or("Shadow Git not initialized. Call init_runtime first.")?;

    let commit_hash = shadow_git.squash(&final_message)
        .map_err(|e| format!("Failed to squash: {}", e))?;

    Ok(serde_json::json!({
        "message": final_message,
        "commit_hash": commit_hash
    }))
}

/// Get all current snapshots
#[tauri::command]
fn get_snapshots() -> Result<serde_json::Value, String> {
    let sg = SHADOW_GIT.lock()
        .map_err(|_| "Failed to acquire shadow git lock")?;

    let shadow_git = sg.as_ref()
        .ok_or("Shadow Git not initialized. Call init_runtime first.")?;

    let snapshots: Vec<serde_json::Value> = shadow_git.get_snapshots()
        .iter()
        .map(|s| serde_json::json!({
            "id": s.id,
            "message": s.message,
            "timestamp_ms": s.timestamp_ms,
            "commit_hash": s.commit_hash
        }))
        .collect();

    Ok(serde_json::json!(snapshots))
}

/// Checkout to a specific git commit (for time travel)
#[tauri::command]
fn checkout_commit(commit_hash: String) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(["checkout", &commit_hash])
        .output()
        .map_err(|e| format!("Failed to checkout: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Checkout failed: {}", stderr));
    }

    Ok(format!("Checked out to commit: {}", commit_hash))
}

/// Get git history (for Time Machine)
#[tauri::command]
fn get_git_history(limit: usize) -> Result<serde_json::Value, String> {
    let output = std::process::Command::new("git")
        .args(["log", "--oneline", "-n", &limit.to_string()])
        .output()
        .map_err(|e| format!("Failed to get git history: {}", e))?;

    let entries: Vec<serde_json::Value> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| {
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            serde_json::json!({
                "hash": parts.first().unwrap_or(&""),
                "message": parts.get(1).unwrap_or(&"")
            })
        })
        .collect();

    Ok(serde_json::json!(entries))
}

/// Get current working directory
#[tauri::command]
fn get_cwd() -> Result<String, String> {
    std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| format!("Failed to get cwd: {}", e))
}

// ============================================================================
// Settings Persistence Commands
// ============================================================================

/// Settings structure for persistence
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProviderConfig {
    pub provider: String,
    pub model: String,
    pub temperature: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentConfig {
    pub interrogator: ProviderConfig,
    pub architect: ProviderConfig,
    pub orchestrator: ProviderConfig,
    pub coder: ProviderConfig,
    pub reviewer: ProviderConfig,
    pub tester: ProviderConfig,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApiKeys {
    pub openai: String,
    pub anthropic: String,
    pub cerebras: String,
    pub ollama_url: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppSettings {
    pub agent_config: AgentConfig,
    pub api_keys: ApiKeys,
}

fn get_settings_path() -> std::path::PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    config_dir.join("cerebras-maker").join("settings.json")
}

/// Save application settings
#[tauri::command]
fn save_settings(settings: AppSettings) -> Result<(), String> {
    let path = get_settings_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config dir: {}", e))?;
    }
    let json = serde_json::to_string_pretty(&settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    std::fs::write(&path, json)
        .map_err(|e| format!("Failed to write settings: {}", e))
}

/// Load application settings
#[tauri::command]
fn load_settings() -> Result<AppSettings, String> {
    let path = get_settings_path();
    if !path.exists() {
        return Err("Settings file not found".to_string());
    }
    let json = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read settings: {}", e))?;
    serde_json::from_str(&json)
        .map_err(|e| format!("Failed to parse settings: {}", e))
}

// ============================================================================
// L1 Interrogation Commands (PRD Analysis)
// ============================================================================

use std::sync::Mutex as StdMutex;

/// Global settings cache
static CACHED_SETTINGS: StdMutex<Option<AppSettings>> = StdMutex::new(None);

/// Get LLM provider from cached settings
fn get_llm_provider() -> Result<llm::LlmProvider, String> {
    let settings = CACHED_SETTINGS.lock()
        .map_err(|_| "Failed to lock settings")?;

    let settings = settings.as_ref()
        .or_else(|| {
            // Try to load from disk
            let path = get_settings_path();
            if path.exists() {
                std::fs::read_to_string(&path).ok()
                    .and_then(|json| serde_json::from_str::<AppSettings>(&json).ok())
                    .as_ref()
                    .map(|_| ())
            } else {
                None
            };
            None
        });

    // Use default config if no settings
    let config = if let Some(s) = settings {
        let interrogator_config = &s.agent_config.interrogator;
        let api_key = match interrogator_config.provider.as_str() {
            "openai" => Some(s.api_keys.openai.clone()),
            "anthropic" => Some(s.api_keys.anthropic.clone()),
            "cerebras" => Some(s.api_keys.cerebras.clone()),
            _ => None,
        };

        llm::LlmConfig {
            provider: match interrogator_config.provider.as_str() {
                "anthropic" => llm::ProviderType::Anthropic,
                "cerebras" => llm::ProviderType::Cerebras,
                "ollama" => llm::ProviderType::Ollama,
                _ => llm::ProviderType::OpenAI,
            },
            model: interrogator_config.model.clone(),
            api_key,
            base_url: None,
            temperature: interrogator_config.temperature,
            max_tokens: 4096,
        }
    } else {
        llm::LlmConfig::default()
    };

    llm::LlmProvider::new(config).map_err(|e| e.to_string())
}

/// Analyze a PRD and start interrogation
#[tauri::command]
async fn analyze_prd(content: String, filename: String) -> Result<serde_json::Value, String> {
    // Try to use LLM, fall back to mock if not configured
    match get_llm_provider() {
        Ok(provider) => {
            let system_prompt = r#"You are the L1 Product Orchestrator in the Cerebras-MAKER system.
Your role is to analyze Product Requirements Documents (PRDs) and ask clarifying questions to resolve ambiguity.

When analyzing a PRD:
1. Identify the core features and requirements
2. Detect any ambiguous or unclear requirements
3. Ask ONE focused clarifying question to start the conversation
4. Be conversational and helpful

Format your response as a friendly message that:
- Acknowledges you've received and analyzed the PRD
- Summarizes the key features you detected (2-3 bullet points)
- Asks ONE specific clarifying question

Do NOT output JSON. Output natural language."#;

            let user_prompt = format!("I've uploaded a PRD file named '{}'. Here is the content:\n\n{}", filename, content);

            let messages = vec![
                llm::Message::system(system_prompt),
                llm::Message::user(&user_prompt),
            ];

            match provider.complete(messages).await {
                Ok(response) => {
                    Ok(serde_json::json!({
                        "status": "analyzing",
                        "filename": filename,
                        "initial_message": response.content,
                        "detected_features": []
                    }))
                }
                Err(e) => {
                    // Fall back to mock on error
                    Ok(serde_json::json!({
                        "status": "analyzing",
                        "filename": filename,
                        "initial_message": format!(
                            "I've analyzed your PRD \"{}\". Let me ask a few clarifying questions to ensure I understand your requirements correctly.\n\n**Question 1:** What is the primary target platform for this application? (Desktop, Web, Mobile, or Cross-platform)\n\n_(Note: LLM error: {})_",
                            filename, e
                        ),
                        "detected_features": []
                    }))
                }
            }
        }
        Err(_) => {
            // No LLM configured, use mock
            Ok(serde_json::json!({
                "status": "analyzing",
                "filename": filename,
                "initial_message": format!(
                    "I've analyzed your PRD \"{}\". Let me ask a few clarifying questions to ensure I understand your requirements correctly.\n\n**Question 1:** What is the primary target platform for this application? (Desktop, Web, Mobile, or Cross-platform)\n\n_(Configure API keys in Settings for AI-powered analysis)_",
                    filename
                ),
                "detected_features": [
                    "Authentication system detected",
                    "Database integration required",
                    "API endpoints needed"
                ]
            }))
        }
    }
}

/// Send a message to L1 and get a response
#[tauri::command]
async fn send_interrogation_message(message: String, context: serde_json::Value) -> Result<serde_json::Value, String> {
    match get_llm_provider() {
        Ok(provider) => {
            let prd_content = context.get("prd")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let system_prompt = format!(r#"You are the L1 Product Orchestrator in the Cerebras-MAKER system.
You are having a conversation with a user to clarify their PRD requirements.

Original PRD content:
{}

Your role:
1. Answer any questions the user has
2. Ask follow-up clarifying questions if needed
3. When you have enough information, say "I have all the information I need to proceed" and set is_final to true

Keep responses concise and focused. Ask ONE question at a time."#, prd_content);

            let messages = vec![
                llm::Message::system(&system_prompt),
                llm::Message::user(&message),
            ];

            match provider.complete(messages).await {
                Ok(response) => {
                    let is_final = response.content.to_lowercase().contains("have all the information")
                        || response.content.to_lowercase().contains("ready to proceed")
                        || response.content.to_lowercase().contains("let's begin");

                    Ok(serde_json::json!({
                        "role": "assistant",
                        "content": response.content,
                        "is_final": is_final
                    }))
                }
                Err(e) => {
                    Ok(serde_json::json!({
                        "role": "assistant",
                        "content": format!("I understand. Let me note that down.\n\n_(LLM error: {})_", e),
                        "is_final": false
                    }))
                }
            }
        }
        Err(_) => {
            // Mock response
            Ok(serde_json::json!({
                "role": "assistant",
                "content": "Thank you for that clarification. Based on your response, I have a follow-up question...\n\n**Question 2:** What authentication method would you prefer? (OAuth, JWT, Session-based, or None)\n\n_(Configure API keys in Settings for AI-powered responses)_",
                "is_final": false
            }))
        }
    }
}

/// Complete interrogation and generate PLAN.md
#[tauri::command]
async fn complete_interrogation(conversation: Vec<serde_json::Value>) -> Result<serde_json::Value, String> {
    // TODO: Generate actual PLAN.md from conversation using L1
    let conversation_text: Vec<String> = conversation.iter()
        .filter_map(|msg| {
            let role = msg.get("role")?.as_str()?;
            let content = msg.get("content")?.as_str()?;
            Some(format!("{}: {}", role, content))
        })
        .collect();

    Ok(serde_json::json!({
        "status": "complete",
        "plan_md": format!("# Project Plan\n\n## Overview\n\nGenerated from {} messages.\n\n## Conversation Summary\n\n{}",
            conversation.len(),
            conversation_text.join("\n\n")
        ),
        "tasks": []
    }))
}

// ============================================================================
// Template Commands
// ============================================================================

/// List available project templates
#[tauri::command]
fn list_templates() -> Result<serde_json::Value, String> {
    let registry = templates::TemplateRegistry::new();
    let templates: Vec<serde_json::Value> = registry.list()
        .iter()
        .map(|t| serde_json::json!({
            "id": t.id,
            "name": t.name,
            "description": t.description,
            "tech_stack": t.tech_stack,
        }))
        .collect();

    Ok(serde_json::json!(templates))
}

/// Create a new project from a template
#[tauri::command]
fn create_from_template(template_id: String, project_path: String, project_name: String) -> Result<String, String> {
    let registry = templates::TemplateRegistry::new();
    let path = std::path::Path::new(&project_path);

    registry.create_project(&template_id, path, &project_name)?;

    Ok(format!("Project '{}' created successfully at {}", project_name, project_path))
}

// ============================================================================
// L2 Technical Orchestrator Commands
// ============================================================================

/// Process a PLAN.md and generate a Rhai execution script
#[tauri::command]
async fn generate_execution_script(plan_content: String, workspace_path: String) -> Result<serde_json::Value, String> {
    let orchestrator = agents::Orchestrator::new();

    let context = agents::AgentContext::new(&workspace_path);

    let script = orchestrator.process_plan(&plan_content, &context).await?;

    // Also parse the plan to return task info
    let plan = orchestrator.parse_plan_md(&plan_content)?;

    Ok(serde_json::json!({
        "script": script,
        "plan_id": plan.plan_id,
        "task_count": plan.micro_tasks.len(),
        "tasks": plan.micro_tasks.iter().map(|t| serde_json::json!({
            "id": t.id,
            "description": t.description,
            "atom_type": t.atom_type
        })).collect::<Vec<_>>()
    }))
}

/// Parse a PLAN.md without generating scripts (for preview)
#[tauri::command]
fn parse_plan(plan_content: String) -> Result<serde_json::Value, String> {
    let orchestrator = agents::Orchestrator::new();
    let plan = orchestrator.parse_plan_md(&plan_content)?;

    Ok(serde_json::json!({
        "plan_id": plan.plan_id,
        "title": plan.title,
        "task_count": plan.micro_tasks.len(),
        "tasks": plan.micro_tasks.iter().map(|t| serde_json::json!({
            "id": t.id,
            "description": t.description,
            "atom_type": t.atom_type,
            "estimated_complexity": t.estimated_complexity,
            "seed_symbols": t.seed_symbols
        })).collect::<Vec<_>>(),
        "dependencies": plan.dependencies
    }))
}

// ============================================================================
// L3 Context Engineer Commands
// ============================================================================

/// Extract context for a micro-task using the L3 Context Engineer
/// PRD 3.1: "Semantic Tree-Shaking" - Extract only topologically-relevant code
#[tauri::command]
fn extract_task_context(
    task_id: String,
    task_description: String,
    atom_type: String,
    seed_symbols: Vec<String>,
    workspace_path: String,
) -> Result<serde_json::Value, String> {
    use agents::context_engineer::{ContextConfig, ContextEngineer};
    use agents::MicroTask;
    use std::path::Path;

    // Create the micro-task
    let task = MicroTask {
        id: task_id,
        description: task_description,
        atom_type,
        estimated_complexity: 3,
        seed_symbols,
    };

    // Create the context engineer with default config
    let engineer = ContextEngineer::new();

    // Extract context
    let context_package = engineer.extract_context(&task, Path::new(&workspace_path))?;

    // Return as JSON
    serde_json::to_value(&context_package).map_err(|e| e.to_string())
}

/// Extract context using a cached symbol graph (more efficient for batch operations)
#[tauri::command]
fn extract_task_context_cached(
    task_id: String,
    task_description: String,
    atom_type: String,
    seed_symbols: Vec<String>,
    workspace_path: String,
) -> Result<serde_json::Value, String> {
    use agents::context_engineer::ContextEngineer;
    use agents::MicroTask;
    use std::path::Path;

    // Get cached graph or return error
    let graph = grits::get_cached_graph()
        .ok_or("No cached graph. Call load_symbol_graph first.")?;

    // Create the micro-task
    let task = MicroTask {
        id: task_id,
        description: task_description,
        atom_type,
        estimated_complexity: 3,
        seed_symbols,
    };

    // Create the context engineer
    let engineer = ContextEngineer::new();

    // Extract context using cached graph
    let context_package = engineer.extract_context_with_graph(
        &task,
        &graph,
        Path::new(&workspace_path),
    )?;

    // Return as JSON
    serde_json::to_value(&context_package).map_err(|e| e.to_string())
}

/// Get the rendered markdown context for a task (for LLM consumption)
#[tauri::command]
fn get_task_context_markdown(
    task_id: String,
    task_description: String,
    atom_type: String,
    seed_symbols: Vec<String>,
    workspace_path: String,
) -> Result<String, String> {
    use agents::context_engineer::ContextEngineer;
    use agents::MicroTask;
    use std::path::Path;

    // Get cached graph or load fresh
    let graph = match grits::get_cached_graph() {
        Some(g) => g,
        None => grits::load_workspace_graph(&workspace_path)?,
    };

    // Create the micro-task
    let task = MicroTask {
        id: task_id,
        description: task_description,
        atom_type,
        estimated_complexity: 3,
        seed_symbols,
    };

    // Create the context engineer
    let engineer = ContextEngineer::new();

    // Extract context
    let context_package = engineer.extract_context_with_graph(
        &task,
        &graph,
        Path::new(&workspace_path),
    )?;

    // Return just the markdown
    Ok(context_package.markdown)
}

// ============================================================================
// L4 Atom Execution Commands
// PRD Section 4.2: "The Atom" - Ephemeral, stateless agents with exactly one tool
// ============================================================================

/// Execute a single atom with context
/// This is the core L4 execution - takes a task and optional context, returns atom result
#[tauri::command]
async fn execute_atom(
    atom_type: String,
    task: String,
    context_package: Option<serde_json::Value>,
    require_json: bool,
    temperature: f32,
) -> Result<serde_json::Value, String> {
    use agents::atom_executor::{AtomExecutor, AtomInput};
    use agents::context_engineer::ContextPackage;
    use maker_core::SpawnFlags;

    // Parse atom type
    let atom_type = AtomType::from_str(&atom_type)
        .ok_or_else(|| format!("Unknown atom type: {}", atom_type))?;

    // Build spawn flags
    let flags = SpawnFlags {
        require_json,
        temperature,
        max_tokens: Some(atom_type.max_tokens()),
        red_flag_check: true,
    };

    // Build atom input
    let mut input = AtomInput::new(atom_type, &task).with_flags(flags);

    // Deserialize context package if provided
    if let Some(ctx_json) = context_package {
        let context: ContextPackage = serde_json::from_value(ctx_json)
            .map_err(|e| format!("Invalid context package: {}", e))?;
        input = input.with_context(context);
    }

    // Get LLM config
    let llm_config = get_llm_config_from_settings()?;

    // Create executor and run
    let executor = AtomExecutor::new(llm_config);
    let result = executor.execute(input).await?;

    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// Execute an atom with full context extraction (L3 + L4 pipeline)
/// Combines context engineering and atom execution in one call
#[tauri::command]
async fn execute_atom_with_context(
    atom_type: String,
    task_id: String,
    task_description: String,
    seed_symbols: Vec<String>,
    workspace_path: String,
    require_json: bool,
) -> Result<serde_json::Value, String> {
    use agents::atom_executor::{AtomExecutor, AtomInput};
    use agents::context_engineer::ContextEngineer;
    use agents::MicroTask;
    use maker_core::SpawnFlags;
    use std::path::Path;

    // Parse atom type
    let parsed_atom_type = AtomType::from_str(&atom_type)
        .ok_or_else(|| format!("Unknown atom type: {}", atom_type))?;

    // Build the micro-task for L3
    let task = MicroTask {
        id: task_id.clone(),
        description: task_description.clone(),
        atom_type: atom_type.clone(),
        estimated_complexity: 3,
        seed_symbols,
    };

    // L3: Extract context
    let engineer = ContextEngineer::new();
    let context_package = engineer.extract_context(&task, Path::new(&workspace_path))?;

    // Build spawn flags
    let flags = SpawnFlags {
        require_json,
        temperature: 0.1,
        max_tokens: Some(parsed_atom_type.max_tokens()),
        red_flag_check: true,
    };

    // L4: Execute atom with context
    let input = AtomInput::new(parsed_atom_type, &task_description)
        .with_context(context_package)
        .with_flags(flags);

    let llm_config = get_llm_config_from_settings()?;
    let executor = AtomExecutor::new(llm_config);
    let result = executor.execute(input).await?;

    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// Parse code output from a Coder atom into structured changes
#[tauri::command]
fn parse_coder_output(raw_output: String) -> Result<serde_json::Value, String> {
    use agents::atom_executor::AtomExecutor;

    let executor = AtomExecutor::new(LlmConfig::default());
    let changes = executor.parse_code_output(&raw_output);

    serde_json::to_value(&changes).map_err(|e| e.to_string())
}

/// Parse review output from a Reviewer atom
#[tauri::command]
fn parse_reviewer_output(raw_output: String) -> Result<serde_json::Value, String> {
    use agents::atom_executor::AtomExecutor;

    let executor = AtomExecutor::new(LlmConfig::default());
    let result = executor.parse_review_output(&raw_output)?;

    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// Get available atom types
#[tauri::command]
fn get_atom_types() -> serde_json::Value {
    serde_json::json!([
        {"id": "Search", "name": "Search", "description": "Find relevant code", "max_tokens": 500},
        {"id": "Coder", "name": "Coder", "description": "Write code", "max_tokens": 2000},
        {"id": "Reviewer", "name": "Reviewer", "description": "Review code", "max_tokens": 750},
        {"id": "Planner", "name": "Planner", "description": "Break down tasks", "max_tokens": 1000},
        {"id": "Validator", "name": "Validator", "description": "Validate requirements", "max_tokens": 500},
        {"id": "Tester", "name": "Tester", "description": "Write tests", "max_tokens": 2000},
        {"id": "Architect", "name": "Architect", "description": "Design interfaces", "max_tokens": 1500},
        {"id": "GritsAnalyzer", "name": "GritsAnalyzer", "description": "Analyze topology", "max_tokens": 1000}
    ])
}

/// Helper to get LLM config from saved settings
fn get_llm_config_from_settings() -> Result<LlmConfig, String> {
    let path = get_settings_path();
    if path.exists() {
        let json = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read settings: {}", e))?;
        let settings: AppSettings = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse settings: {}", e))?;

        // Use the first configured provider
        if !settings.api_keys.openai.is_empty() {
            return Ok(LlmConfig {
                api_key: Some(settings.api_keys.openai.clone()),
                model: "gpt-4o".to_string(),
                ..Default::default()
            });
        }
        if !settings.api_keys.anthropic.is_empty() {
            return Ok(LlmConfig {
                api_key: Some(settings.api_keys.anthropic.clone()),
                ..LlmConfig::anthropic()
            });
        }
        if !settings.api_keys.cerebras.is_empty() {
            return Ok(LlmConfig {
                api_key: Some(settings.api_keys.cerebras.clone()),
                ..LlmConfig::cerebras()
            });
        }
    }

    // Fall back to environment variables
    if std::env::var("OPENAI_API_KEY").is_ok() {
        return Ok(LlmConfig::default());
    }
    if std::env::var("ANTHROPIC_API_KEY").is_ok() {
        return Ok(LlmConfig::anthropic());
    }
    if std::env::var("CEREBRAS_API_KEY").is_ok() {
        return Ok(LlmConfig::cerebras());
    }

    Err("No LLM provider configured. Please set an API key in Settings.".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Core plugins
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        // File system and shell
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        // Storage and persistence
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        // Notifications and clipboard
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        // Global shortcuts
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        // Logging
        .plugin(tauri_plugin_log::Builder::new().build())
        // Single instance (prevents duplicate processes)
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {
            // Focus the existing window when a second instance is launched
        }))
        // CLI argument parsing
        .plugin(tauri_plugin_cli::init())
        // Auto-updater
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            greet,
            // Grits commands
            load_symbol_graph,
            assemble_mini_codebase,
            check_red_flags,
            analyze_topology,
            // MAKER runtime commands
            init_runtime,
            execute_script,
            get_execution_log,
            get_cwd,
            // Shadow Git commands
            create_snapshot,
            rollback_snapshot,
            rollback_to_snapshot,
            squash_snapshots,
            get_snapshots,
            checkout_commit,
            get_git_history,
            // Settings commands
            save_settings,
            load_settings,
            // L1 Interrogation commands
            analyze_prd,
            send_interrogation_message,
            complete_interrogation,
            // Template commands
            list_templates,
            create_from_template,
            // L2 Orchestrator commands
            generate_execution_script,
            parse_plan,
            // L3 Context Engineer commands
            extract_task_context,
            extract_task_context_cached,
            get_task_context_markdown,
            // L4 Atom Execution commands
            execute_atom,
            execute_atom_with_context,
            parse_coder_output,
            parse_reviewer_output,
            get_atom_types
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
