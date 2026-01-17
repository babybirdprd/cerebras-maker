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

// Knowledge Base module - pre-existing research and documentation
pub mod knowledge_base;

// Re-export maker_core types for convenience
pub use maker_core::{AtomType, AtomResult, CodeModeRuntime, ShadowGit, ConsensusConfig, ConsensusResult};

// Re-export agent types
pub use agents::{Agent, Interrogator, Architect, Orchestrator, AgentContext, AgentOutput, AtomExecutor, AtomInput, AtomOutput, CodeChange, ReviewResult, ValidationResult};

// Re-export LLM types
pub use llm::{LlmProvider, LlmConfig, Message, Role};

// Re-export generator types
pub use generators::{ScriptGenerator, GeneratorRegistry, GenerationResult, RhaiScriptGenerator, TaskScriptGenerator};

// Validation helpers for Tauri commands
mod validation {
    use std::path::Path;

    /// Standardized error response helper
    pub fn make_error(operation: &str, details: &str) -> String {
        format!("[{}] {}", operation, details)
    }

    /// Standardized error with code
    pub fn make_error_with_code(operation: &str, code: &str, details: &str) -> String {
        format!("[{}:{}] {}", operation, code, details)
    }

    pub fn validate_workspace_path(path: &str) -> Result<(), String> {
        if path.is_empty() {
            return Err("Workspace path cannot be empty".to_string());
        }
        if !Path::new(path).exists() {
            return Err(format!("Workspace path does not exist: {}", path));
        }
        Ok(())
    }

    pub fn validate_non_empty(value: &str, field_name: &str) -> Result<(), String> {
        if value.trim().is_empty() {
            return Err(format!("{} cannot be empty", field_name));
        }
        Ok(())
    }

    pub fn validate_file_path(path: &str) -> Result<(), String> {
        if path.is_empty() {
            return Err("File path cannot be empty".to_string());
        }
        // Check for path traversal attempts
        if path.contains("..") {
            return Err("Path traversal not allowed".to_string());
        }
        Ok(())
    }
}

// Global runtime instance (lazy initialized)
static RUNTIME: Mutex<Option<CodeModeRuntime>> = Mutex::new(None);

// Global ShadowGit instance for transactional file operations
static SHADOW_GIT: Mutex<Option<ShadowGit>> = Mutex::new(None);

// Global Knowledge Base instance
static KNOWLEDGE_BASE: Mutex<Option<knowledge_base::KnowledgeBase>> = Mutex::new(None);

// ============================================================================
// Execution Metrics & Voting State
// ============================================================================

/// Execution metrics for the swarm dashboard
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct ExecutionMetrics {
    /// Number of active atoms currently executing
    pub active_atoms: usize,
    /// Total atoms spawned in current session
    pub total_atoms_spawned: usize,
    /// Total tokens processed in current session
    pub total_tokens: u64,
    /// Tokens processed per second (rolling average)
    pub tokens_per_second: f64,
    /// Number of red flags detected
    pub red_flag_count: usize,
    /// Number of shadow commits (snapshots)
    pub shadow_commits: usize,
    /// Timestamp of last update
    pub last_updated_ms: u64,
}

/// A voting candidate for display
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VotingCandidate {
    pub id: usize,
    pub snippet: String,
    pub score: f64,
    pub red_flags: Vec<String>,
    pub status: String, // "pending", "accepted", "rejected"
    pub votes: usize,
}

/// Current voting state
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct VotingState {
    pub task_id: String,
    pub task_description: String,
    pub candidates: Vec<VotingCandidate>,
    pub is_voting: bool,
    pub winner_id: Option<usize>,
}

// Global execution metrics state
static EXECUTION_METRICS: Mutex<ExecutionMetrics> = Mutex::new(ExecutionMetrics {
    active_atoms: 0,
    total_atoms_spawned: 0,
    total_tokens: 0,
    tokens_per_second: 0.0,
    red_flag_count: 0,
    shadow_commits: 0,
    last_updated_ms: 0,
});

// Global voting state
static VOTING_STATE: Mutex<VotingState> = Mutex::new(VotingState {
    task_id: String::new(),
    task_description: String::new(),
    candidates: Vec::new(),
    is_voting: false,
    winner_id: None,
});

/// Module for grits-core integration functionality
pub mod grits {
    use super::*;
    use grits_core::topology::analysis::{InvariantResult, LayerViolation};
    use grits_core::topology::layers::load_layer_config;
    use grits_core::topology::scanner::DirectoryScanner;
    use std::sync::Mutex;

    /// Cached SymbolGraph for the workspace (thread-safe)
    static WORKSPACE_GRAPH: Mutex<Option<SymbolGraph>> = Mutex::new(None);

    /// Cached workspace path for layer config loading
    static WORKSPACE_PATH: Mutex<Option<String>> = Mutex::new(None);

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

        // Cache the graph and workspace path
        if let Ok(mut cached) = WORKSPACE_GRAPH.lock() {
            *cached = Some(graph.clone());
        }
        if let Ok(mut cached_path) = WORKSPACE_PATH.lock() {
            *cached_path = Some(workspace_path.to_string());
        }

        Ok(graph)
    }

    /// Get the cached workspace graph
    pub fn get_cached_graph() -> Option<SymbolGraph> {
        WORKSPACE_GRAPH.lock().ok().and_then(|g| g.clone())
    }

    /// Get the cached workspace path
    pub fn get_cached_workspace_path() -> Option<String> {
        WORKSPACE_PATH.lock().ok().and_then(|p| p.clone())
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
    /// PRD 3.2: "Architectural Red-Flagging" - Detect cycle membership via triangles and layer violations
    pub fn red_flag_check(graph: &SymbolGraph, previous_betti_1: usize, workspace_path: Option<&str>) -> RedFlagResult {
        let analysis = TopologicalAnalysis::analyze(graph);

        // Load layer config and check for layer violations
        let (layer_violations, layer_config_loaded) = if let Some(ws_path) = workspace_path {
            match load_layer_config(Path::new(ws_path)) {
                Ok(config) => {
                    let invariant_result = InvariantResult::check(graph, &config);
                    (invariant_result.layer_violations, true)
                }
                Err(e) => {
                    eprintln!("Warning: Failed to load layer config: {}", e);
                    (Vec::new(), false)
                }
            }
        } else {
            (Vec::new(), false)
        };

        RedFlagResult {
            introduced_cycle: analysis.betti_1 > previous_betti_1,
            has_layer_violations: !layer_violations.is_empty(),
            betti_1: analysis.betti_1,
            betti_0: analysis.betti_0,
            triangle_count: analysis.triangle_count,
            solid_score: analysis.solid_score().normalized,
            cycles_detected: analysis
                .triangles
                .iter()
                .map(|t| t.nodes.to_vec())
                .collect(),
            layer_violations,
            layer_config_loaded,
        }
    }

    /// Check if proposed changes would introduce architectural violations (virtual apply)
    /// PRD 3.2: Pre-check changes before applying them
    pub fn virtual_red_flag_check(
        graph: &SymbolGraph,
        proposed_changes: &[ProposedChange],
        workspace_path: Option<&str>,
    ) -> RedFlagResult {
        // Get current Betti_1 before changes
        let current_analysis = TopologicalAnalysis::analyze(graph);
        let previous_betti_1 = current_analysis.betti_1;

        // Create a temporary graph with proposed changes applied
        let mut temp_graph = graph.clone();
        for change in proposed_changes {
            // Add new symbols from proposed changes
            if let Some(ref symbol) = change.new_symbol {
                temp_graph.add_symbol(symbol.clone());
            }
            // Add new dependencies
            for (from, to, relation) in &change.new_dependencies {
                temp_graph.add_dependency(from, to, relation);
            }
        }

        // Check the temporary graph
        red_flag_check(&temp_graph, previous_betti_1, workspace_path)
    }

    /// A proposed change to be virtually applied for red-flag checking
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct ProposedChange {
        pub file_path: String,
        pub new_symbol: Option<Symbol>,
        pub new_dependencies: Vec<(String, String, String)>, // (from, to, relation)
    }

    /// Result of architectural red-flag check
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct RedFlagResult {
        pub introduced_cycle: bool,
        pub has_layer_violations: bool,
        pub betti_1: usize,
        pub betti_0: usize,
        pub triangle_count: usize,
        pub solid_score: f32,
        pub cycles_detected: Vec<Vec<String>>,
        pub layer_violations: Vec<LayerViolation>,
        pub layer_config_loaded: bool,
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
    use validation::{make_error, make_error_with_code};

    let graph = grits::load_workspace_graph(&workspace_path)
        .map_err(|e| make_error("WORKSPACE_ANALYSIS", &e))?;
    serde_json::to_value(&graph)
        .map_err(|e| make_error_with_code("WORKSPACE_ANALYSIS", "SERIALIZE", &e.to_string()))
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
    let workspace_path = grits::get_cached_workspace_path();
    Ok(grits::red_flag_check(&graph, previous_betti_1, workspace_path.as_deref()))
}

/// Check if proposed changes would introduce red flags (virtual apply)
#[tauri::command]
fn check_proposed_changes(
    proposed_changes: Vec<grits::ProposedChange>,
    _previous_betti_1: usize,
) -> Result<grits::RedFlagResult, String> {
    let graph = grits::get_cached_graph().ok_or("No cached graph. Call load_symbol_graph first.")?;
    let workspace_path = grits::get_cached_workspace_path();
    Ok(grits::virtual_red_flag_check(&graph, &proposed_changes, workspace_path.as_deref()))
}

/// Analyze topology and return full analysis
#[tauri::command]
fn analyze_topology() -> Result<serde_json::Value, String> {
    use validation::make_error_with_code;

    let graph = grits::get_cached_graph()
        .ok_or_else(|| make_error_with_code("TOPOLOGY_ANALYSIS", "NO_GRAPH", "No cached graph. Call load_symbol_graph first."))?;
    let analysis = TopologicalAnalysis::analyze(&graph);
    serde_json::to_value(&analysis)
        .map_err(|e| make_error_with_code("TOPOLOGY_ANALYSIS", "SERIALIZE", &e.to_string()))
}

// ============================================================================
// Multi-file Edit Validation Commands (PRD Section 3.2)
// ============================================================================

/// Input for multi-file edit validation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MultiFileEdit {
    pub file_path: String,
    pub operation: String, // "create", "modify", "delete"
    pub content: Option<String>,
    pub language: Option<String>,
}

/// Result of multi-file validation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MultiFileValidationResult {
    pub is_safe: bool,
    pub original_betti_1: usize,
    pub new_betti_1: usize,
    pub introduces_cycles: bool,
    pub layer_violations: Vec<serde_json::Value>,
    pub new_symbols: Vec<String>,
    pub new_dependencies: Vec<(String, String, String)>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub files_analyzed: usize,
    pub cross_file_issues: Vec<String>,
}

/// Validate multiple file edits together for architectural violations
/// Uses Grits VirtualApply to check proposed changes before applying
#[tauri::command]
fn validate_multi_file_edit(
    workspace_path: String,
    edits: Vec<MultiFileEdit>,
) -> Result<MultiFileValidationResult, String> {
    use grits_core::topology::virtual_apply::{ChangeType, ProposedChange, VirtualApply};
    use grits_core::topology::layers::load_layer_config;
    use std::path::Path;
    use validation::make_error_with_code;

    // Ensure we have a symbol graph loaded
    let graph = grits::get_cached_graph()
        .ok_or_else(|| make_error_with_code("MULTI_FILE_EDIT", "NO_GRAPH", "No cached graph. Call load_symbol_graph first."))?;

    // Load layer config if available
    let layer_config = load_layer_config(Path::new(&workspace_path)).ok();

    // Convert edits to ProposedChanges
    let proposed_changes: Vec<ProposedChange> = edits.iter().map(|edit| {
        let change_type = match edit.operation.as_str() {
            "create" => ChangeType::CreateFile,
            "delete" => ChangeType::DeleteFile,
            _ => ChangeType::ModifyFile,
        };

        let language = edit.language.clone().unwrap_or_else(|| {
            // Detect from extension
            if edit.file_path.ends_with(".rs") { "rust".to_string() }
            else if edit.file_path.ends_with(".ts") || edit.file_path.ends_with(".tsx") { "typescript".to_string() }
            else if edit.file_path.ends_with(".js") || edit.file_path.ends_with(".jsx") { "javascript".to_string() }
            else if edit.file_path.ends_with(".py") { "python".to_string() }
            else if edit.file_path.ends_with(".go") { "go".to_string() }
            else { "unknown".to_string() }
        });

        ProposedChange {
            file_path: edit.file_path.clone(),
            change_type,
            code_content: edit.content.clone().unwrap_or_default(),
            language,
        }
    }).collect();

    // Run VirtualApply validation
    let virtual_apply = VirtualApply::new(graph.clone(), layer_config);
    let result = virtual_apply.validate(&proposed_changes);

    // Check for cross-file dependency issues
    let mut cross_file_issues = Vec::new();

    // Collect symbols by file
    let mut symbols_by_file: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    for (from, to, _) in &result.new_dependencies {
        let from_file = from.split("::").next().unwrap_or(from);
        symbols_by_file.entry(from_file.to_string()).or_default().push(to.clone());
    }

    // Check if new dependencies cross files in problematic ways
    for (file, deps) in &symbols_by_file {
        for dep in deps {
            let dep_file = dep.split("::").next().unwrap_or(dep);
            if file != dep_file && !dep_file.starts_with("std") && !dep_file.starts_with("core") {
                // Check if the dependency target exists
                if !graph.nodes.contains_key(dep) {
                    cross_file_issues.push(format!(
                        "New dependency from {} to {} (target may not exist)",
                        file, dep
                    ));
                }
            }
        }
    }

    // Convert layer violations to JSON
    let layer_violations: Vec<serde_json::Value> = result.layer_violations.iter().map(|v| {
        serde_json::json!({
            "from_symbol": v.from_symbol,
            "from_layer": v.from_layer,
            "to_symbol": v.to_symbol,
            "to_layer": v.to_layer,
            "message": v.message
        })
    }).collect();

    Ok(MultiFileValidationResult {
        is_safe: result.is_safe && cross_file_issues.is_empty(),
        original_betti_1: result.original_betti_1,
        new_betti_1: result.new_betti_1,
        introduces_cycles: result.introduces_cycles,
        layer_violations,
        new_symbols: result.new_symbols,
        new_dependencies: result.new_dependencies,
        warnings: result.warnings,
        errors: result.errors,
        files_analyzed: edits.len(),
        cross_file_issues,
    })
}

/// Get a preview of what symbols and dependencies would be created by edits
#[tauri::command]
fn preview_edit_impact(
    edits: Vec<MultiFileEdit>,
) -> Result<serde_json::Value, String> {
    use grits_core::topology::virtual_apply::{ChangeType, ProposedChange, VirtualApply};
    use grits_core::topology::SymbolGraph;

    // Use empty graph just to extract symbols
    let graph = SymbolGraph::new();
    let virtual_apply = VirtualApply::new(graph, None);

    let proposed_changes: Vec<ProposedChange> = edits.iter().map(|edit| {
        let change_type = match edit.operation.as_str() {
            "create" => ChangeType::CreateFile,
            "delete" => ChangeType::DeleteFile,
            _ => ChangeType::ModifyFile,
        };

        let language = edit.language.clone().unwrap_or_else(|| {
            if edit.file_path.ends_with(".rs") { "rust".to_string() }
            else if edit.file_path.ends_with(".ts") || edit.file_path.ends_with(".tsx") { "typescript".to_string() }
            else if edit.file_path.ends_with(".js") || edit.file_path.ends_with(".jsx") { "javascript".to_string() }
            else if edit.file_path.ends_with(".py") { "python".to_string() }
            else if edit.file_path.ends_with(".go") { "go".to_string() }
            else { "unknown".to_string() }
        });

        ProposedChange {
            file_path: edit.file_path.clone(),
            change_type,
            code_content: edit.content.clone().unwrap_or_default(),
            language,
        }
    }).collect();

    let result = virtual_apply.validate(&proposed_changes);

    Ok(serde_json::json!({
        "new_symbols": result.new_symbols,
        "new_dependencies": result.new_dependencies,
        "files_affected": edits.len(),
    }))
}

// ============================================================================
// Test Generation & Execution Commands (PRD Phase 3)
// ============================================================================

/// Test framework detection result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestFrameworkInfo {
    pub framework: String,
    pub test_command: String,
    pub test_pattern: String,
    pub config_file: Option<String>,
}

/// Detect test framework in workspace
#[tauri::command]
async fn detect_test_framework(workspace_path: String) -> Result<TestFrameworkInfo, String> {
    use std::path::Path;

    let ws = Path::new(&workspace_path);

    // Check for Rust (Cargo.toml)
    if ws.join("Cargo.toml").exists() {
        return Ok(TestFrameworkInfo {
            framework: "rust-cargo".to_string(),
            test_command: "cargo test".to_string(),
            test_pattern: "#[test]".to_string(),
            config_file: Some("Cargo.toml".to_string()),
        });
    }

    // Check for Node.js / TypeScript
    if ws.join("package.json").exists() {
        let pkg_content = std::fs::read_to_string(ws.join("package.json"))
            .map_err(|e| e.to_string())?;

        // Check for vitest
        if pkg_content.contains("vitest") {
            return Ok(TestFrameworkInfo {
                framework: "vitest".to_string(),
                test_command: "npm run test".to_string(),
                test_pattern: "*.test.ts".to_string(),
                config_file: ws.join("vitest.config.ts").exists().then(|| "vitest.config.ts".to_string()),
            });
        }

        // Check for jest
        if pkg_content.contains("jest") {
            return Ok(TestFrameworkInfo {
                framework: "jest".to_string(),
                test_command: "npm test".to_string(),
                test_pattern: "*.test.ts".to_string(),
                config_file: ws.join("jest.config.js").exists().then(|| "jest.config.js".to_string()),
            });
        }

        // Default Node.js
        return Ok(TestFrameworkInfo {
            framework: "node".to_string(),
            test_command: "npm test".to_string(),
            test_pattern: "*.test.js".to_string(),
            config_file: Some("package.json".to_string()),
        });
    }

    // Check for Python
    if ws.join("pyproject.toml").exists() || ws.join("setup.py").exists() {
        let is_pytest = ws.join("pytest.ini").exists() ||
            ws.join("pyproject.toml").exists() &&
            std::fs::read_to_string(ws.join("pyproject.toml"))
                .unwrap_or_default()
                .contains("pytest");

        return Ok(TestFrameworkInfo {
            framework: if is_pytest { "pytest".to_string() } else { "unittest".to_string() },
            test_command: if is_pytest { "pytest".to_string() } else { "python -m unittest".to_string() },
            test_pattern: "test_*.py".to_string(),
            config_file: ws.join("pytest.ini").exists().then(|| "pytest.ini".to_string()),
        });
    }

    // Check for Go
    if ws.join("go.mod").exists() {
        return Ok(TestFrameworkInfo {
            framework: "go-test".to_string(),
            test_command: "go test ./...".to_string(),
            test_pattern: "*_test.go".to_string(),
            config_file: Some("go.mod".to_string()),
        });
    }

    Err("No test framework detected".to_string())
}

/// Test execution result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestExecutionResult {
    pub success: bool,
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub duration_ms: u64,
    pub output: String,
    pub failed_tests: Vec<FailedTest>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FailedTest {
    pub name: String,
    pub file: Option<String>,
    pub error: String,
}

/// Run tests in workspace
#[tauri::command]
async fn run_tests(
    workspace_path: String,
    test_pattern: Option<String>,
    timeout_seconds: Option<u64>,
) -> Result<TestExecutionResult, String> {
    use std::process::Command;
    use std::time::Instant;
    use validation::make_error;

    let framework = detect_test_framework(workspace_path.clone()).await
        .map_err(|e| make_error("TEST_EXEC", &e))?;
    let start = Instant::now();
    let _timeout = timeout_seconds.unwrap_or(300); // 5 minutes default (reserved for future timeout implementation)

    let mut cmd_parts: Vec<&str> = framework.test_command.split_whitespace().collect();
    let program = cmd_parts.remove(0);

    let mut cmd = Command::new(program);
    cmd.args(&cmd_parts);
    cmd.current_dir(&workspace_path);

    // Add test pattern filter if specified
    if let Some(pattern) = &test_pattern {
        match framework.framework.as_str() {
            "rust-cargo" => { cmd.arg("--").arg(pattern); }
            "vitest" | "jest" => { cmd.arg(pattern); }
            "pytest" => { cmd.arg("-k").arg(pattern); }
            "go-test" => { cmd.arg("-run").arg(pattern); }
            _ => {}
        }
    }

    let output = cmd.output()
        .map_err(|e| make_error("TEST_EXEC", &format!("Failed to run tests: {}", e)))?;
    let duration = start.elapsed().as_millis() as u64;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined_output = format!("{}\n{}", stdout, stderr);

    // Parse test results based on framework
    let (total, passed, failed, skipped, failed_tests) = parse_test_output(&framework.framework, &combined_output);

    Ok(TestExecutionResult {
        success: output.status.success(),
        total_tests: total,
        passed,
        failed,
        skipped,
        duration_ms: duration,
        output: combined_output,
        failed_tests,
    })
}

/// Parse test output to extract stats
fn parse_test_output(framework: &str, output: &str) -> (usize, usize, usize, usize, Vec<FailedTest>) {
    let mut total = 0;
    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;
    let mut failed_tests = Vec::new();

    match framework {
        "rust-cargo" => {
            // Parse: "test result: ok. 10 passed; 0 failed; 0 ignored"
            for line in output.lines() {
                if line.contains("test result:") {
                    if let Some(caps) = line.split(';').collect::<Vec<_>>().first() {
                        if let Some(num) = caps.split_whitespace().find(|s| s.parse::<usize>().is_ok()) {
                            passed = num.parse().unwrap_or(0);
                        }
                    }
                    // Extract failed count
                    if let Some(failed_part) = line.split(';').nth(1) {
                        if let Some(num) = failed_part.split_whitespace().find(|s| s.parse::<usize>().is_ok()) {
                            failed = num.parse().unwrap_or(0);
                        }
                    }
                    // Extract ignored count
                    if let Some(ignored_part) = line.split(';').nth(2) {
                        if let Some(num) = ignored_part.split_whitespace().find(|s| s.parse::<usize>().is_ok()) {
                            skipped = num.parse().unwrap_or(0);
                        }
                    }
                }
                // Capture failed test names
                if line.contains("FAILED") && line.contains("test ") {
                    let test_name = line.replace("test ", "").replace(" ... FAILED", "").trim().to_string();
                    failed_tests.push(FailedTest {
                        name: test_name,
                        file: None,
                        error: "Test failed".to_string(),
                    });
                }
            }
            total = passed + failed + skipped;
        }
        "vitest" | "jest" => {
            // Parse: "Tests: 1 passed, 1 total"
            for line in output.lines() {
                if line.contains("Tests:") || line.contains("Test Suites:") {
                    for part in line.split(',') {
                        let words: Vec<&str> = part.split_whitespace().collect();
                        if words.len() >= 2 {
                            let num: usize = words[0].parse().unwrap_or(0);
                            let kind = words[1].to_lowercase();
                            if kind.contains("passed") { passed += num; }
                            else if kind.contains("failed") { failed += num; }
                            else if kind.contains("skipped") { skipped += num; }
                            else if kind.contains("total") { total = num; }
                        }
                    }
                }
            }
            if total == 0 { total = passed + failed + skipped; }
        }
        "pytest" => {
            // Parse: "5 passed, 1 failed, 2 skipped"
            for line in output.lines() {
                if line.contains(" passed") || line.contains(" failed") || line.contains(" skipped") {
                    for part in line.split(',') {
                        let words: Vec<&str> = part.split_whitespace().collect();
                        if words.len() >= 2 {
                            let num: usize = words[0].parse().unwrap_or(0);
                            let kind = words[1].to_lowercase();
                            if kind.contains("passed") { passed = num; }
                            else if kind.contains("failed") { failed = num; }
                            else if kind.contains("skipped") { skipped = num; }
                        }
                    }
                }
            }
            total = passed + failed + skipped;
        }
        "go-test" => {
            // Parse: "ok/FAIL package (duration)"
            for line in output.lines() {
                if line.starts_with("ok") { passed += 1; }
                else if line.starts_with("FAIL") { failed += 1; }
                else if line.contains("--- SKIP") { skipped += 1; }
            }
            total = passed + failed + skipped;
        }
        _ => {
            // Generic: count lines with ok/fail/pass/error
            for line in output.lines() {
                let lower = line.to_lowercase();
                if lower.contains("pass") || lower.contains("ok") { passed += 1; }
                else if lower.contains("fail") || lower.contains("error") { failed += 1; }
            }
            total = passed + failed;
        }
    }

    (total, passed, failed, skipped, failed_tests)
}

/// Generate test code for a given source file
#[tauri::command]
async fn generate_tests(
    workspace_path: String,
    source_file: String,
    test_type: Option<String>, // "unit", "integration", "property"
) -> Result<serde_json::Value, String> {
    use crate::agents::atom_executor::{AtomExecutor, AtomInput};
    use crate::llm::LlmConfig;
    use crate::maker_core::SpawnFlags;

    // Read the source file
    let source_path = std::path::Path::new(&workspace_path).join(&source_file);
    let source_content = std::fs::read_to_string(&source_path)
        .map_err(|e| format!("Failed to read source file: {}", e))?;

    // Detect language from extension
    let language = if source_file.ends_with(".rs") { "rust" }
        else if source_file.ends_with(".ts") || source_file.ends_with(".tsx") { "typescript" }
        else if source_file.ends_with(".js") || source_file.ends_with(".jsx") { "javascript" }
        else if source_file.ends_with(".py") { "python" }
        else if source_file.ends_with(".go") { "go" }
        else { "unknown" };

    let test_type = test_type.unwrap_or_else(|| "unit".to_string());

    // Build the test generation prompt
    let prompt = format!(
        r#"Generate {} tests for the following {} code.

Source file: {}

```{}
{}
```

Requirements:
1. Write comprehensive test cases covering all public functions/methods
2. Include edge cases and error conditions
3. Use the standard testing framework for this language
4. Include meaningful test names that describe what is being tested
5. Add comments explaining complex test logic

Return ONLY the test code, no explanations."#,
        test_type, language, source_file, language, source_content
    );

    // Execute using Tester atom
    let input = AtomInput::new(AtomType::Tester, &prompt)
        .with_flags(SpawnFlags {
            require_json: false,
            temperature: 0.3,
            max_tokens: Some(4000),
            red_flag_check: false,
        });

    // Get LLM config
    let config = LlmConfig::cerebras();

    let executor = AtomExecutor::new(config);
    let result = executor.execute(input).await
        .map_err(|e| format!("Test generation failed: {}", e))?;

    // Determine test file path
    let test_file = match language {
        "rust" => {
            if source_file.contains("/src/") {
                source_file.replace("/src/", "/tests/").replace(".rs", "_test.rs")
            } else {
                source_file.replace(".rs", "_test.rs")
            }
        }
        "typescript" | "javascript" => {
            let ext = if source_file.ends_with(".tsx") { ".tsx" }
                else if source_file.ends_with(".ts") { ".ts" }
                else if source_file.ends_with(".jsx") { ".jsx" }
                else { ".js" };
            source_file.replace(ext, &format!(".test{}", ext))
        }
        "python" => {
            let file_name = std::path::Path::new(&source_file)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&source_file);
            format!("tests/test_{}", file_name)
        }
        "go" => source_file.replace(".go", "_test.go"),
        _ => format!("{}.test", source_file),
    };

    Ok(serde_json::json!({
        "test_code": result.output,
        "suggested_file": test_file,
        "source_file": source_file,
        "language": language,
        "test_type": test_type,
    }))
}

// ============================================================================
// MAKER Runtime Commands (PRD Section 4)
// ============================================================================

/// Initialize the MAKER runtime for a workspace
#[tauri::command]
fn init_runtime(workspace_path: String) -> Result<String, String> {
    use validation::make_error;

    validation::validate_workspace_path(&workspace_path)
        .map_err(|e| make_error("RUNTIME_INIT", &e))?;

    let runtime = CodeModeRuntime::new(&workspace_path)
        .map_err(|e| make_error("RUNTIME_INIT", &format!("Failed to create runtime: {}", e)))?;

    if let Ok(mut global) = RUNTIME.lock() {
        *global = Some(runtime);
    }

    // Also initialize the ShadowGit for the workspace
    let mut shadow_git = ShadowGit::new(&workspace_path);
    shadow_git.init()
        .map_err(|e| make_error("RUNTIME_INIT", &format!("Shadow git init failed: {}", e)))?;

    if let Ok(mut sg) = SHADOW_GIT.lock() {
        *sg = Some(shadow_git);
    }

    Ok("Runtime initialized".to_string())
}

/// Execute a Rhai script
#[tauri::command]
fn execute_script(script: String) -> Result<serde_json::Value, String> {
    use validation::{make_error, make_error_with_code};

    validation::validate_non_empty(&script, "Script")
        .map_err(|e| make_error("SCRIPT_EXEC", &e))?;

    let runtime = RUNTIME.lock()
        .map_err(|_| make_error_with_code("SCRIPT_EXEC", "LOCK", "Failed to acquire runtime lock"))?;

    let runtime = runtime.as_ref()
        .ok_or_else(|| make_error_with_code("SCRIPT_EXEC", "NOT_INIT", "Runtime not initialized. Call init_runtime first."))?;

    let result = runtime.execute_script(&script)
        .map_err(|e| make_error("SCRIPT_EXEC", &format!("Execution failed: {}", e)))?;

    serde_json::to_value(&result.to_string())
        .map_err(|e| make_error_with_code("SCRIPT_EXEC", "SERIALIZE", &e.to_string()))
}

/// Get the execution log (for Cockpit)
#[tauri::command]
fn get_execution_log() -> Result<serde_json::Value, String> {
    let runtime = RUNTIME.lock()
        .map_err(|_| "Failed to acquire runtime lock while getting execution log")?;

    let runtime = runtime.as_ref()
        .ok_or("Runtime not initialized")?;

    let log = runtime.get_execution_log();
    serde_json::to_value(&log).map_err(|e| e.to_string())
}

/// Get real-time execution metrics for the swarm dashboard
#[tauri::command]
fn get_execution_metrics() -> Result<ExecutionMetrics, String> {
    let metrics = EXECUTION_METRICS.lock()
        .map_err(|_| "Failed to acquire metrics lock while getting execution metrics")?;

    Ok(metrics.clone())
}

/// Update execution metrics (called internally when atoms execute)
#[tauri::command]
fn update_execution_metrics(
    active_atoms: Option<usize>,
    tokens_added: Option<u64>,
    red_flags_added: Option<usize>,
) -> Result<(), String> {
    let mut metrics = EXECUTION_METRICS.lock()
        .map_err(|_| "Failed to acquire metrics lock while updating execution metrics")?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    if let Some(active) = active_atoms {
        metrics.active_atoms = active;
    }

    if let Some(tokens) = tokens_added {
        let elapsed_secs = (now - metrics.last_updated_ms) as f64 / 1000.0;
        if elapsed_secs > 0.0 {
            // Rolling average of tokens per second
            let instant_tps = tokens as f64 / elapsed_secs;
            metrics.tokens_per_second = (metrics.tokens_per_second * 0.7) + (instant_tps * 0.3);
        }
        metrics.total_tokens += tokens;
    }

    if let Some(red_flags) = red_flags_added {
        metrics.red_flag_count += red_flags;
    }

    metrics.last_updated_ms = now;

    Ok(())
}

/// Increment atom spawn count and optionally shadow commits
#[tauri::command]
fn record_atom_spawned() -> Result<(), String> {
    let mut metrics = EXECUTION_METRICS.lock()
        .map_err(|_| "Failed to acquire metrics lock while recording atom spawn")?;

    metrics.total_atoms_spawned += 1;
    metrics.active_atoms += 1;
    metrics.last_updated_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    Ok(())
}

/// Record atom completion
#[tauri::command]
fn record_atom_completed(tokens_used: u64, had_red_flag: bool) -> Result<(), String> {
    let mut metrics = EXECUTION_METRICS.lock()
        .map_err(|_| "Failed to acquire metrics lock while recording atom completion")?;

    if metrics.active_atoms > 0 {
        metrics.active_atoms -= 1;
    }

    metrics.total_tokens += tokens_used;

    if had_red_flag {
        metrics.red_flag_count += 1;
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let elapsed_secs = (now - metrics.last_updated_ms) as f64 / 1000.0;
    if elapsed_secs > 0.0 && tokens_used > 0 {
        let instant_tps = tokens_used as f64 / elapsed_secs;
        metrics.tokens_per_second = (metrics.tokens_per_second * 0.8) + (instant_tps * 0.2);
    }

    metrics.last_updated_ms = now;

    Ok(())
}

/// Record a shadow commit (snapshot)
#[tauri::command]
fn record_shadow_commit() -> Result<(), String> {
    let mut metrics = EXECUTION_METRICS.lock()
        .map_err(|_| "Failed to acquire metrics lock while recording shadow commit")?;

    metrics.shadow_commits += 1;

    Ok(())
}

/// Reset execution metrics (e.g., when starting a new session)
#[tauri::command]
fn reset_execution_metrics() -> Result<(), String> {
    let mut metrics = EXECUTION_METRICS.lock()
        .map_err(|_| "Failed to acquire metrics lock while resetting metrics")?;

    *metrics = ExecutionMetrics::default();

    Ok(())
}

/// Get current voting state
#[tauri::command]
fn get_voting_state() -> Result<VotingState, String> {
    let state = VOTING_STATE.lock()
        .map_err(|_| "Failed to acquire voting state lock while getting voting state")?;

    Ok(state.clone())
}

/// Start a voting session for a task
#[tauri::command]
fn start_voting(task_id: String, task_description: String) -> Result<(), String> {
    let mut state = VOTING_STATE.lock()
        .map_err(|_| "Failed to acquire voting state lock while starting voting session")?;

    *state = VotingState {
        task_id,
        task_description,
        candidates: Vec::new(),
        is_voting: true,
        winner_id: None,
    };

    Ok(())
}

/// Add a voting candidate
#[tauri::command]
fn add_voting_candidate(
    snippet: String,
    score: f64,
    red_flags: Vec<String>,
) -> Result<usize, String> {
    let mut state = VOTING_STATE.lock()
        .map_err(|_| "Failed to acquire voting state lock while adding candidate")?;

    let id = state.candidates.len() + 1;
    let status = if red_flags.is_empty() { "pending" } else { "rejected" };

    state.candidates.push(VotingCandidate {
        id,
        snippet,
        score,
        red_flags,
        status: status.to_string(),
        votes: 0,
    });

    Ok(id)
}

/// Record a vote for a candidate
#[tauri::command]
fn record_vote(candidate_id: usize) -> Result<(), String> {
    let mut state = VOTING_STATE.lock()
        .map_err(|_| "Failed to acquire voting state lock while recording vote")?;

    if let Some(candidate) = state.candidates.iter_mut().find(|c| c.id == candidate_id) {
        candidate.votes += 1;
    }

    Ok(())
}

/// Complete voting with a winner
#[tauri::command]
fn complete_voting(winner_id: usize) -> Result<(), String> {
    let mut state = VOTING_STATE.lock()
        .map_err(|_| "Failed to acquire voting state lock while completing voting")?;

    state.is_voting = false;
    state.winner_id = Some(winner_id);

    // Update candidate statuses
    for candidate in &mut state.candidates {
        if candidate.id == winner_id {
            candidate.status = "accepted".to_string();
        } else if candidate.status == "pending" {
            candidate.status = "rejected".to_string();
        }
    }

    Ok(())
}

/// Clear voting state
#[tauri::command]
fn clear_voting_state() -> Result<(), String> {
    let mut state = VOTING_STATE.lock()
        .map_err(|_| "Failed to acquire voting state lock while clearing voting state")?;

    *state = VotingState::default();

    Ok(())
}

// ============================================================================
// Shadow Git Commands - Transactional File System
// ============================================================================

/// Create a snapshot (Shadow Git)
/// PRD 5.1: "Before any Rhai script touches disk, gitoxide creates a blob"
#[tauri::command]
fn create_snapshot(message: String) -> Result<serde_json::Value, String> {
    use validation::{make_error, make_error_with_code};

    validation::validate_non_empty(&message, "Snapshot message")
        .map_err(|e| make_error("SNAPSHOT", &e))?;

    let mut sg = SHADOW_GIT.lock()
        .map_err(|_| make_error_with_code("SNAPSHOT", "LOCK", "Failed to acquire shadow git lock"))?;

    let shadow_git = sg.as_mut()
        .ok_or_else(|| make_error_with_code("SNAPSHOT", "NOT_INIT", "Shadow Git not initialized. Call init_runtime first."))?;

    let snapshot = shadow_git.snapshot(&message)
        .map_err(|e| make_error("SNAPSHOT", &format!("Failed to create: {}", e)))?;

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
    use validation::{make_error, make_error_with_code};

    let mut sg = SHADOW_GIT.lock()
        .map_err(|_| make_error_with_code("SNAPSHOT", "LOCK", "Failed to acquire shadow git lock"))?;

    let shadow_git = sg.as_mut()
        .ok_or_else(|| make_error_with_code("SNAPSHOT", "NOT_INIT", "Shadow Git not initialized. Call init_runtime first."))?;

    shadow_git.rollback()
        .map_err(|e| make_error("SNAPSHOT", &format!("Rollback failed: {}", e)))?;

    Ok("Rolled back to previous snapshot".to_string())
}

/// Rollback to a specific snapshot by ID
#[tauri::command]
fn rollback_to_snapshot(snapshot_id: String) -> Result<String, String> {
    let mut sg = SHADOW_GIT.lock()
        .map_err(|_| "Failed to acquire shadow git lock while rolling back to specific snapshot")?;

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
        .map_err(|_| "Failed to acquire shadow git lock while squashing snapshots")?;

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
        .map_err(|_| "Failed to acquire shadow git lock while getting snapshots")?;

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
fn checkout_commit(workspace_path: String, commit_hash: String) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .current_dir(&workspace_path)
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
fn get_git_history(workspace_path: String, limit: usize) -> Result<serde_json::Value, String> {
    let output = std::process::Command::new("git")
        .current_dir(&workspace_path)
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
    std::fs::write(&path, &json)
        .map_err(|e| format!("Failed to write settings: {}", e))?;

    // Update cached settings so get_llm_provider uses them immediately
    if let Ok(mut cached) = CACHED_SETTINGS.lock() {
        *cached = Some(settings);
    }

    Ok(())
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
    // First, try to get from cache
    let mut cached = CACHED_SETTINGS.lock()
        .map_err(|_| "Failed to lock settings while getting LLM provider")?;

    // If cache is empty, try to load from disk
    if cached.is_none() {
        let path = get_settings_path();
        if path.exists() {
            if let Ok(json) = std::fs::read_to_string(&path) {
                if let Ok(loaded_settings) = serde_json::from_str::<AppSettings>(&json) {
                    *cached = Some(loaded_settings);
                }
            }
        }
    }

    // Use settings if available, otherwise default config
    let config = if let Some(s) = cached.as_ref() {
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
                "openrouter" => llm::ProviderType::OpenRouter,
                _ => llm::ProviderType::OpenAI,
            },
            model: interrogator_config.model.clone(),
            api_key,
            base_url: None,
            temperature: interrogator_config.temperature,
            max_tokens: 4096,
            timeout_secs: 120,
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

/// P2-1: Conversation message structure for maintaining history
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ConversationMessage {
    role: String,
    content: String,
}

// =============================================================================
// Threaded Messaging System - Agent Unblocker
// =============================================================================

/// Type of thread - determines the nature of the agent's request
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ThreadType {
    #[serde(rename = "help_request")]
    HelpRequest,
    #[serde(rename = "clarification")]
    Clarification,
    #[serde(rename = "resource_needed")]
    ResourceNeeded,
    #[serde(rename = "approval_needed")]
    ApprovalNeeded,
}

/// Current status of the thread
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum ThreadStatus {
    #[serde(rename = "open")]
    Open,
    #[serde(rename = "resolved")]
    Resolved,
    #[serde(rename = "pending")]
    Pending,
}

/// Priority level for thread handling
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ThreadPriority {
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "high")]
    High,
    #[serde(rename = "urgent")]
    Urgent,
}

/// Resource attachment that can be shared in thread messages
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThreadAttachment {
    #[serde(rename = "type")]
    pub attachment_type: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
}

/// Individual message within a thread
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThreadMessage {
    pub id: String,
    pub thread_id: String,
    pub role: String,  // "agent" or "human"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_name: Option<String>,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<ThreadAttachment>>,
    pub timestamp_ms: u64,
}

/// A conversation thread - created by agents when blocked, resolved by humans
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Thread {
    pub id: String,
    #[serde(rename = "type")]
    pub thread_type: ThreadType,
    pub status: ThreadStatus,
    pub priority: ThreadPriority,
    pub title: String,
    pub agent_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_at_ms: Option<u64>,
    pub messages: Vec<ThreadMessage>,
    pub is_blocking: bool,
}

/// Global thread storage for agent unblocker system
static THREADS: Mutex<Vec<Thread>> = Mutex::new(Vec::new());

/// Send a message to L1 and get a response
/// P2-1: Now supports conversation history for multi-turn interactions
#[tauri::command]
async fn send_interrogation_message(
    message: String,
    context: serde_json::Value,
    conversation_history: Option<Vec<ConversationMessage>>,
) -> Result<serde_json::Value, String> {
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

            // Build messages array with conversation history
            let mut messages = vec![llm::Message::system(&system_prompt)];

            // Add conversation history if provided (P2-1: Multi-turn support)
            if let Some(history) = conversation_history {
                for msg in history {
                    match msg.role.as_str() {
                        "user" => messages.push(llm::Message::user(&msg.content)),
                        "assistant" => messages.push(llm::Message::assistant(&msg.content)),
                        _ => {} // Ignore unknown roles
                    }
                }
            }

            // Add the current message
            messages.push(llm::Message::user(&message));

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
    // Build conversation text for the LLM
    let conversation_text: Vec<String> = conversation.iter()
        .filter_map(|msg| {
            let role = msg.get("role")?.as_str()?;
            let content = msg.get("content")?.as_str()?;
            Some(format!("{}: {}", role, content))
        })
        .collect();

    // Try to use LLM to generate PLAN.md
    match get_llm_provider() {
        Ok(provider) => {
            let system_prompt = r#"You are the L1 Product Orchestrator in the Cerebras-MAKER system.
Your role is to synthesize a conversation about a project into a structured PLAN.md document.

The PLAN.md format must follow this structure:
1. Start with a title: # Project Plan: [Project Name]
2. Include an overview section: ## Overview
3. Break down into phases: ## Phase 1: [Phase Name]
4. Each phase contains tasks as checkboxes: - [ ] Task description

Task descriptions should be atomic and actionable. Include keywords that help identify the task type:
- "Create/Implement/Add" for coding tasks
- "Test/Verify/Assert" for testing tasks
- "Review/Check/Validate" for review tasks
- "Design/Architect/Interface" for design tasks
- "Analyze/Topology/Dependency" for analysis tasks

Example output:
```markdown
# Project Plan: User Authentication System

## Overview
Implement a secure user authentication system with JWT tokens.

## Phase 1: Core Authentication
- [ ] Create User model with email and password fields
- [ ] Implement password hashing using bcrypt
- [ ] Create login endpoint with JWT token generation
- [ ] Add token validation middleware

## Phase 2: Testing & Validation
- [ ] Test user registration flow
- [ ] Verify password hashing security
- [ ] Review authentication middleware for vulnerabilities
```

Output ONLY the PLAN.md content, no additional commentary."#;

            let user_prompt = format!(
                "Based on the following conversation, generate a PLAN.md document:\n\n{}",
                conversation_text.join("\n\n")
            );

            let messages = vec![
                llm::Message::system(system_prompt),
                llm::Message::user(&user_prompt),
            ];

            match provider.complete(messages).await {
                Ok(response) => {
                    let plan_md = extract_plan_md(&response.content);

                    // Parse the generated plan to extract tasks
                    let orchestrator = agents::Orchestrator::new();
                    let tasks = match orchestrator.parse_plan_md(&plan_md) {
                        Ok(plan) => plan.micro_tasks.iter().map(|t| serde_json::json!({
                            "id": t.id,
                            "description": t.description,
                            "atom_type": t.atom_type
                        })).collect::<Vec<_>>(),
                        Err(_) => Vec::new(),
                    };

                    Ok(serde_json::json!({
                        "status": "complete",
                        "plan_md": plan_md,
                        "tasks": tasks
                    }))
                }
                Err(_e) => {
                    // Fall back to mock if LLM fails
                    Ok(generate_mock_plan(&conversation_text))
                }
            }
        }
        Err(_) => {
            // Fall back to mock if no provider configured
            Ok(generate_mock_plan(&conversation_text))
        }
    }
}

/// Extract PLAN.md content from LLM response (handles markdown code blocks)
fn extract_plan_md(response: &str) -> String {
    // Try to find markdown in code blocks first
    if let Some(start) = response.find("```markdown") {
        if let Some(end) = response[start..].find("```\n").or_else(|| response[start..].rfind("```")) {
            let md_start = start + 11; // Skip "```markdown"
            let md_end = start + end;
            if md_start < md_end {
                return response[md_start..md_end].trim().to_string();
            }
        }
    }

    // Try generic code blocks
    if let Some(start) = response.find("```") {
        let after_start = start + 3;
        let content_start = response[after_start..]
            .find('\n')
            .map(|i| after_start + i + 1)
            .unwrap_or(after_start);
        if let Some(end) = response[content_start..].find("```") {
            return response[content_start..content_start + end].trim().to_string();
        }
    }

    // Return as-is if no code blocks
    response.trim().to_string()
}

/// Generate a mock PLAN.md when LLM is not available
fn generate_mock_plan(conversation_text: &[String]) -> serde_json::Value {
    serde_json::json!({
        "status": "complete",
        "plan_md": format!(
            "# Project Plan\n\n## Overview\n\nGenerated from {} messages.\n\n## Phase 1: Implementation\n\n- [ ] Analyze requirements from conversation\n- [ ] Design system architecture\n- [ ] Implement core functionality\n- [ ] Test implementation\n- [ ] Review and validate\n\n## Conversation Summary\n\n{}",
            conversation_text.len(),
            conversation_text.join("\n\n")
        ),
        "tasks": [
            {"id": "t1", "description": "Analyze requirements from conversation", "atom_type": "Analyzer"},
            {"id": "t2", "description": "Design system architecture", "atom_type": "Architect"},
            {"id": "t3", "description": "Implement core functionality", "atom_type": "Coder"},
            {"id": "t4", "description": "Test implementation", "atom_type": "Tester"},
            {"id": "t5", "description": "Review and validate", "atom_type": "Reviewer"}
        ]
    })
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
    use agents::context_engineer::ContextEngineer;
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
        {"id": "GritsAnalyzer", "name": "GritsAnalyzer", "description": "Analyze topology", "max_tokens": 1000},
        {"id": "RLMProcessor", "name": "RLMProcessor", "description": "Process large contexts recursively", "max_tokens": 4000}
    ])
}

// ============================================================================
// RLM (Recursive Language Model) Commands
// Based on: "Recursive Language Models: Scaling Context with Recursive Prompt Decomposition"
// ============================================================================

use maker_core::{RLMConfig, RLMTrajectoryStep, RLMOperation, ContextType, create_shared_store};

/// Global RLM context store for cross-command access
static RLM_STORE: Mutex<Option<maker_core::SharedRLMContextStore>> = Mutex::new(None);

/// Global RLM trajectory for visualization
static RLM_TRAJECTORY: Mutex<Vec<RLMTrajectoryStep>> = Mutex::new(Vec::new());

/// Initialize the RLM context store
#[tauri::command]
fn init_rlm_store() -> Result<String, String> {
    let store = create_shared_store();
    if let Ok(mut global) = RLM_STORE.lock() {
        *global = Some(store);
    }
    if let Ok(mut traj) = RLM_TRAJECTORY.lock() {
        traj.clear();
    }
    Ok("RLM store initialized".to_string())
}

/// Load a large context into the RLM store
#[tauri::command]
fn rlm_load_context(
    var_name: String,
    content: String,
    context_type: String,
) -> Result<serde_json::Value, String> {
    let store = RLM_STORE.lock()
        .map_err(|_| "Failed to acquire RLM store lock while setting context")?;

    let store = store.as_ref()
        .ok_or("RLM store not initialized. Call init_rlm_store first.")?;

    let ctx_type = match context_type.as_str() {
        "minicodebase" => ContextType::MiniCodebase,
        "symbolgraph" => ContextType::SymbolGraph,
        "file" => ContextType::File,
        _ => ContextType::String,
    };

    let length = content.len();

    if let Ok(mut s) = store.lock() {
        s.load_variable(&var_name, content, ctx_type);
    }

    // Log trajectory with memory estimate
    if let Ok(mut traj) = RLM_TRAJECTORY.lock() {
        let step_num = traj.len();
        let memory_estimate = length + std::mem::size_of::<String>() + 64;
        traj.push(RLMTrajectoryStep {
            step: step_num,
            operation: RLMOperation::LoadContext {
                var_name: var_name.clone(),
                length,
            },
            description: format!("Loaded '{}' ({} chars, ~{} bytes)", var_name, length, memory_estimate),
            data: None,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            duration_ms: None,
            memory_estimate_bytes: Some(memory_estimate),
            error_context: None,
        });
    }

    Ok(serde_json::json!({
        "var_name": var_name,
        "length": length,
        "context_type": context_type
    }))
}

/// Peek at a portion of a context variable
#[tauri::command]
fn rlm_peek_context(
    var_name: String,
    start: usize,
    end: usize,
) -> Result<String, String> {
    let store = RLM_STORE.lock()
        .map_err(|_| "Failed to acquire RLM store lock while peeking context")?;

    let store = store.as_ref()
        .ok_or("RLM store not initialized")?;

    let result = if let Ok(s) = store.lock() {
        s.peek(&var_name, start, end)
            .ok_or_else(|| format!("Context variable '{}' not found", var_name))?
    } else {
        return Err("Failed to lock store".to_string());
    };

    // Log trajectory with memory estimate for slice
    if let Ok(mut traj) = RLM_TRAJECTORY.lock() {
        let step_num = traj.len();
        let slice_len = result.len();
        traj.push(RLMTrajectoryStep {
            step: step_num,
            operation: RLMOperation::Peek {
                var_name: var_name.clone(),
                start,
                end,
            },
            description: format!("Peeked '{}' [{}-{}] ({} chars)", var_name, start, end, slice_len),
            data: None,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            duration_ms: None,
            memory_estimate_bytes: Some(slice_len),
            error_context: None,
        });
    }

    Ok(result)
}

/// Get the length of a context variable
#[tauri::command]
fn rlm_context_length(var_name: String) -> Result<usize, String> {
    let guard = RLM_STORE.lock()
        .map_err(|_| "Failed to acquire RLM store lock while getting context length")?;

    let shared_store = guard.as_ref()
        .ok_or("RLM store not initialized")?
        .clone();

    drop(guard);

    let inner = shared_store.lock()
        .map_err(|_| "Failed to lock inner store while getting context length")?;

    inner.length(&var_name)
        .ok_or_else(|| format!("Context variable '{}' not found", var_name))
}

/// Chunk a context variable into smaller pieces
#[tauri::command]
fn rlm_chunk_context(
    var_name: String,
    chunk_size: usize,
) -> Result<serde_json::Value, String> {
    let store = RLM_STORE.lock()
        .map_err(|_| "Failed to acquire RLM store lock while chunking context")?;

    let store = store.as_ref()
        .ok_or("RLM store not initialized")?;

    let chunks = if let Ok(mut s) = store.lock() {
        s.chunk(&var_name, chunk_size)
    } else {
        return Err("Failed to lock store".to_string());
    };

    let num_chunks = chunks.len();

    // Log trajectory
    if let Ok(mut traj) = RLM_TRAJECTORY.lock() {
        let step_num = traj.len();
        traj.push(RLMTrajectoryStep {
            step: step_num,
            operation: RLMOperation::Chunk {
                var_name: var_name.clone(),
                num_chunks,
            },
            description: format!("Chunked '{}' into {} pieces (size {})", var_name, num_chunks, chunk_size),
            data: None,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            duration_ms: None,
            memory_estimate_bytes: None,
            error_context: None,
        });
    }

    Ok(serde_json::json!({
        "var_name": var_name,
        "chunk_size": chunk_size,
        "num_chunks": num_chunks,
        "chunks": chunks
    }))
}

/// Filter a context variable using regex
#[tauri::command]
fn rlm_regex_filter(
    var_name: String,
    pattern: String,
) -> Result<serde_json::Value, String> {
    let store = RLM_STORE.lock()
        .map_err(|_| "Failed to acquire RLM store lock while applying regex filter")?;

    let store = store.as_ref()
        .ok_or("RLM store not initialized")?;

    let matches = if let Ok(s) = store.lock() {
        s.regex_filter(&var_name, &pattern)
            .map_err(|e| format!("Regex error: {}", e))?
    } else {
        return Err("Failed to lock store".to_string());
    };

    let num_matches = matches.len();

    // Log trajectory
    if let Ok(mut traj) = RLM_TRAJECTORY.lock() {
        let step_num = traj.len();
        traj.push(RLMTrajectoryStep {
            step: step_num,
            operation: RLMOperation::RegexFilter {
                var_name: var_name.clone(),
                pattern: pattern.clone(),
                matches: num_matches,
            },
            description: format!("Regex '{}' on '{}': {} matches", pattern, var_name, num_matches),
            data: None,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            duration_ms: None,
            memory_estimate_bytes: None,
            error_context: None,
        });
    }

    Ok(serde_json::json!({
        "var_name": var_name,
        "pattern": pattern,
        "num_matches": num_matches,
        "matches": matches
    }))
}

/// P2-2: Execute an RLM-aware atom with iterative loop
/// Based on RLM paper: max_iterations=20, max_depth=1
#[tauri::command]
async fn execute_rlm_atom(
    _atom_type: String,
    task: String,
    context_var: String,
    max_iterations: usize,
) -> Result<serde_json::Value, String> {
    use agents::atom_executor::{AtomExecutor, AtomInput};
    use maker_core::{SpawnFlags, RLMAction, RLMExecutionState, RLMConfig, RLMResult};

    let config = RLMConfig::default();
    let effective_max_iterations = if max_iterations > 0 { max_iterations } else { config.max_iterations };
    let start_time = std::time::Instant::now();

    // Get context info from store
    let (context_length, context_preview) = {
        let guard = RLM_STORE.lock()
            .map_err(|_| "Failed to acquire RLM store lock while executing RLM call")?;
        let shared_store = guard.as_ref()
            .ok_or("RLM store not initialized")?
            .clone();
        drop(guard);
        let inner = shared_store.lock()
            .map_err(|_| "Failed to lock inner store while executing RLM call")?;
        let len = inner.length(&context_var).unwrap_or(0);
        let preview = inner.peek(&context_var, 0, 1000).unwrap_or_default();
        (len, preview)
    };

    // Initialize execution state
    let mut state = RLMExecutionState::new();
    state.add_step(
        RLMOperation::Start,
        format!("RLM execution started for '{}' ({} chars, max {} iterations)",
                context_var, context_length, effective_max_iterations),
        None
    );

    // Log to global trajectory
    if let Ok(mut traj) = RLM_TRAJECTORY.lock() {
        traj.extend(state.trajectory.clone());
    }

    // Get LLM config
    let llm_config = get_llm_config_from_settings()?;
    let executor = AtomExecutor::new(llm_config.clone());

    // Build context info string
    let context_info = format!(
        "**Context Variable**: `{}`\n**Context Length**: {} characters\n**Preview (first 1000 chars)**:\n```\n{}\n```",
        context_var, context_length, context_preview
    );

    // P2-2: Iterative RLM loop (based on RLM paper Section 3.1)
    let mut final_answer: Option<String> = None;

    while state.iteration < effective_max_iterations {
        state.iteration += 1;

        // Build iteration prompt
        let prompt = state.build_iteration_prompt(&task, &context_info);

        // Execute LLM call
        let flags = SpawnFlags {
            require_json: true,
            temperature: 0.1,
            max_tokens: Some(4000),
            red_flag_check: false, // Don't red-flag RLM iterations
        };
        let input = AtomInput::new(AtomType::RLMProcessor, &prompt).with_flags(flags);

        let result = match executor.execute(input).await {
            Ok(r) => r,
            Err(e) => {
                state.add_step(
                    RLMOperation::Error { message: e.clone() },
                    format!("Iteration {} failed: {}", state.iteration, e),
                    None
                );
                break;
            }
        };

        // Parse the action from LLM output
        let action = match RLMAction::parse(&result.output) {
            Ok(a) => a,
            Err(e) => {
                state.add_observation(format!("Parse error: {}. Raw output: {}", e,
                    result.output.chars().take(200).collect::<String>()));
                continue;
            }
        };

        // Execute the action
        match action {
            RLMAction::Peek { var_name, start, end } => {
                let peek_result = {
                    let guard = RLM_STORE.lock().map_err(|_| "Lock error")?;
                    let store = guard.as_ref().ok_or("Store not initialized")?;
                    let inner = store.lock().map_err(|_| "Inner lock error")?;
                    inner.peek(&var_name, start, end).unwrap_or_else(|| format!("Variable '{}' not found", var_name))
                };
                state.add_step(
                    RLMOperation::Peek { var_name: var_name.clone(), start, end },
                    format!("Peeked '{}' [{}-{}]: {} chars", var_name, start, end, peek_result.len()),
                    None
                );
                state.add_observation(format!("peek({}, {}, {}) = \"{}...\"",
                    var_name, start, end, peek_result.chars().take(100).collect::<String>()));
            }
            RLMAction::Chunk { var_name, chunk_size } => {
                let num_chunks = {
                    let guard = RLM_STORE.lock().map_err(|_| "Lock error")?;
                    let store = guard.as_ref().ok_or("Store not initialized")?;
                    let mut inner = store.lock().map_err(|_| "Inner lock error")?;
                    inner.chunk(&var_name, chunk_size).len()
                };
                state.add_step(
                    RLMOperation::Chunk { var_name: var_name.clone(), num_chunks },
                    format!("Chunked '{}' into {} pieces of {} chars", var_name, num_chunks, chunk_size),
                    None
                );
                state.add_observation(format!("chunk({}, {}) = {} chunks", var_name, chunk_size, num_chunks));
            }
            RLMAction::RegexFilter { var_name, pattern } => {
                let matches = {
                    let guard = RLM_STORE.lock().map_err(|_| "Lock error")?;
                    let store = guard.as_ref().ok_or("Store not initialized")?;
                    let inner = store.lock().map_err(|_| "Inner lock error")?;
                    inner.regex_filter(&var_name, &pattern).unwrap_or_default()
                };
                let num_matches = matches.len();
                state.add_step(
                    RLMOperation::RegexFilter { var_name: var_name.clone(), pattern: pattern.clone(), matches: num_matches },
                    format!("Regex '{}' on '{}': {} matches", pattern, var_name, num_matches),
                    Some(serde_json::json!(matches.iter().take(5).collect::<Vec<_>>()))
                );
                state.add_observation(format!("regex_filter({}, \"{}\") = {} matches: {:?}",
                    var_name, pattern, num_matches, matches.iter().take(3).collect::<Vec<_>>()));
            }
            RLMAction::SubQuery { prompt } => {
                // Check depth limit (max_depth=1 means sub-calls are regular LLMs)
                if state.depth >= config.max_depth {
                    state.add_observation(format!("Sub-query depth limit ({}) reached", config.max_depth));
                    continue;
                }
                state.sub_calls += 1;
                state.add_step(
                    RLMOperation::SubQuery { prompt_preview: prompt.chars().take(100).collect(), depth: state.depth + 1 },
                    format!("Sub-query at depth {}", state.depth + 1),
                    None
                );
                // Execute sub-query as regular LLM (not RLM)
                let sub_input = AtomInput::new(AtomType::Coder, &prompt)
                    .with_flags(SpawnFlags { temperature: 0.1, ..Default::default() });
                match executor.execute(sub_input).await {
                    Ok(sub_result) => {
                        state.add_step(
                            RLMOperation::SubResult { result_preview: sub_result.output.chars().take(100).collect() },
                            "Sub-query completed".to_string(),
                            None
                        );
                        state.add_observation(format!("llm_query result: \"{}...\"",
                            sub_result.output.chars().take(200).collect::<String>()));
                    }
                    Err(e) => {
                        state.add_observation(format!("Sub-query failed: {}", e));
                    }
                }
            }
            RLMAction::Final { answer } => {
                state.add_step(
                    RLMOperation::Final { answer_preview: answer.chars().take(100).collect() },
                    format!("Final answer after {} iterations", state.iteration),
                    None
                );
                final_answer = Some(answer);
                break;
            }
            RLMAction::Continue { reasoning } => {
                state.add_observation(format!("Thinking: {}", reasoning));
            }
        }

        // Update global trajectory
        if let Ok(mut traj) = RLM_TRAJECTORY.lock() {
            *traj = state.trajectory.clone();
        }
    }

    // Build result
    let execution_time_ms = start_time.elapsed().as_millis() as u64;
    let result = if let Some(answer) = final_answer {
        RLMResult {
            success: true,
            output: answer,
            iterations: state.iteration,
            sub_calls: state.sub_calls,
            total_tokens: 0,
            trajectory: state.trajectory.clone(),
            errors: Vec::new(),
            execution_time_ms,
        }
    } else {
        RLMResult {
            success: false,
            output: format!("Max iterations ({}) reached without final answer", effective_max_iterations),
            iterations: state.iteration,
            sub_calls: state.sub_calls,
            total_tokens: 0,
            trajectory: state.trajectory.clone(),
            errors: vec!["Max iterations reached".to_string()],
            execution_time_ms,
        }
    };

    // Final trajectory update
    if let Ok(mut traj) = RLM_TRAJECTORY.lock() {
        *traj = state.trajectory;
    }

    serde_json::to_value(&result).map_err(|e| e.to_string())
}

/// Get the RLM execution trajectory for visualization
#[tauri::command]
fn get_rlm_trajectory() -> Result<serde_json::Value, String> {
    let trajectory = RLM_TRAJECTORY.lock()
        .map_err(|_| "Failed to acquire trajectory lock while getting RLM trajectory")?;

    serde_json::to_value(&*trajectory).map_err(|e| e.to_string())
}

/// Clear the RLM trajectory
#[tauri::command]
fn clear_rlm_trajectory() -> Result<String, String> {
    if let Ok(mut traj) = RLM_TRAJECTORY.lock() {
        traj.clear();
    }
    Ok("Trajectory cleared".to_string())
}

/// List all context variables in the RLM store
#[tauri::command]
fn rlm_list_contexts() -> Result<serde_json::Value, String> {
    let store = RLM_STORE.lock()
        .map_err(|_| "Failed to acquire RLM store lock while listing contexts")?;

    let store = store.as_ref()
        .ok_or("RLM store not initialized")?;

    let vars = if let Ok(s) = store.lock() {
        s.list_variables()
    } else {
        return Err("Failed to lock store".to_string());
    };

    Ok(serde_json::json!(vars))
}

/// Clear a context variable from the RLM store
#[tauri::command]
fn rlm_clear_context(var_name: String) -> Result<bool, String> {
    let guard = RLM_STORE.lock()
        .map_err(|_| "Failed to acquire RLM store lock while clearing context")?;

    let shared_store = guard.as_ref()
        .ok_or("RLM store not initialized")?
        .clone();

    drop(guard);

    let mut inner = shared_store.lock()
        .map_err(|_| "Failed to lock inner store while clearing context")?;

    Ok(inner.remove(&var_name).is_some())
}

/// HIGH-10: Clear all context variables from the RLM store
/// Use this for cleanup after long-running operations to free memory
#[tauri::command]
fn rlm_clear_all() -> Result<usize, String> {
    let guard = RLM_STORE.lock()
        .map_err(|_| "Failed to acquire RLM store lock while clearing all contexts")?;

    let shared_store = guard.as_ref()
        .ok_or("RLM store not initialized")?
        .clone();

    drop(guard);

    let mut inner = shared_store.lock()
        .map_err(|_| "Failed to lock inner store while clearing all contexts")?;

    let count = inner.list_variables().len();
    inner.clear();
    Ok(count)
}

/// Get RLM configuration
#[tauri::command]
fn get_rlm_config() -> serde_json::Value {
    let config = RLMConfig::default();
    serde_json::json!({
        "max_depth": config.max_depth,
        "max_iterations": config.max_iterations,
        "default_chunk_size": config.default_chunk_size,
        "rlm_threshold": config.rlm_threshold,
        "use_sub_model": config.use_sub_model
    })
}

/// Check if context size exceeds RLM threshold
#[tauri::command]
fn should_use_rlm(context_length: usize) -> bool {
    context_length >= RLMConfig::default().rlm_threshold
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

// ============================================================================
// GitHub Integration Commands
// ============================================================================

/// Initialize a git repository in the workspace
#[tauri::command]
async fn git_init(workspace_path: String) -> Result<String, String> {
    validation::validate_workspace_path(&workspace_path)?;

    let output = std::process::Command::new("git")
        .current_dir(&workspace_path)
        .args(["init"])
        .output()
        .map_err(|e| format!("Failed to initialize git repository at '{}': {}", workspace_path, e))?;

    if !output.status.success() {
        return Err(format!("Failed to initialize git repository at '{}': {}", workspace_path, String::from_utf8_lossy(&output.stderr)));
    }
    Ok("Repository initialized".to_string())
}

/// Add a remote to the repository
#[tauri::command]
async fn git_add_remote(workspace_path: String, name: String, url: String) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .current_dir(&workspace_path)
        .args(["remote", "add", &name, &url])
        .output()
        .map_err(|e| format!("Failed to add remote '{}' with URL '{}': {}", name, url, e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        // Check if remote already exists
        if stderr.contains("already exists") {
            // Update existing remote
            let _ = std::process::Command::new("git")
                .current_dir(&workspace_path)
                .args(["remote", "set-url", &name, &url])
                .output();
            return Ok(format!("Remote '{}' updated to {}", name, url));
        }
        return Err(format!("Failed to add remote '{}' with URL '{}': {}", name, url, stderr));
    }
    Ok(format!("Remote '{}' added: {}", name, url))
}

/// Get remote information
#[tauri::command]
async fn git_get_remotes(workspace_path: String) -> Result<serde_json::Value, String> {
    let output = std::process::Command::new("git")
        .current_dir(&workspace_path)
        .args(["remote", "-v"])
        .output()
        .map_err(|e| format!("Failed to get remotes for repository at '{}': {}", workspace_path, e))?;

    if !output.status.success() {
        return Err(format!("Failed to get remotes for repository at '{}': {}", workspace_path, String::from_utf8_lossy(&output.stderr)));
    }

    let remotes: Vec<serde_json::Value> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                Some(serde_json::json!({
                    "name": parts[0],
                    "url": parts[1],
                    "type": parts.get(2).unwrap_or(&"")
                }))
            } else {
                None
            }
        })
        .collect();

    Ok(serde_json::json!({ "remotes": remotes }))
}

/// Push changes to remote
#[tauri::command]
async fn git_push(workspace_path: String, remote: String, branch: String, set_upstream: bool) -> Result<String, String> {
    let mut args = vec!["push", &remote, &branch];
    if set_upstream {
        args.insert(1, "-u");
    }

    let output = std::process::Command::new("git")
        .current_dir(&workspace_path)
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to push to '{}/{}': {}", remote, branch, e))?;

    if !output.status.success() {
        return Err(format!("Failed to push to '{}/{}': {}", remote, branch, String::from_utf8_lossy(&output.stderr)));
    }
    Ok(format!("Pushed to {}/{}", remote, branch))
}

/// Get current branch name
#[tauri::command]
async fn git_current_branch(workspace_path: String) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .current_dir(&workspace_path)
        .args(["branch", "--show-current"])
        .output()
        .map_err(|e| format!("Failed to get current branch for repository at '{}': {}", workspace_path, e))?;

    if !output.status.success() {
        return Err(format!("Failed to get current branch for repository at '{}': {}", workspace_path, String::from_utf8_lossy(&output.stderr)));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Get repository status
#[tauri::command]
async fn git_status(workspace_path: String) -> Result<serde_json::Value, String> {
    let output = std::process::Command::new("git")
        .current_dir(&workspace_path)
        .args(["status", "--porcelain"])
        .output()
        .map_err(|e| format!("Failed to get status for repository at '{}': {}", workspace_path, e))?;

    if !output.status.success() {
        return Err(format!("Failed to get status for repository at '{}': {}", workspace_path, String::from_utf8_lossy(&output.stderr)));
    }

    let changes: Vec<serde_json::Value> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            if line.len() >= 3 {
                let status = &line[0..2];
                let file = &line[3..];
                Some(serde_json::json!({
                    "status": status.trim(),
                    "file": file
                }))
            } else {
                None
            }
        })
        .collect();

    let is_clean = changes.is_empty();

    Ok(serde_json::json!({
        "is_clean": is_clean,
        "changes": changes,
        "change_count": changes.len()
    }))
}

/// Clone a repository
#[tauri::command]
async fn git_clone(url: String, target_path: String) -> Result<String, String> {
    validation::validate_non_empty(&url, "Repository URL")?;
    validation::validate_file_path(&target_path)?;

    let output = std::process::Command::new("git")
        .args(["clone", &url, &target_path])
        .output()
        .map_err(|e| format!("Failed to clone '{}' to '{}': {}", url, target_path, e))?;

    if !output.status.success() {
        return Err(format!("Failed to clone '{}' to '{}': {}", url, target_path, String::from_utf8_lossy(&output.stderr)));
    }
    Ok(format!("Cloned {} to {}", url, target_path))
}

/// Stage files for commit
#[tauri::command]
async fn git_add(workspace_path: String, paths: Vec<String>) -> Result<String, String> {
    let mut args = vec!["add"];
    let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
    args.extend(path_refs);

    let output = std::process::Command::new("git")
        .current_dir(&workspace_path)
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to stage files in '{}': {}", workspace_path, e))?;

    if !output.status.success() {
        return Err(format!("Failed to stage files in '{}': {}", workspace_path, String::from_utf8_lossy(&output.stderr)));
    }
    Ok(format!("Staged {} files", paths.len()))
}

/// Commit staged changes
#[tauri::command]
async fn git_commit(workspace_path: String, message: String) -> Result<serde_json::Value, String> {
    let truncated_msg = if message.len() > 50 { format!("{}...", &message[..50]) } else { message.clone() };
    let output = std::process::Command::new("git")
        .current_dir(&workspace_path)
        .args(["commit", "-m", &message])
        .output()
        .map_err(|e| format!("Failed to commit with message '{}': {}", truncated_msg, e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        // Check if it's just "nothing to commit"
        if stderr.contains("nothing to commit") {
            return Ok(serde_json::json!({
                "success": false,
                "message": "Nothing to commit"
            }));
        }
        return Err(format!("Failed to commit with message '{}': {}", truncated_msg, stderr));
    }

    // Get the commit hash
    let hash_output = std::process::Command::new("git")
        .current_dir(&workspace_path)
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|e| format!("Failed to get commit hash: {}", e))?;

    let commit_hash = String::from_utf8_lossy(&hash_output.stdout).trim().to_string();

    Ok(serde_json::json!({
        "success": true,
        "commit_hash": commit_hash,
        "message": message
    }))
}

/// Create or switch branches
#[tauri::command]
async fn git_branch(workspace_path: String, branch_name: String, create: bool) -> Result<String, String> {
    let args = if create {
        vec!["checkout", "-b", &branch_name]
    } else {
        vec!["checkout", &branch_name]
    };

    let action = if create { "create and switch to" } else { "switch to" };
    let output = std::process::Command::new("git")
        .current_dir(&workspace_path)
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to {} branch '{}': {}", action, branch_name, e))?;

    if !output.status.success() {
        return Err(format!("Failed to {} branch '{}': {}", action, branch_name, String::from_utf8_lossy(&output.stderr)));
    }

    if create {
        Ok(format!("Created and switched to branch '{}'", branch_name))
    } else {
        Ok(format!("Switched to branch '{}'", branch_name))
    }
}

/// List all branches
#[tauri::command]
async fn git_list_branches(workspace_path: String) -> Result<serde_json::Value, String> {
    let output = std::process::Command::new("git")
        .current_dir(&workspace_path)
        .args(["branch", "-a", "--format=%(refname:short)|%(objectname:short)|%(upstream:short)"])
        .output()
        .map_err(|e| format!("Failed to list branches for repository at '{}': {}", workspace_path, e))?;

    if !output.status.success() {
        return Err(format!("Failed to list branches for repository at '{}': {}", workspace_path, String::from_utf8_lossy(&output.stderr)));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let branches: Vec<serde_json::Value> = output_str
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let parts: Vec<&str> = line.split('|').collect();
            serde_json::json!({
                "name": parts.first().unwrap_or(&""),
                "commit": parts.get(1).unwrap_or(&""),
                "upstream": parts.get(2).unwrap_or(&"")
            })
        })
        .collect();

    // Get current branch
    let current = std::process::Command::new("git")
        .current_dir(&workspace_path)
        .args(["branch", "--show-current"])
        .output()
        .map_err(|e| eprintln!("Warning: Failed to get current git branch: {}", e))
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();

    Ok(serde_json::json!({
        "branches": branches,
        "current": current
    }))
}

/// Pull changes from remote
#[tauri::command]
async fn git_pull(workspace_path: String, remote: String, branch: String, rebase: bool) -> Result<String, String> {
    let mut args = vec!["pull", &remote, &branch];
    if rebase {
        args.push("--rebase");
    }

    let output = std::process::Command::new("git")
        .current_dir(&workspace_path)
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to pull from '{}/{}': {}", remote, branch, e))?;

    if !output.status.success() {
        return Err(format!("Failed to pull from '{}/{}': {}", remote, branch, String::from_utf8_lossy(&output.stderr)));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Get diff of changes
#[tauri::command]
async fn git_diff(workspace_path: String, staged: bool, file_path: Option<String>) -> Result<String, String> {
    let mut args = vec!["diff"];
    if staged {
        args.push("--cached");
    }
    if let Some(ref path) = file_path {
        args.push("--");
        args.push(path);
    }

    let diff_target = file_path.as_ref().map(|p| format!(" for file '{}'", p)).unwrap_or_default();
    let output = std::process::Command::new("git")
        .current_dir(&workspace_path)
        .args(&args)
        .output()
        .map_err(|e| format!("Failed to get diff{} in '{}': {}", diff_target, workspace_path, e))?;

    if !output.status.success() {
        return Err(format!("Failed to get diff{} in '{}': {}", diff_target, workspace_path, String::from_utf8_lossy(&output.stderr)));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Get commit log
#[tauri::command]
async fn git_log(workspace_path: String, count: Option<usize>) -> Result<serde_json::Value, String> {
    let count_str = count.unwrap_or(20).to_string();
    let output = std::process::Command::new("git")
        .current_dir(&workspace_path)
        .args(["log", "--format=%H|%s|%an|%ae|%aI", "-n", &count_str])
        .output()
        .map_err(|e| format!("Failed to get commit log for repository at '{}': {}", workspace_path, e))?;

    if !output.status.success() {
        return Err(format!("Failed to get commit log for repository at '{}': {}", workspace_path, String::from_utf8_lossy(&output.stderr)));
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let commits: Vec<serde_json::Value> = output_str
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let parts: Vec<&str> = line.split('|').collect();
            serde_json::json!({
                "hash": parts.first().unwrap_or(&""),
                "message": parts.get(1).unwrap_or(&""),
                "author": parts.get(2).unwrap_or(&""),
                "email": parts.get(3).unwrap_or(&""),
                "date": parts.get(4).unwrap_or(&"")
            })
        })
        .collect();

    Ok(serde_json::json!({ "commits": commits }))
}

// ============================================================================
// GitHub Actions & Deployment Automation
// ============================================================================

/// Project type for workflow generation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkflowConfig {
    pub project_type: String,       // "tauri", "react", "node", "rust"
    pub node_version: Option<String>,
    pub rust_version: Option<String>,
    pub deploy_target: Option<String>,  // "vercel", "netlify", "github-pages"
    pub run_tests: bool,
    pub run_lint: bool,
}

/// Generate GitHub Actions workflow
#[tauri::command]
async fn generate_github_workflow(workspace_path: String, config: WorkflowConfig) -> Result<serde_json::Value, String> {
    let workflow = match config.project_type.as_str() {
        "tauri" => generate_tauri_workflow(&config),
        "react" | "vite" => generate_react_workflow(&config),
        "node" => generate_node_workflow(&config),
        "rust" => generate_rust_workflow(&config),
        _ => generate_generic_workflow(&config),
    };

    // Create .github/workflows directory
    let workflows_dir = std::path::Path::new(&workspace_path).join(".github").join("workflows");
    std::fs::create_dir_all(&workflows_dir)
        .map_err(|e| format!("Failed to create workflows directory: {}", e))?;

    // Write workflow file
    let workflow_path = workflows_dir.join("ci.yml");
    std::fs::write(&workflow_path, &workflow)
        .map_err(|e| format!("Failed to write workflow: {}", e))?;

    Ok(serde_json::json!({
        "success": true,
        "path": workflow_path.to_string_lossy(),
        "content": workflow
    }))
}

fn generate_tauri_workflow(config: &WorkflowConfig) -> String {
    let node_version = config.node_version.as_deref().unwrap_or("20");
    let rust_version = config.rust_version.as_deref().unwrap_or("stable");

    format!(r#"name: Tauri CI/CD

on:
  push:
    branches: [ main, master ]
  pull_request:
    branches: [ main, master ]

jobs:
  build:
    strategy:
      fail-fast: false
      matrix:
        platform: [macos-latest, ubuntu-22.04, windows-latest]

    runs-on: ${{{{ matrix.platform }}}}

    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '{node_version}'
          cache: 'npm'

      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          toolchain: {rust_version}

      - name: Install dependencies (Ubuntu)
        if: matrix.platform == 'ubuntu-22.04'
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf

      - name: Install frontend dependencies
        run: npm ci
      {}{}
      - name: Build Tauri app
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{{{ secrets.GITHUB_TOKEN }}}}
"#,
        if config.run_lint { "\n      - name: Lint\n        run: npm run lint\n" } else { "" },
        if config.run_tests { "\n      - name: Test\n        run: npm test\n" } else { "" }
    )
}

fn generate_react_workflow(config: &WorkflowConfig) -> String {
    let node_version = config.node_version.as_deref().unwrap_or("20");
    let deploy_step = match config.deploy_target.as_deref() {
        Some("vercel") => r#"
      - name: Deploy to Vercel
        uses: amondnet/vercel-action@v25
        with:
          vercel-token: ${{ secrets.VERCEL_TOKEN }}
          vercel-org-id: ${{ secrets.VERCEL_ORG_ID }}
          vercel-project-id: ${{ secrets.VERCEL_PROJECT_ID }}
          vercel-args: '--prod'"#,
        Some("netlify") => r#"
      - name: Deploy to Netlify
        uses: nwtgck/actions-netlify@v2
        with:
          publish-dir: './dist'
          production-branch: main
          github-token: ${{ secrets.GITHUB_TOKEN }}
          deploy-message: 'Deploy from GitHub Actions'
        env:
          NETLIFY_AUTH_TOKEN: ${{ secrets.NETLIFY_AUTH_TOKEN }}
          NETLIFY_SITE_ID: ${{ secrets.NETLIFY_SITE_ID }}"#,
        Some("github-pages") => r#"
      - name: Setup Pages
        uses: actions/configure-pages@v4

      - name: Upload artifact
        uses: actions/upload-pages-artifact@v3
        with:
          path: './dist'

      - name: Deploy to GitHub Pages
        id: deployment
        uses: actions/deploy-pages@v4"#,
        _ => "",
    };

    format!(r#"name: React CI/CD

on:
  push:
    branches: [ main, master ]
  pull_request:
    branches: [ main, master ]

jobs:
  build:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '{node_version}'
          cache: 'npm'

      - name: Install dependencies
        run: npm ci
      {}{}
      - name: Build
        run: npm run build
{deploy_step}
"#,
        if config.run_lint { "\n      - name: Lint\n        run: npm run lint\n" } else { "" },
        if config.run_tests { "\n      - name: Test\n        run: npm test\n" } else { "" }
    )
}

fn generate_node_workflow(config: &WorkflowConfig) -> String {
    let node_version = config.node_version.as_deref().unwrap_or("20");

    format!(r#"name: Node.js CI

on:
  push:
    branches: [ main, master ]
  pull_request:
    branches: [ main, master ]

jobs:
  build:
    runs-on: ubuntu-latest

    strategy:
      matrix:
        node-version: [{node_version}]

    steps:
      - uses: actions/checkout@v4

      - name: Use Node.js ${{{{ matrix.node-version }}}}
        uses: actions/setup-node@v4
        with:
          node-version: ${{{{ matrix.node-version }}}}
          cache: 'npm'

      - name: Install dependencies
        run: npm ci
      {}{}
      - name: Build
        run: npm run build --if-present
"#,
        if config.run_lint { "\n      - name: Lint\n        run: npm run lint\n" } else { "" },
        if config.run_tests { "\n      - name: Test\n        run: npm test\n" } else { "" }
    )
}

fn generate_rust_workflow(config: &WorkflowConfig) -> String {
    let rust_version = config.rust_version.as_deref().unwrap_or("stable");

    format!(r#"name: Rust CI

on:
  push:
    branches: [ main, master ]
  pull_request:
    branches: [ main, master ]

env:
  CARGO_TERM_COLOR: always

jobs:
  build:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          toolchain: {rust_version}
          components: rustfmt, clippy
      {}{}
      - name: Build
        run: cargo build --verbose
"#,
        if config.run_lint { "\n      - name: Check format\n        run: cargo fmt --all -- --check\n\n      - name: Clippy\n        run: cargo clippy -- -D warnings\n" } else { "" },
        if config.run_tests { "\n      - name: Run tests\n        run: cargo test --verbose\n" } else { "" }
    )
}

fn generate_generic_workflow(config: &WorkflowConfig) -> String {
    format!(r#"name: CI

on:
  push:
    branches: [ main, master ]
  pull_request:
    branches: [ main, master ]

jobs:
  build:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Build
        run: echo "Add your build steps here"
      {}
"#,
        if config.run_tests { "\n      - name: Test\n        run: echo \"Add your test steps here\"\n" } else { "" }
    )
}

/// Generate deployment configuration files
#[tauri::command]
async fn generate_deploy_config(workspace_path: String, platform: String) -> Result<serde_json::Value, String> {
    let (filename, content) = match platform.as_str() {
        "vercel" => ("vercel.json", serde_json::json!({
            "buildCommand": "npm run build",
            "outputDirectory": "dist",
            "framework": "vite",
            "rewrites": [
                { "source": "/(.*)", "destination": "/index.html" }
            ]
        }).to_string()),
        "netlify" => ("netlify.toml", r#"[build]
  command = "npm run build"
  publish = "dist"

[[redirects]]
  from = "/*"
  to = "/index.html"
  status = 200

[build.environment]
  NODE_VERSION = "20"
"#.to_string()),
        "github-pages" => {
            // For GitHub Pages, we need to update vite.config
            let vite_config = r#"// Add this to your vite.config.ts
export default defineConfig({
  base: '/<repository-name>/', // Replace with your repo name
  // ... other config
})
"#;
            ("_github-pages-notes.md", format!("# GitHub Pages Setup\n\n1. Enable GitHub Pages in repository settings\n2. Set source to 'GitHub Actions'\n3. Update vite.config.ts:\n\n```typescript\n{}\n```", vite_config))
        },
        _ => return Err(format!("Unknown platform: {}", platform)),
    };

    let config_path = std::path::Path::new(&workspace_path).join(&filename);
    std::fs::write(&config_path, &content)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(serde_json::json!({
        "success": true,
        "platform": platform,
        "path": config_path.to_string_lossy(),
        "content": content
    }))
}

// ============================================================================
// Crawl4AI Commands - Web Crawling & Research
// ============================================================================

/// Crawl a single URL and return the content
#[tauri::command]
async fn crawl_url(url: String, convert_to_markdown: bool) -> Result<serde_json::Value, String> {
    use crawl4ai::{HttpCrawler, HttpCrawlConfig};

    let config = HttpCrawlConfig {
        convert_to_markdown,
        filter_content: true,
        ..Default::default()
    };

    let crawler = HttpCrawler::with_config(config)
        .map_err(|e| format!("Failed to create crawler: {}", e))?;

    let result = crawler.crawl(&url).await
        .map_err(|e| format!("Crawl failed: {}", e))?;

    Ok(serde_json::json!({
        "url": result.url,
        "status_code": result.status_code,
        "title": result.title,
        "markdown": result.markdown,
        "cleaned_content": result.cleaned_content,
        "duration_ms": result.duration_ms
    }))
}

/// Crawl multiple URLs in parallel for documentation research
#[tauri::command]
async fn research_docs(urls: Vec<String>) -> Result<serde_json::Value, String> {
    use crawl4ai::{HttpCrawler, HttpCrawlConfig};

    let config = HttpCrawlConfig {
        convert_to_markdown: true,
        filter_content: true,
        timeout_secs: 30,
        ..Default::default()
    };

    let crawler = HttpCrawler::with_config(config)
        .map_err(|e| format!("Failed to create crawler: {}", e))?;

    let url_refs: Vec<&str> = urls.iter().map(|s| s.as_str()).collect();
    let results = crawler.crawl_many(&url_refs).await;

    let mut documents = Vec::new();
    let mut errors = Vec::new();

    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok(r) => documents.push(serde_json::json!({
                "url": r.url,
                "title": r.title,
                "markdown": r.markdown,
                "status_code": r.status_code
            })),
            Err(e) => errors.push(serde_json::json!({
                "url": urls.get(i).cloned().unwrap_or_default(),
                "error": e.to_string()
            })),
        }
    }

    Ok(serde_json::json!({
        "documents": documents,
        "errors": errors,
        "total_urls": urls.len(),
        "success_count": documents.len(),
        "error_count": errors.len()
    }))
}

/// Extract structured content from a URL using CSS or XPath selectors
#[tauri::command]
async fn extract_content(
    url: String,
    strategy_type: String,
    schema: serde_json::Value
) -> Result<serde_json::Value, String> {
    use crawl4ai::{HttpCrawler, HttpCrawlConfig, JsonCssExtractionStrategy, JsonXPathExtractionStrategy};

    let config = HttpCrawlConfig::default();
    let crawler = HttpCrawler::with_config(config)
        .map_err(|e| format!("Failed to create crawler: {}", e))?;

    let result = crawler.crawl(&url).await
        .map_err(|e| format!("Crawl failed: {}", e))?;

    let extracted = match strategy_type.as_str() {
        "css" => {
            let strategy = JsonCssExtractionStrategy::new(schema);
            strategy.extract(&result.html)
        },
        "xpath" => {
            let strategy = JsonXPathExtractionStrategy::new(schema);
            strategy.extract(&result.html)
        },
        _ => return Err(format!("Unknown strategy type: {}. Use 'css' or 'xpath'", strategy_type))
    };

    Ok(serde_json::json!({
        "url": result.url,
        "title": result.title,
        "extracted": extracted,
        "count": extracted.len()
    }))
}

// ============================================================================
// Knowledge Base Commands
// ============================================================================

/// Initialize or get the knowledge base
fn get_or_init_kb() -> Result<(), String> {
    let mut guard = KNOWLEDGE_BASE.lock().map_err(|_| "Failed to acquire knowledge base lock while initializing KB")?;
    if guard.is_none() {
        *guard = Some(knowledge_base::KnowledgeBase::new());
    }
    Ok(())
}

/// Add a document to the knowledge base with explicit type
#[tauri::command]
fn kb_add_document(name: String, content: String, doc_type: String) -> Result<String, String> {
    get_or_init_kb()?;
    let mut guard = KNOWLEDGE_BASE.lock().map_err(|_| "Failed to acquire knowledge base lock while adding document")?;
    let kb = guard.as_mut().ok_or("Knowledge base not initialized")?;
    let parsed_type = knowledge_base::DocumentType::from_str(&doc_type);
    let id = kb.add_document(name, content, parsed_type);
    Ok(id)
}

/// Add a document with automatic type classification
#[tauri::command]
fn kb_add_document_auto(name: String, content: String) -> Result<serde_json::Value, String> {
    get_or_init_kb()?;
    let mut guard = KNOWLEDGE_BASE.lock().map_err(|_| "Failed to acquire knowledge base lock while adding document with auto-classification")?;
    let kb = guard.as_mut().ok_or("Knowledge base not initialized")?;
    let (id, doc_type) = kb.add_document_auto(name, content);
    Ok(serde_json::json!({
        "id": id,
        "doc_type": format!("{:?}", doc_type),
        "auto_classified": true
    }))
}

/// Classify document content without adding it
#[tauri::command]
fn kb_classify_document(content: String, filename: String) -> Result<String, String> {
    let doc_type = knowledge_base::DocumentClassifier::classify(&content, &filename);
    Ok(format!("{:?}", doc_type).to_lowercase())
}

/// Add web research to the knowledge base
#[tauri::command]
fn kb_add_web_research(url: String, title: String, content: String) -> Result<String, String> {
    get_or_init_kb()?;
    let mut guard = KNOWLEDGE_BASE.lock().map_err(|_| "Failed to acquire knowledge base lock while adding web research")?;
    let kb = guard.as_mut().ok_or("Knowledge base not initialized")?;
    let id = kb.add_web_research(url, title, content);
    Ok(id)
}

/// Remove a document from the knowledge base
#[tauri::command]
fn kb_remove_document(id: String) -> Result<(), String> {
    get_or_init_kb()?;
    let mut guard = KNOWLEDGE_BASE.lock().map_err(|_| "Failed to acquire knowledge base lock while removing document")?;
    let kb = guard.as_mut().ok_or("Knowledge base not initialized")?;
    if kb.remove_document(&id) {
        Ok(())
    } else {
        Err(format!("Document with id '{}' not found", id))
    }
}

/// Get all documents from the knowledge base
#[tauri::command]
fn kb_get_documents() -> Result<Vec<knowledge_base::KnowledgeDocument>, String> {
    get_or_init_kb()?;
    let guard = KNOWLEDGE_BASE.lock().map_err(|_| "Failed to acquire knowledge base lock while getting documents")?;
    let kb = guard.as_ref().ok_or("Knowledge base not initialized")?;
    Ok(kb.get_all_documents().clone())
}

/// Compile all knowledge into a single context string for LLM consumption
#[tauri::command]
fn kb_compile_context() -> Result<String, String> {
    get_or_init_kb()?;
    let guard = KNOWLEDGE_BASE.lock().map_err(|_| "Failed to acquire knowledge base lock while compiling context")?;
    let kb = guard.as_ref().ok_or("Knowledge base not initialized")?;
    Ok(kb.compile_context())
}

/// Compile context with token budget
#[tauri::command]
fn kb_compile_context_with_budget(max_tokens: usize) -> Result<String, String> {
    get_or_init_kb()?;
    let guard = KNOWLEDGE_BASE.lock().map_err(|_| "Failed to acquire knowledge base lock while compiling context with budget")?;
    let kb = guard.as_ref().ok_or("Knowledge base not initialized")?;
    Ok(kb.compile_context_with_budget(Some(max_tokens)))
}

/// Compile context optimized for L1 Interrogator
#[tauri::command]
fn kb_compile_for_interrogator() -> Result<String, String> {
    get_or_init_kb()?;
    let guard = KNOWLEDGE_BASE.lock().map_err(|_| "Failed to acquire knowledge base lock while compiling for interrogator")?;
    let kb = guard.as_ref().ok_or("Knowledge base not initialized")?;
    Ok(kb.compile_for_interrogator())
}

/// Get knowledge base stats
#[tauri::command]
fn kb_get_stats() -> Result<serde_json::Value, String> {
    get_or_init_kb()?;
    let guard = KNOWLEDGE_BASE.lock().map_err(|_| "Failed to acquire knowledge base lock while getting stats")?;
    let kb = guard.as_ref().ok_or("Knowledge base not initialized")?;

    Ok(serde_json::json!({
        "document_count": kb.documents.len(),
        "web_research_count": kb.web_research.len(),
        "total_tokens": kb.total_tokens(),
        "documents_by_type": kb.documents.iter()
            .fold(std::collections::HashMap::<String, usize>::new(), |mut acc, doc| {
                *acc.entry(format!("{:?}", doc.doc_type)).or_insert(0) += 1;
                acc
            })
    }))
}

// ============================================================================
// Session Persistence Commands
// ============================================================================

/// Session data structure for persistence
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionData {
    /// Session ID (UUID)
    pub id: String,
    /// Session name (user-provided or auto-generated)
    pub name: String,
    /// Workspace path
    pub workspace_path: String,
    /// PRD content (if any)
    pub prd_content: Option<String>,
    /// PRD filename
    pub prd_filename: Option<String>,
    /// Conversation history (serialized)
    pub conversation_history: Vec<serde_json::Value>,
    /// Plan content (if any)
    pub plan_content: Option<String>,
    /// Knowledge base documents (serialized)
    pub kb_documents: Vec<serde_json::Value>,
    /// Current view/panel
    pub current_view: String,
    /// Timestamp when session was created
    pub created_at_ms: u64,
    /// Timestamp when session was last updated
    pub updated_at_ms: u64,
}

/// Get the sessions directory path
fn get_sessions_dir() -> Result<std::path::PathBuf, String> {
    let home = dirs::home_dir().ok_or("Could not find home directory")?;
    let sessions_dir = home.join(".cerebras-maker").join("sessions");
    std::fs::create_dir_all(&sessions_dir)
        .map_err(|e| format!("Failed to create sessions directory: {}", e))?;
    Ok(sessions_dir)
}

/// Save current session state
#[tauri::command]
fn save_session(
    session_name: String,
    workspace_path: String,
    prd_content: Option<String>,
    prd_filename: Option<String>,
    conversation_history: Vec<serde_json::Value>,
    plan_content: Option<String>,
    current_view: String,
) -> Result<SessionData, String> {
    let sessions_dir = get_sessions_dir()?;

    // Generate session ID
    let session_id = uuid::Uuid::new_v4().to_string();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    // Get KB documents
    let kb_documents = {
        let guard = KNOWLEDGE_BASE.lock().map_err(|_| "Failed to acquire KB lock while saving session")?;
        if let Some(kb) = guard.as_ref() {
            kb.documents.iter()
                .map(|doc| serde_json::json!({
                    "name": doc.name,
                    "content": doc.content,
                    "doc_type": format!("{:?}", doc.doc_type),
                    "metadata": doc.metadata,
                    "auto_classified": doc.auto_classified,
                    "word_count": doc.word_count,
                }))
                .collect()
        } else {
            Vec::new()
        }
    };

    let session = SessionData {
        id: session_id.clone(),
        name: session_name,
        workspace_path,
        prd_content,
        prd_filename,
        conversation_history,
        plan_content,
        kb_documents,
        current_view,
        created_at_ms: now,
        updated_at_ms: now,
    };

    // Save to file
    let session_file = sessions_dir.join(format!("{}.json", session_id));
    let json = serde_json::to_string_pretty(&session)
        .map_err(|e| format!("Failed to serialize session: {}", e))?;
    std::fs::write(&session_file, json)
        .map_err(|e| format!("Failed to write session file: {}", e))?;

    Ok(session)
}

/// Update an existing session
#[tauri::command]
fn update_session(
    session_id: String,
    prd_content: Option<String>,
    conversation_history: Vec<serde_json::Value>,
    plan_content: Option<String>,
    current_view: String,
) -> Result<SessionData, String> {
    let sessions_dir = get_sessions_dir()?;
    let session_file = sessions_dir.join(format!("{}.json", session_id));

    // Load existing session
    let content = std::fs::read_to_string(&session_file)
        .map_err(|e| format!("Failed to read session file: {}", e))?;
    let mut session: SessionData = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse session: {}", e))?;

    // Update fields
    session.prd_content = prd_content;
    session.conversation_history = conversation_history;
    session.plan_content = plan_content;
    session.current_view = current_view;
    session.updated_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    // Update KB documents
    session.kb_documents = {
        let guard = KNOWLEDGE_BASE.lock().map_err(|_| "Failed to acquire KB lock while restoring session")?;
        if let Some(kb) = guard.as_ref() {
            kb.documents.iter()
                .map(|doc| serde_json::json!({
                    "name": doc.name,
                    "content": doc.content,
                    "doc_type": format!("{:?}", doc.doc_type),
                    "metadata": doc.metadata,
                    "auto_classified": doc.auto_classified,
                    "word_count": doc.word_count,
                }))
                .collect()
        } else {
            Vec::new()
        }
    };

    // Save updated session
    let json = serde_json::to_string_pretty(&session)
        .map_err(|e| format!("Failed to serialize session: {}", e))?;
    std::fs::write(&session_file, json)
        .map_err(|e| format!("Failed to write session file: {}", e))?;

    Ok(session)
}

/// Load a session by ID
#[tauri::command]
fn load_session(session_id: String) -> Result<SessionData, String> {
    let sessions_dir = get_sessions_dir()?;
    let session_file = sessions_dir.join(format!("{}.json", session_id));

    let content = std::fs::read_to_string(&session_file)
        .map_err(|e| format!("Failed to read session file: {}", e))?;
    let session: SessionData = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse session: {}", e))?;

    Ok(session)
}

/// List all saved sessions
#[tauri::command]
fn list_sessions() -> Result<Vec<serde_json::Value>, String> {
    let sessions_dir = get_sessions_dir()?;

    let mut sessions = Vec::new();

    for entry in std::fs::read_dir(&sessions_dir)
        .map_err(|e| format!("Failed to read sessions directory: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "json") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(session) = serde_json::from_str::<SessionData>(&content) {
                    sessions.push(serde_json::json!({
                        "id": session.id,
                        "name": session.name,
                        "workspace_path": session.workspace_path,
                        "created_at_ms": session.created_at_ms,
                        "updated_at_ms": session.updated_at_ms,
                        "has_prd": session.prd_content.is_some(),
                        "has_plan": session.plan_content.is_some(),
                        "message_count": session.conversation_history.len(),
                        "kb_document_count": session.kb_documents.len(),
                    }));
                }
            }
        }
    }

    // Sort by updated_at_ms descending (most recent first)
    sessions.sort_by(|a, b| {
        let a_time = a["updated_at_ms"].as_u64().unwrap_or(0);
        let b_time = b["updated_at_ms"].as_u64().unwrap_or(0);
        b_time.cmp(&a_time)
    });

    Ok(sessions)
}

/// Delete a session by ID
#[tauri::command]
fn delete_session(session_id: String) -> Result<(), String> {
    let sessions_dir = get_sessions_dir()?;
    let session_file = sessions_dir.join(format!("{}.json", session_id));

    std::fs::remove_file(&session_file)
        .map_err(|e| format!("Failed to delete session file: {}", e))?;

    Ok(())
}

// =============================================================================
// Thread Management Commands - Agent Unblocker System
// =============================================================================

/// Create a new thread (called by agent when blocked)
#[tauri::command]
fn create_thread(
    thread_type: ThreadType,
    priority: ThreadPriority,
    title: String,
    agent_name: String,
    initial_message: String,
    task_id: Option<String>,
    is_blocking: bool,
) -> Result<Thread, String> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let thread_id = format!("thread_{}", timestamp);
    let message_id = format!("msg_{}", timestamp);

    let initial_msg = ThreadMessage {
        id: message_id,
        thread_id: thread_id.clone(),
        role: "agent".to_string(),
        agent_name: Some(agent_name.clone()),
        content: initial_message,
        attachments: None,
        timestamp_ms: timestamp,
    };

    let thread = Thread {
        id: thread_id,
        thread_type,
        status: ThreadStatus::Open,
        priority,
        title,
        agent_name,
        task_id,
        created_at_ms: timestamp,
        updated_at_ms: timestamp,
        resolved_at_ms: None,
        messages: vec![initial_msg],
        is_blocking,
    };

    let mut threads = THREADS.lock().map_err(|e| e.to_string())?;
    threads.push(thread.clone());

    Ok(thread)
}

/// List all threads with optional status filter
#[tauri::command]
fn list_threads(status_filter: Option<String>) -> Result<Vec<Thread>, String> {
    let threads = THREADS.lock().map_err(|e| e.to_string())?;

    let filtered: Vec<Thread> = if let Some(status) = status_filter {
        threads.iter()
            .filter(|t| match &t.status {
                ThreadStatus::Open => status == "open",
                ThreadStatus::Resolved => status == "resolved",
                ThreadStatus::Pending => status == "pending",
            })
            .cloned()
            .collect()
    } else {
        threads.clone()
    };

    Ok(filtered)
}

/// Get a single thread by ID
#[tauri::command]
fn get_thread(thread_id: String) -> Result<Thread, String> {
    let threads = THREADS.lock().map_err(|e| e.to_string())?;
    threads.iter()
        .find(|t| t.id == thread_id)
        .cloned()
        .ok_or_else(|| format!("Thread not found: {}", thread_id))
}

/// Reply to a thread (called by human)
#[tauri::command]
fn reply_to_thread(
    thread_id: String,
    content: String,
    attachments: Option<Vec<ThreadAttachment>>,
    resolve: bool,
) -> Result<Thread, String> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let mut threads = THREADS.lock().map_err(|e| e.to_string())?;
    let thread = threads.iter_mut()
        .find(|t| t.id == thread_id)
        .ok_or_else(|| format!("Thread not found: {}", thread_id))?;

    let message = ThreadMessage {
        id: format!("msg_{}", timestamp),
        thread_id: thread_id.clone(),
        role: "human".to_string(),
        agent_name: None,
        content,
        attachments,
        timestamp_ms: timestamp,
    };

    thread.messages.push(message);
    thread.updated_at_ms = timestamp;
    thread.is_blocking = false; // Human responded, agent unblocked

    if resolve {
        thread.status = ThreadStatus::Resolved;
        thread.resolved_at_ms = Some(timestamp);
    } else {
        thread.status = ThreadStatus::Pending;
    }

    Ok(thread.clone())
}

/// Resolve a thread
#[tauri::command]
fn resolve_thread(thread_id: String) -> Result<Thread, String> {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let mut threads = THREADS.lock().map_err(|e| e.to_string())?;
    let thread = threads.iter_mut()
        .find(|t| t.id == thread_id)
        .ok_or_else(|| format!("Thread not found: {}", thread_id))?;

    thread.status = ThreadStatus::Resolved;
    thread.resolved_at_ms = Some(timestamp);
    thread.updated_at_ms = timestamp;
    thread.is_blocking = false;

    Ok(thread.clone())
}

/// Get all threads that are currently blocking agents
#[tauri::command]
fn get_blocking_threads() -> Result<Vec<Thread>, String> {
    let threads = THREADS.lock().map_err(|e| e.to_string())?;
    let blocking: Vec<Thread> = threads.iter()
        .filter(|t| t.is_blocking)
        .cloned()
        .collect();
    Ok(blocking)
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
            check_proposed_changes,
            analyze_topology,
            // MAKER runtime commands
            init_runtime,
            execute_script,
            get_execution_log,
            get_cwd,
            // Execution metrics commands
            get_execution_metrics,
            update_execution_metrics,
            record_atom_spawned,
            record_atom_completed,
            record_shadow_commit,
            reset_execution_metrics,
            // Voting state commands
            get_voting_state,
            start_voting,
            add_voting_candidate,
            record_vote,
            complete_voting,
            clear_voting_state,
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
            get_atom_types,
            // RLM (Recursive Language Model) commands
            init_rlm_store,
            rlm_load_context,
            rlm_peek_context,
            rlm_context_length,
            rlm_chunk_context,
            rlm_regex_filter,
            execute_rlm_atom,
            get_rlm_trajectory,
            clear_rlm_trajectory,
            rlm_list_contexts,
            rlm_clear_context,
            rlm_clear_all,
            get_rlm_config,
            should_use_rlm,
            // Crawl4AI commands
            crawl_url,
            research_docs,
            extract_content,
            // GitHub integration commands
            git_init,
            git_add_remote,
            git_get_remotes,
            git_push,
            git_current_branch,
            git_status,
            git_clone,
            git_add,
            git_commit,
            git_branch,
            git_list_branches,
            git_pull,
            git_diff,
            git_log,
            // GitHub Actions & Deployment
            generate_github_workflow,
            generate_deploy_config,
            // Multi-file Edit Validation commands
            validate_multi_file_edit,
            preview_edit_impact,
            // Test Generation & Execution commands
            detect_test_framework,
            run_tests,
            generate_tests,
            // Knowledge Base commands
            kb_add_document,
            kb_add_document_auto,
            kb_classify_document,
            kb_add_web_research,
            kb_remove_document,
            kb_get_documents,
            kb_compile_context,
            kb_compile_context_with_budget,
            kb_compile_for_interrogator,
            kb_get_stats,
            // Session Persistence commands
            save_session,
            update_session,
            load_session,
            list_sessions,
            delete_session,
            // Thread Management commands (Agent Unblocker)
            create_thread,
            list_threads,
            get_thread,
            reply_to_thread,
            resolve_thread,
            get_blocking_threads
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    // ========================================================================
    // Git Integration Tests
    // ========================================================================

    #[tokio::test]
    async fn test_git_init_success() {
        let dir = tempdir().unwrap();
        let workspace_path = dir.path().to_str().unwrap().to_string();

        let result = git_init(workspace_path.clone()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Repository initialized");

        // Verify .git directory was created
        assert!(dir.path().join(".git").exists());
    }

    #[tokio::test]
    async fn test_git_init_invalid_path() {
        let result = git_init("/nonexistent/path/that/does/not/exist".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_git_add_remote_success() {
        let dir = tempdir().unwrap();
        let workspace_path = dir.path().to_str().unwrap().to_string();

        // Initialize repo first
        git_init(workspace_path.clone()).await.unwrap();

        // Add remote
        let result = git_add_remote(
            workspace_path.clone(),
            "origin".to_string(),
            "https://github.com/example/repo.git".to_string(),
        )
        .await;

        assert!(result.is_ok());
        assert!(result.unwrap().contains("origin"));
    }

    #[tokio::test]
    async fn test_git_add_remote_updates_existing() {
        let dir = tempdir().unwrap();
        let workspace_path = dir.path().to_str().unwrap().to_string();

        // Initialize repo first
        git_init(workspace_path.clone()).await.unwrap();

        // Add remote
        git_add_remote(
            workspace_path.clone(),
            "origin".to_string(),
            "https://github.com/example/repo.git".to_string(),
        )
        .await
        .unwrap();

        // Try to add same remote again (should update)
        let result = git_add_remote(
            workspace_path.clone(),
            "origin".to_string(),
            "https://github.com/example/other-repo.git".to_string(),
        )
        .await;

        assert!(result.is_ok());
        assert!(result.unwrap().contains("updated"));
    }

    #[tokio::test]
    async fn test_git_get_remotes_empty() {
        let dir = tempdir().unwrap();
        let workspace_path = dir.path().to_str().unwrap().to_string();

        // Initialize repo first
        git_init(workspace_path.clone()).await.unwrap();

        let result = git_get_remotes(workspace_path).await;
        assert!(result.is_ok());

        let json = result.unwrap();
        let remotes = json["remotes"].as_array().unwrap();
        assert!(remotes.is_empty());
    }

    #[tokio::test]
    async fn test_git_get_remotes_with_remote() {
        let dir = tempdir().unwrap();
        let workspace_path = dir.path().to_str().unwrap().to_string();

        // Initialize repo and add remote
        git_init(workspace_path.clone()).await.unwrap();
        git_add_remote(
            workspace_path.clone(),
            "origin".to_string(),
            "https://github.com/example/repo.git".to_string(),
        )
        .await
        .unwrap();

        let result = git_get_remotes(workspace_path).await;
        assert!(result.is_ok());

        let json = result.unwrap();
        let remotes = json["remotes"].as_array().unwrap();
        assert!(!remotes.is_empty());
        assert_eq!(remotes[0]["name"], "origin");
    }

    #[tokio::test]
    async fn test_git_current_branch_default() {
        let dir = tempdir().unwrap();
        let workspace_path = dir.path().to_str().unwrap().to_string();

        // Initialize repo
        git_init(workspace_path.clone()).await.unwrap();

        // Configure git user for commit
        std::process::Command::new("git")
            .current_dir(&workspace_path)
            .args(["config", "user.email", "test@example.com"])
            .output()
            .unwrap();
        std::process::Command::new("git")
            .current_dir(&workspace_path)
            .args(["config", "user.name", "Test User"])
            .output()
            .unwrap();

        // Create initial commit so we have a branch
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();
        std::process::Command::new("git")
            .current_dir(&workspace_path)
            .args(["add", "."])
            .output()
            .unwrap();
        std::process::Command::new("git")
            .current_dir(&workspace_path)
            .args(["commit", "-m", "initial"])
            .output()
            .unwrap();

        let result = git_current_branch(workspace_path).await;
        assert!(result.is_ok());
        // Default branch is either "master" or "main" depending on git config
        let branch = result.unwrap();
        assert!(branch == "master" || branch == "main");
    }

    #[tokio::test]
    async fn test_git_status_clean() {
        let dir = tempdir().unwrap();
        let workspace_path = dir.path().to_str().unwrap().to_string();

        // Initialize repo
        git_init(workspace_path.clone()).await.unwrap();

        // Configure git user for commit
        std::process::Command::new("git")
            .current_dir(&workspace_path)
            .args(["config", "user.email", "test@example.com"])
            .output()
            .unwrap();
        std::process::Command::new("git")
            .current_dir(&workspace_path)
            .args(["config", "user.name", "Test User"])
            .output()
            .unwrap();

        // Create initial commit
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();
        std::process::Command::new("git")
            .current_dir(&workspace_path)
            .args(["add", "."])
            .output()
            .unwrap();
        std::process::Command::new("git")
            .current_dir(&workspace_path)
            .args(["commit", "-m", "initial"])
            .output()
            .unwrap();

        let result = git_status(workspace_path).await;
        assert!(result.is_ok());

        let json = result.unwrap();
        assert!(json["is_clean"].as_bool().unwrap());
        assert_eq!(json["change_count"].as_i64().unwrap(), 0);
    }

    #[tokio::test]
    async fn test_git_status_dirty() {
        let dir = tempdir().unwrap();
        let workspace_path = dir.path().to_str().unwrap().to_string();

        // Initialize repo
        git_init(workspace_path.clone()).await.unwrap();

        // Configure git user for commit
        std::process::Command::new("git")
            .current_dir(&workspace_path)
            .args(["config", "user.email", "test@example.com"])
            .output()
            .unwrap();
        std::process::Command::new("git")
            .current_dir(&workspace_path)
            .args(["config", "user.name", "Test User"])
            .output()
            .unwrap();

        // Create initial commit
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();
        std::process::Command::new("git")
            .current_dir(&workspace_path)
            .args(["add", "."])
            .output()
            .unwrap();
        std::process::Command::new("git")
            .current_dir(&workspace_path)
            .args(["commit", "-m", "initial"])
            .output()
            .unwrap();

        // Modify file to make repo dirty
        fs::write(&file_path, "modified content").unwrap();

        let result = git_status(workspace_path).await;
        assert!(result.is_ok());

        let json = result.unwrap();
        assert!(!json["is_clean"].as_bool().unwrap());
        assert!(json["change_count"].as_i64().unwrap() > 0);
    }

    #[tokio::test]
    async fn test_git_clone_success() {
        let dir = tempdir().unwrap();
        let target_path = dir.path().join("cloned-repo");
        let target_path_str = target_path.to_str().unwrap().to_string();

        // Clone a small public repo
        let result = git_clone(
            "https://github.com/octocat/Hello-World.git".to_string(),
            target_path_str.clone(),
        )
        .await;

        assert!(result.is_ok());
        assert!(target_path.join(".git").exists());
    }

    #[tokio::test]
    async fn test_git_clone_invalid_url() {
        let dir = tempdir().unwrap();
        let target_path = dir.path().join("cloned-repo");
        let target_path_str = target_path.to_str().unwrap().to_string();

        let result = git_clone(
            "https://github.com/nonexistent-user-12345/nonexistent-repo-67890.git".to_string(),
            target_path_str,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_git_push_no_remote() {
        let dir = tempdir().unwrap();
        let workspace_path = dir.path().to_str().unwrap().to_string();

        // Initialize repo
        git_init(workspace_path.clone()).await.unwrap();

        // Try to push without remote (should fail)
        let result = git_push(
            workspace_path,
            "origin".to_string(),
            "main".to_string(),
            false,
        )
        .await;

        assert!(result.is_err());
    }

    // ========================================================================
    // Crawl4AI Tests
    // ========================================================================

    #[tokio::test]
    async fn test_crawl_url_success() {
        let result = crawl_url("https://example.com".to_string(), true).await;

        assert!(result.is_ok());
        let json = result.unwrap();
        assert_eq!(json["url"], "https://example.com");
        assert_eq!(json["status_code"], 200);
        assert!(json["title"].as_str().is_some());
        assert!(json["duration_ms"].as_u64().is_some());
    }

    #[tokio::test]
    async fn test_crawl_url_without_markdown() {
        let result = crawl_url("https://example.com".to_string(), false).await;

        assert!(result.is_ok());
        let json = result.unwrap();
        assert_eq!(json["url"], "https://example.com");
        assert_eq!(json["status_code"], 200);
    }

    #[tokio::test]
    async fn test_crawl_url_invalid() {
        let result = crawl_url("https://this-domain-definitely-does-not-exist-12345.com".to_string(), true).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_research_docs_success() {
        let urls = vec![
            "https://example.com".to_string(),
            "https://example.org".to_string(),
        ];

        let result = research_docs(urls).await;

        assert!(result.is_ok());
        let json = result.unwrap();
        assert_eq!(json["total_urls"], 2);
        assert!(json["success_count"].as_i64().unwrap() > 0);
        assert!(json["documents"].as_array().is_some());
    }

    #[tokio::test]
    async fn test_research_docs_empty() {
        let urls: Vec<String> = vec![];

        let result = research_docs(urls).await;

        assert!(result.is_ok());
        let json = result.unwrap();
        assert_eq!(json["total_urls"], 0);
        assert_eq!(json["success_count"], 0);
    }

    #[tokio::test]
    async fn test_research_docs_with_errors() {
        let urls = vec![
            "https://example.com".to_string(),
            "https://this-domain-definitely-does-not-exist-12345.com".to_string(),
        ];

        let result = research_docs(urls).await;

        assert!(result.is_ok());
        let json = result.unwrap();
        assert_eq!(json["total_urls"], 2);
        // At least one should succeed (example.com)
        assert!(json["success_count"].as_i64().unwrap() >= 1);
        // At least one should fail
        assert!(json["error_count"].as_i64().unwrap() >= 1);
    }

    #[tokio::test]
    async fn test_extract_content_css_success() {
        let schema = serde_json::json!({
            "baseSelector": "body",
            "fields": [
                {"name": "heading", "selector": "h1", "type": "text"}
            ]
        });

        let result = extract_content(
            "https://example.com".to_string(),
            "css".to_string(),
            schema,
        )
        .await;

        assert!(result.is_ok());
        let json = result.unwrap();
        assert_eq!(json["url"], "https://example.com");
        assert!(json["extracted"].as_array().is_some());
    }

    #[tokio::test]
    async fn test_extract_content_xpath_success() {
        let schema = serde_json::json!({
            "baseSelector": "//body",
            "fields": [
                {"name": "heading", "selector": "h1", "type": "text"}
            ]
        });

        let result = extract_content(
            "https://example.com".to_string(),
            "xpath".to_string(),
            schema,
        )
        .await;

        assert!(result.is_ok());
        let json = result.unwrap();
        assert_eq!(json["url"], "https://example.com");
        assert!(json["extracted"].as_array().is_some());
    }

    #[tokio::test]
    async fn test_extract_content_invalid_strategy() {
        let schema = serde_json::json!({
            "baseSelector": "body",
            "fields": []
        });

        let result = extract_content(
            "https://example.com".to_string(),
            "invalid_strategy".to_string(),
            schema,
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown strategy type"));
    }

    #[tokio::test]
    async fn test_extract_content_invalid_url() {
        let schema = serde_json::json!({
            "baseSelector": "body",
            "fields": []
        });

        let result = extract_content(
            "https://this-domain-definitely-does-not-exist-12345.com".to_string(),
            "css".to_string(),
            schema,
        )
        .await;

        assert!(result.is_err());
    }
}
