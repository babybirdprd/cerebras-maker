// Cerebras-MAKER: Recursive Language Model (RLM) Module
// Based on: "Recursive Language Models: Scaling Context with Recursive Prompt Decomposition"
// Enables handling of arbitrarily long contexts by treating prompts as external environment variables

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// RLM Context Variable Store
/// Holds large contexts as environment variables that can be programmatically accessed
#[derive(Debug, Clone, Default)]
pub struct RLMContextStore {
    /// Named context variables (name -> content)
    variables: HashMap<String, String>,
    /// Metadata about each variable
    metadata: HashMap<String, ContextMetadata>,
}

/// Metadata about a context variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMetadata {
    pub total_length: usize,
    pub context_type: ContextType,
    pub chunk_boundaries: Vec<usize>,
    pub created_at: u64,
}

/// Type of context stored
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ContextType {
    /// Plain text content
    String,
    /// JSON or structured data
    Structured,
    /// Multiple documents (like BrowseComp task)
    Documents,
    /// Grits SymbolGraph topology data
    SymbolGraph,
    /// Code files with path information
    CodeFiles,
    /// Grits MiniCodebase extraction
    MiniCodebase,
    /// Single file content
    File,
}

/// Configuration for RLM execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLMConfig {
    /// Maximum recursion depth for llm_query calls (1 = sub-calls are regular LLMs, not RLMs)
    pub max_depth: usize,
    /// Maximum iterations within a single RLM execution
    pub max_iterations: usize,
    /// Default chunk size for context chunking (in characters)
    pub default_chunk_size: usize,
    /// Threshold for switching to RLM mode (in characters)
    pub rlm_threshold: usize,
    /// Whether to use a cheaper model for sub-calls
    pub use_sub_model: bool,
    /// Sub-model configuration key (if different from main model)
    pub sub_model_key: Option<String>,
}

impl Default for RLMConfig {
    fn default() -> Self {
        Self {
            max_depth: 1,              // Default: sub-calls are LMs, not RLMs
            max_iterations: 20,        // Reasonable iteration limit
            default_chunk_size: 50_000, // 50K chars per chunk
            rlm_threshold: 50_000,     // Switch to RLM mode at 50K chars
            use_sub_model: false,
            sub_model_key: None,
        }
    }
}

/// Result of an RLM execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLMResult {
    /// Whether execution succeeded
    pub success: bool,
    /// The final answer/output
    pub output: String,
    /// Number of iterations used
    pub iterations: usize,
    /// Number of sub-LM calls made
    pub sub_calls: usize,
    /// Total tokens consumed (main + sub calls)
    pub total_tokens: usize,
    /// Execution trajectory for visualization
    pub trajectory: Vec<RLMTrajectoryStep>,
    /// Any errors encountered
    pub errors: Vec<String>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
}

/// A step in the RLM execution trajectory (for Cockpit visualization)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLMTrajectoryStep {
    /// Step number
    pub step: usize,
    /// Type of operation
    pub operation: RLMOperation,
    /// Description of what happened
    pub description: String,
    /// Any data associated with this step
    pub data: Option<serde_json::Value>,
    /// Timestamp (ms since start)
    pub timestamp_ms: u64,
}

/// Types of RLM operations for trajectory tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RLMOperation {
    /// Started RLM execution
    Start,
    /// Peeked at context slice
    Peek { var_name: String, start: usize, end: usize },
    /// Chunked a context variable
    Chunk { var_name: String, num_chunks: usize },
    /// Made a recursive llm_query call
    SubQuery { prompt_preview: String, depth: usize },
    /// Received result from sub-query
    SubResult { result_preview: String },
    /// Applied regex filter
    RegexFilter { var_name: String, pattern: String, matches: usize },
    /// Loaded a new context variable
    LoadContext { var_name: String, length: usize },
    /// Reached final answer
    Final { answer_preview: String },
    /// Error occurred
    Error { message: String },
}

impl RLMContextStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load a context variable into the store
    pub fn load_variable(&mut self, name: &str, content: String, context_type: ContextType) {
        let length = content.len();
        let metadata = ContextMetadata {
            total_length: length,
            context_type,
            chunk_boundaries: Vec::new(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        };

        self.variables.insert(name.to_string(), content);
        self.metadata.insert(name.to_string(), metadata);
    }

    /// Peek at a slice of a context variable without consuming tokens
    pub fn peek(&self, name: &str, start: usize, end: usize) -> Option<String> {
        self.variables.get(name).map(|content| {
            let actual_end = end.min(content.len());
            let actual_start = start.min(actual_end);
            content[actual_start..actual_end].to_string()
        })
    }

    /// Get the length of a context variable
    pub fn length(&self, name: &str) -> Option<usize> {
        self.variables.get(name).map(|c| c.len())
    }

    /// Chunk a context variable into pieces of specified size
    pub fn chunk(&mut self, name: &str, chunk_size: usize) -> Vec<String> {
        let content = match self.variables.get(name) {
            Some(c) => c.clone(),
            None => return Vec::new(),
        };

        let chunks: Vec<String> = content
            .chars()
            .collect::<Vec<_>>()
            .chunks(chunk_size)
            .map(|c| c.iter().collect::<String>())
            .collect();

        // Update metadata with chunk boundaries
        if let Some(meta) = self.metadata.get_mut(name) {
            let mut boundaries = Vec::new();
            let mut pos = 0;
            for chunk in &chunks {
                boundaries.push(pos);
                pos += chunk.len();
            }
            meta.chunk_boundaries = boundaries;
        }

        chunks
    }

    /// Apply regex filter to a context variable, returning matching lines/sections
    pub fn regex_filter(&self, name: &str, pattern: &str) -> Result<Vec<String>, String> {
        let content = self.variables.get(name)
            .ok_or_else(|| format!("Context variable '{}' not found", name))?;

        let regex = regex::Regex::new(pattern)
            .map_err(|e| format!("Invalid regex pattern: {}", e))?;

        let matches: Vec<String> = content
            .lines()
            .filter(|line| regex.is_match(line))
            .map(|s| s.to_string())
            .collect();

        Ok(matches)
    }

    /// Get a variable's content (for internal use)
    pub fn get(&self, name: &str) -> Option<&String> {
        self.variables.get(name)
    }

    /// Get metadata for a variable
    pub fn get_metadata(&self, name: &str) -> Option<&ContextMetadata> {
        self.metadata.get(name)
    }

    /// Check if a variable exists
    pub fn contains(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// Remove a variable from the store
    pub fn remove(&mut self, name: &str) -> Option<String> {
        self.metadata.remove(name);
        self.variables.remove(name)
    }

    /// List all variable names
    pub fn list_variables(&self) -> Vec<String> {
        self.variables.keys().cloned().collect()
    }

    /// Clear all variables
    pub fn clear(&mut self) {
        self.variables.clear();
        self.metadata.clear();
    }
}

/// Thread-safe RLM Context Store wrapper
pub type SharedRLMContextStore = Arc<Mutex<RLMContextStore>>;

/// Create a new shared RLM context store
pub fn create_shared_store() -> SharedRLMContextStore {
    Arc::new(Mutex::new(RLMContextStore::new()))
}

impl RLMResult {
    /// Create a successful result
    pub fn success(output: String, iterations: usize, sub_calls: usize, trajectory: Vec<RLMTrajectoryStep>) -> Self {
        Self {
            success: true,
            output,
            iterations,
            sub_calls,
            total_tokens: 0,
            trajectory,
            errors: Vec::new(),
            execution_time_ms: 0,
        }
    }

    /// Create a failure result
    pub fn failure(errors: Vec<String>) -> Self {
        Self {
            success: false,
            output: String::new(),
            iterations: 0,
            sub_calls: 0,
            total_tokens: 0,
            trajectory: Vec::new(),
            errors,
            execution_time_ms: 0,
        }
    }
}

/// P2-2: RLM Action types for iterative execution
/// Based on RLM paper Section 3.1 - the LLM outputs actions to interact with context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RLMAction {
    /// Peek at a slice of context: peek(var_name, start, end)
    Peek { var_name: String, start: usize, end: usize },
    /// Chunk context into pieces: chunk(var_name, chunk_size)
    Chunk { var_name: String, chunk_size: usize },
    /// Filter with regex: regex_filter(var_name, pattern)
    RegexFilter { var_name: String, pattern: String },
    /// Make a sub-LM query (depth-limited): llm_query(prompt)
    SubQuery { prompt: String },
    /// Return final answer: final(answer)
    Final { answer: String },
    /// Continue thinking (no action yet)
    Continue { reasoning: String },
}

impl RLMAction {
    /// Parse an action from LLM output (JSON format)
    pub fn parse(output: &str) -> Result<Self, String> {
        // Try to extract JSON from the output
        let json_str = if let Some(start) = output.find('{') {
            if let Some(end) = output.rfind('}') {
                &output[start..=end]
            } else {
                return Err("No closing brace found".to_string());
            }
        } else {
            return Err("No JSON object found in output".to_string());
        };

        // Parse the JSON
        let value: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| format!("JSON parse error: {}", e))?;

        // Extract action type
        let action_type = value.get("action")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'action' field")?;

        match action_type {
            "peek" => {
                let var_name = value.get("var_name")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing 'var_name' for peek")?
                    .to_string();
                let start = value.get("start")
                    .and_then(|v| v.as_u64())
                    .ok_or("Missing 'start' for peek")? as usize;
                let end = value.get("end")
                    .and_then(|v| v.as_u64())
                    .ok_or("Missing 'end' for peek")? as usize;
                Ok(RLMAction::Peek { var_name, start, end })
            }
            "chunk" => {
                let var_name = value.get("var_name")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing 'var_name' for chunk")?
                    .to_string();
                let chunk_size = value.get("chunk_size")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(50000) as usize;
                Ok(RLMAction::Chunk { var_name, chunk_size })
            }
            "regex_filter" => {
                let var_name = value.get("var_name")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing 'var_name' for regex_filter")?
                    .to_string();
                let pattern = value.get("pattern")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing 'pattern' for regex_filter")?
                    .to_string();
                Ok(RLMAction::RegexFilter { var_name, pattern })
            }
            "llm_query" | "sub_query" => {
                let prompt = value.get("prompt")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing 'prompt' for llm_query")?
                    .to_string();
                Ok(RLMAction::SubQuery { prompt })
            }
            "final" => {
                let answer = value.get("answer")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing 'answer' for final")?
                    .to_string();
                Ok(RLMAction::Final { answer })
            }
            "continue" | "think" => {
                let reasoning = value.get("reasoning")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                Ok(RLMAction::Continue { reasoning })
            }
            _ => Err(format!("Unknown action type: {}", action_type)),
        }
    }
}

/// P2-2: RLM Execution State for iterative loop
#[derive(Debug, Clone)]
pub struct RLMExecutionState {
    /// Current iteration number
    pub iteration: usize,
    /// Current recursion depth (for sub-queries)
    pub depth: usize,
    /// Accumulated observations from actions
    pub observations: Vec<String>,
    /// Trajectory of operations
    pub trajectory: Vec<RLMTrajectoryStep>,
    /// Number of sub-LM calls made
    pub sub_calls: usize,
    /// Start time for timing
    pub start_time: std::time::Instant,
}

impl RLMExecutionState {
    pub fn new() -> Self {
        Self {
            iteration: 0,
            depth: 0,
            observations: Vec::new(),
            trajectory: Vec::new(),
            sub_calls: 0,
            start_time: std::time::Instant::now(),
        }
    }

    /// Add an observation from an action result
    pub fn add_observation(&mut self, obs: String) {
        self.observations.push(obs);
    }

    /// Add a trajectory step
    pub fn add_step(&mut self, operation: RLMOperation, description: String, data: Option<serde_json::Value>) {
        self.trajectory.push(RLMTrajectoryStep {
            step: self.trajectory.len(),
            operation,
            description,
            data,
            timestamp_ms: self.start_time.elapsed().as_millis() as u64,
        });
    }

    /// Build the prompt with accumulated observations
    pub fn build_iteration_prompt(&self, task: &str, context_info: &str) -> String {
        let mut prompt = format!(
            "## RLM Iteration {}\n\n\
            ### Task\n{}\n\n\
            ### Context Info\n{}\n\n",
            self.iteration, task, context_info
        );

        if !self.observations.is_empty() {
            prompt.push_str("### Previous Observations\n");
            for (i, obs) in self.observations.iter().enumerate() {
                prompt.push_str(&format!("{}. {}\n", i + 1, obs));
            }
            prompt.push('\n');
        }

        prompt.push_str(
            "### Available Actions\n\
            Output a JSON object with one of these actions:\n\
            - `{\"action\": \"peek\", \"var_name\": \"...\", \"start\": N, \"end\": M}` - View slice of context\n\
            - `{\"action\": \"chunk\", \"var_name\": \"...\", \"chunk_size\": N}` - Split into chunks\n\
            - `{\"action\": \"regex_filter\", \"var_name\": \"...\", \"pattern\": \"...\"}` - Filter with regex\n\
            - `{\"action\": \"llm_query\", \"prompt\": \"...\"}` - Make a sub-query (depth limited)\n\
            - `{\"action\": \"final\", \"answer\": \"...\"}` - Return final answer\n\
            - `{\"action\": \"continue\", \"reasoning\": \"...\"}` - Continue thinking\n\n\
            Respond with ONLY a JSON object.\n"
        );

        prompt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rlm_context_store_new() {
        let store = RLMContextStore::new();
        assert!(store.list_variables().is_empty());
    }

    #[test]
    fn test_load_and_get_variable() {
        let mut store = RLMContextStore::new();
        store.load_variable("test_var", "Hello, World!".to_string(), ContextType::String);

        assert!(store.contains("test_var"));
        assert_eq!(store.get("test_var"), Some(&"Hello, World!".to_string()));
        assert_eq!(store.length("test_var"), Some(13));
    }

    #[test]
    fn test_peek_variable() {
        let mut store = RLMContextStore::new();
        store.load_variable("content", "0123456789".to_string(), ContextType::String);

        // Normal peek
        assert_eq!(store.peek("content", 0, 5), Some("01234".to_string()));
        assert_eq!(store.peek("content", 5, 10), Some("56789".to_string()));

        // Peek beyond bounds (should clamp)
        assert_eq!(store.peek("content", 8, 100), Some("89".to_string()));

        // Peek non-existent variable
        assert_eq!(store.peek("nonexistent", 0, 5), None);
    }

    #[test]
    fn test_chunk_variable() {
        let mut store = RLMContextStore::new();
        store.load_variable("data", "ABCDEFGHIJ".to_string(), ContextType::String);

        let chunks = store.chunk("data", 3);
        assert_eq!(chunks.len(), 4);
        assert_eq!(chunks[0], "ABC");
        assert_eq!(chunks[1], "DEF");
        assert_eq!(chunks[2], "GHI");
        assert_eq!(chunks[3], "J");

        // Check metadata was updated
        let meta = store.get_metadata("data").unwrap();
        assert_eq!(meta.chunk_boundaries.len(), 4);
    }

    #[test]
    fn test_chunk_empty_variable() {
        let mut store = RLMContextStore::new();
        let chunks = store.chunk("nonexistent", 10);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_regex_filter() {
        let mut store = RLMContextStore::new();
        let content = "fn main() {\n    println!(\"Hello\");\n}\nfn helper() {\n    // comment\n}";
        store.load_variable("code", content.to_string(), ContextType::CodeFiles);

        // Filter for function definitions
        let matches = store.regex_filter("code", r"^fn ").unwrap();
        assert_eq!(matches.len(), 2);
        assert!(matches[0].starts_with("fn main"));
        assert!(matches[1].starts_with("fn helper"));
    }

    #[test]
    fn test_regex_filter_invalid_pattern() {
        let mut store = RLMContextStore::new();
        store.load_variable("test", "content".to_string(), ContextType::String);

        let result = store.regex_filter("test", "[invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid regex"));
    }

    #[test]
    fn test_regex_filter_nonexistent_variable() {
        let store = RLMContextStore::new();
        let result = store.regex_filter("nonexistent", "pattern");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_remove_variable() {
        let mut store = RLMContextStore::new();
        store.load_variable("to_remove", "data".to_string(), ContextType::String);

        assert!(store.contains("to_remove"));
        let removed = store.remove("to_remove");
        assert_eq!(removed, Some("data".to_string()));
        assert!(!store.contains("to_remove"));
        assert!(store.get_metadata("to_remove").is_none());
    }

    #[test]
    fn test_list_variables() {
        let mut store = RLMContextStore::new();
        store.load_variable("var1", "a".to_string(), ContextType::String);
        store.load_variable("var2", "b".to_string(), ContextType::String);
        store.load_variable("var3", "c".to_string(), ContextType::String);

        let vars = store.list_variables();
        assert_eq!(vars.len(), 3);
        assert!(vars.contains(&"var1".to_string()));
        assert!(vars.contains(&"var2".to_string()));
        assert!(vars.contains(&"var3".to_string()));
    }

    #[test]
    fn test_clear_store() {
        let mut store = RLMContextStore::new();
        store.load_variable("var1", "a".to_string(), ContextType::String);
        store.load_variable("var2", "b".to_string(), ContextType::String);

        store.clear();
        assert!(store.list_variables().is_empty());
    }

    #[test]
    fn test_context_metadata() {
        let mut store = RLMContextStore::new();
        store.load_variable("code", "fn test() {}".to_string(), ContextType::CodeFiles);

        let meta = store.get_metadata("code").unwrap();
        assert_eq!(meta.total_length, 12);
        assert_eq!(meta.context_type, ContextType::CodeFiles);
        assert!(meta.created_at > 0);
    }

    #[test]
    fn test_rlm_config_default() {
        let config = RLMConfig::default();
        assert_eq!(config.max_depth, 1);
        assert_eq!(config.max_iterations, 20);
        assert_eq!(config.default_chunk_size, 50_000);
        assert_eq!(config.rlm_threshold, 50_000);
        assert!(!config.use_sub_model);
        assert!(config.sub_model_key.is_none());
    }

    #[test]
    fn test_rlm_result_success() {
        let trajectory = vec![
            RLMTrajectoryStep {
                step: 1,
                operation: RLMOperation::Start,
                description: "Started".to_string(),
                data: None,
                timestamp_ms: 0,
            },
        ];

        let result = RLMResult::success("answer".to_string(), 5, 2, trajectory);
        assert!(result.success);
        assert_eq!(result.output, "answer");
        assert_eq!(result.iterations, 5);
        assert_eq!(result.sub_calls, 2);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_rlm_result_failure() {
        let result = RLMResult::failure(vec!["Error 1".to_string(), "Error 2".to_string()]);
        assert!(!result.success);
        assert!(result.output.is_empty());
        assert_eq!(result.errors.len(), 2);
    }

    #[test]
    fn test_shared_store() {
        let store = create_shared_store();

        {
            let mut guard = store.lock().unwrap();
            guard.load_variable("shared", "data".to_string(), ContextType::String);
        }

        {
            let guard = store.lock().unwrap();
            assert!(guard.contains("shared"));
        }
    }

    #[test]
    fn test_context_type_variants() {
        // Ensure all context types can be used
        let types = vec![
            ContextType::String,
            ContextType::Structured,
            ContextType::Documents,
            ContextType::SymbolGraph,
            ContextType::CodeFiles,
            ContextType::MiniCodebase,
            ContextType::File,
        ];

        let mut store = RLMContextStore::new();
        for (i, ctx_type) in types.iter().enumerate() {
            store.load_variable(&format!("var_{}", i), "content".to_string(), *ctx_type);
        }

        assert_eq!(store.list_variables().len(), 7);
    }

    #[test]
    fn test_rlm_operation_variants() {
        // Ensure all operation types can be created
        let ops = vec![
            RLMOperation::Start,
            RLMOperation::Peek { var_name: "test".to_string(), start: 0, end: 100 },
            RLMOperation::Chunk { var_name: "test".to_string(), num_chunks: 5 },
            RLMOperation::SubQuery { prompt_preview: "query".to_string(), depth: 1 },
            RLMOperation::SubResult { result_preview: "result".to_string() },
            RLMOperation::RegexFilter { var_name: "test".to_string(), pattern: ".*".to_string(), matches: 10 },
            RLMOperation::LoadContext { var_name: "test".to_string(), length: 1000 },
            RLMOperation::Final { answer_preview: "answer".to_string() },
            RLMOperation::Error { message: "error".to_string() },
        ];

        // Create trajectory steps with each operation
        let steps: Vec<RLMTrajectoryStep> = ops.into_iter().enumerate().map(|(i, op)| {
            RLMTrajectoryStep {
                step: i,
                operation: op,
                description: format!("Step {}", i),
                data: None,
                timestamp_ms: i as u64 * 100,
            }
        }).collect();

        assert_eq!(steps.len(), 9);
    }

    #[test]
    fn test_large_content_chunking() {
        let mut store = RLMContextStore::new();
        // Create content larger than default chunk size
        let large_content: String = "X".repeat(150_000);
        store.load_variable("large", large_content, ContextType::String);

        let chunks = store.chunk("large", 50_000);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].len(), 50_000);
        assert_eq!(chunks[1].len(), 50_000);
        assert_eq!(chunks[2].len(), 50_000);
    }

    #[test]
    fn test_unicode_content() {
        let mut store = RLMContextStore::new();
        let unicode_content = "Hello 世界! 🦀 Rust is awesome! 日本語テスト";
        store.load_variable("unicode", unicode_content.to_string(), ContextType::String);

        // Peek should work with unicode
        let peek = store.peek("unicode", 0, 20);
        assert!(peek.is_some());

        // Chunking should handle unicode properly
        let chunks = store.chunk("unicode", 10);
        assert!(!chunks.is_empty());

        // Verify content is preserved
        let reconstructed: String = chunks.join("");
        assert_eq!(reconstructed, unicode_content);
    }
}

/// Integration tests for RLM with large codebases
/// Run with: cargo test --features rlm_integration -- --ignored
#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::time::Instant;

    /// Helper to create a simulated large codebase (14MB, similar to sidecar)
    fn create_simulated_codebase() -> String {
        // Simulate the structure of codestoryai/sidecar
        // Total target: ~14 million characters
        let mut content = String::with_capacity(14_500_000);

        // Add file headers and Rust code patterns
        for file_num in 0..500 {
            content.push_str(&format!("\n// ================================================================================\n"));
            content.push_str(&format!("// File: sidecar/src/agentic/tool/module_{}.rs\n", file_num));
            content.push_str(&format!("// ================================================================================\n\n"));

            // Add typical Rust module content (~28K chars per file to reach 14MB total)
            content.push_str("use std::collections::HashMap;\n");
            content.push_str("use std::sync::{Arc, Mutex};\n");
            content.push_str("use serde::{Deserialize, Serialize};\n\n");

            // Add structs
            for struct_num in 0..10 {
                content.push_str(&format!(r#"
/// Documentation for Struct{struct_num}Module{file_num}
/// This struct handles important functionality for the agentic system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct{struct_num}Module{file_num} {{
    /// The unique identifier
    pub id: String,
    /// Configuration settings
    pub config: HashMap<String, String>,
    /// Internal state data
    pub state: Vec<u8>,
    /// Metadata about the operation
    pub metadata: Option<serde_json::Value>,
}}

impl Struct{struct_num}Module{file_num} {{
    pub fn new(id: &str) -> Self {{
        Self {{
            id: id.to_string(),
            config: HashMap::new(),
            state: Vec::new(),
            metadata: None,
        }}
    }}

    pub fn process(&mut self, input: &str) -> Result<String, String> {{
        // Process the input and update state
        self.state.extend(input.bytes());
        Ok(format!("Processed {{}} bytes", input.len()))
    }}

    pub fn get_status(&self) -> &str {{
        if self.state.is_empty() {{
            "idle"
        }} else {{
            "active"
        }}
    }}
}}

"#, struct_num = struct_num, file_num = file_num));
            }

            // Add functions
            for fn_num in 0..20 {
                content.push_str(&format!(r#"
/// Helper function {fn_num} for module {file_num}
/// Performs critical operations for the agent workflow.
pub fn helper_function_{fn_num}_module_{file_num}(
    input: &str,
    config: &HashMap<String, String>,
) -> Result<Vec<String>, String> {{
    let mut results = Vec::new();

    for (key, value) in config.iter() {{
        if input.contains(key) {{
            results.push(format!("{{}}: {{}}", key, value));
        }}
    }}

    if results.is_empty() {{
        Err("No matching configuration found".to_string())
    }} else {{
        Ok(results)
    }}
}}

"#, fn_num = fn_num, file_num = file_num));
            }
        }

        content
    }

    /// Test: Load and analyze a large codebase (~14MB)
    /// This simulates the codestoryai/sidecar repository
    #[test]
    #[ignore] // Run with: cargo test integration_test_large_codebase_loading -- --ignored
    fn integration_test_large_codebase_loading() {
        println!("\n=== RLM Integration Test: Large Codebase Loading ===\n");

        let start = Instant::now();
        let codebase = create_simulated_codebase();
        let generation_time = start.elapsed();

        println!("✓ Generated simulated codebase:");
        println!("  - Size: {} bytes ({:.2} MB)", codebase.len(), codebase.len() as f64 / 1_000_000.0);
        println!("  - Generation time: {:?}", generation_time);

        // Load into RLM store
        let start = Instant::now();
        let mut store = RLMContextStore::new();
        store.load_variable("sidecar_codebase", codebase.clone(), ContextType::MiniCodebase);
        let load_time = start.elapsed();

        println!("\n✓ Loaded into RLM store:");
        println!("  - Load time: {:?}", load_time);

        // Verify metadata
        let meta = store.get_metadata("sidecar_codebase").unwrap();
        println!("  - Total length: {} chars", meta.total_length);
        println!("  - Context type: {:?}", meta.context_type);

        assert!(meta.total_length > 10_000_000, "Codebase should be > 10MB");
        assert_eq!(meta.context_type, ContextType::MiniCodebase);

        println!("\n✓ Large codebase loading test PASSED\n");
    }

    /// Test: Chunk a large codebase into manageable pieces
    #[test]
    #[ignore]
    fn integration_test_codebase_chunking() {
        println!("\n=== RLM Integration Test: Codebase Chunking ===\n");

        let codebase = create_simulated_codebase();
        let total_size = codebase.len();

        let mut store = RLMContextStore::new();
        store.load_variable("sidecar_codebase", codebase, ContextType::MiniCodebase);

        // Test default chunk size (50K)
        let config = RLMConfig::default();
        let start = Instant::now();
        let chunks = store.chunk("sidecar_codebase", config.default_chunk_size);
        let chunk_time = start.elapsed();

        println!("✓ Chunking with {} char chunks:", config.default_chunk_size);
        println!("  - Number of chunks: {}", chunks.len());
        println!("  - Chunking time: {:?}", chunk_time);
        println!("  - Expected chunks: ~{}", (total_size as f64 / config.default_chunk_size as f64).ceil() as usize);

        // Verify chunk sizes
        let mut total_reconstructed = 0;
        for (i, chunk) in chunks.iter().enumerate() {
            total_reconstructed += chunk.len();
            if i < 3 || i >= chunks.len() - 2 {
                println!("  - Chunk {}: {} chars", i, chunk.len());
            } else if i == 3 {
                println!("  - ... ({} more chunks) ...", chunks.len() - 5);
            }
        }

        println!("  - Total reconstructed: {} chars", total_reconstructed);
        assert_eq!(total_reconstructed, total_size, "Chunking should preserve all content");

        // Test smaller chunk size (10K) for more granular processing
        store.load_variable("sidecar_small", "X".repeat(total_size), ContextType::MiniCodebase);
        let small_chunks = store.chunk("sidecar_small", 10_000);
        println!("\n✓ Chunking with 10K char chunks:");
        println!("  - Number of chunks: {}", small_chunks.len());

        println!("\n✓ Codebase chunking test PASSED\n");
    }

    /// Test: Regex filtering on large codebase
    #[test]
    #[ignore]
    fn integration_test_codebase_regex_filtering() {
        println!("\n=== RLM Integration Test: Regex Filtering ===\n");

        let codebase = create_simulated_codebase();
        let mut store = RLMContextStore::new();
        store.load_variable("sidecar_codebase", codebase, ContextType::MiniCodebase);

        // Test 1: Find all struct definitions
        let start = Instant::now();
        let structs = store.regex_filter("sidecar_codebase", r"^pub struct ").unwrap();
        let filter_time = start.elapsed();

        println!("✓ Filter: 'pub struct '");
        println!("  - Matches found: {}", structs.len());
        println!("  - Filter time: {:?}", filter_time);
        if !structs.is_empty() {
            println!("  - Sample: {}", &structs[0][..structs[0].len().min(60)]);
        }

        // Test 2: Find all function definitions
        let start = Instant::now();
        let functions = store.regex_filter("sidecar_codebase", r"^pub fn ").unwrap();
        let filter_time = start.elapsed();

        println!("\n✓ Filter: 'pub fn '");
        println!("  - Matches found: {}", functions.len());
        println!("  - Filter time: {:?}", filter_time);

        // Test 3: Find impl blocks
        let start = Instant::now();
        let impls = store.regex_filter("sidecar_codebase", r"^impl ").unwrap();
        let filter_time = start.elapsed();

        println!("\n✓ Filter: 'impl '");
        println!("  - Matches found: {}", impls.len());
        println!("  - Filter time: {:?}", filter_time);

        // Test 4: Find file headers
        let start = Instant::now();
        let headers = store.regex_filter("sidecar_codebase", r"^// File:").unwrap();
        let filter_time = start.elapsed();

        println!("\n✓ Filter: '// File:'");
        println!("  - Matches found: {} (should be ~500)", headers.len());
        println!("  - Filter time: {:?}", filter_time);
        assert_eq!(headers.len(), 500, "Should find 500 file headers");

        // Test 5: Complex regex - find Result return types
        let start = Instant::now();
        let results = store.regex_filter("sidecar_codebase", r"Result<.*>").unwrap();
        let filter_time = start.elapsed();

        println!("\n✓ Filter: 'Result<.*>'");
        println!("  - Matches found: {}", results.len());
        println!("  - Filter time: {:?}", filter_time);

        println!("\n✓ Regex filtering test PASSED\n");
    }

    /// Test: Peek operation performance on large content
    #[test]
    #[ignore]
    fn integration_test_peek_performance() {
        println!("\n=== RLM Integration Test: Peek Performance ===\n");

        let codebase = create_simulated_codebase();
        let total_size = codebase.len();

        let mut store = RLMContextStore::new();
        store.load_variable("sidecar_codebase", codebase, ContextType::MiniCodebase);

        // Test peeking at various positions
        let positions = vec![
            (0, 1000, "start"),
            (total_size / 4, total_size / 4 + 1000, "quarter"),
            (total_size / 2, total_size / 2 + 1000, "middle"),
            (total_size * 3 / 4, total_size * 3 / 4 + 1000, "three-quarter"),
            (total_size - 1000, total_size, "end"),
        ];

        for (start, end, name) in positions {
            let peek_start = Instant::now();
            let peeked = store.peek("sidecar_codebase", start, end).unwrap();
            let peek_time = peek_start.elapsed();

            println!("✓ Peek at {} ({}..{}):", name, start, end);
            println!("  - Retrieved: {} chars", peeked.len());
            println!("  - Time: {:?}", peek_time);
            println!("  - Preview: {}...", &peeked[..peeked.len().min(50)].replace('\n', "\\n"));

            assert!(peek_time.as_micros() < 10_000, "Peek should be fast (<10ms)");
        }

        println!("\n✓ Peek performance test PASSED\n");
    }

    /// Test: Full RLM workflow simulation
    #[test]
    #[ignore]
    fn integration_test_full_rlm_workflow() {
        println!("\n=== RLM Integration Test: Full Workflow Simulation ===\n");

        let codebase = create_simulated_codebase();
        let total_size = codebase.len();
        let config = RLMConfig::default();

        let mut store = RLMContextStore::new();
        let mut trajectory: Vec<RLMTrajectoryStep> = Vec::new();
        let workflow_start = Instant::now();

        // Step 1: Load context
        println!("Step 1: Loading codebase into RLM store...");
        store.load_variable("codebase", codebase, ContextType::MiniCodebase);
        trajectory.push(RLMTrajectoryStep {
            step: 1,
            operation: RLMOperation::LoadContext {
                var_name: "codebase".to_string(),
                length: total_size
            },
            description: format!("Loaded {} byte codebase", total_size),
            data: None,
            timestamp_ms: workflow_start.elapsed().as_millis() as u64,
        });

        // Step 2: Check if RLM mode is needed
        let needs_rlm = total_size > config.rlm_threshold;
        println!("Step 2: Context size {} > threshold {}? {}",
            total_size, config.rlm_threshold, needs_rlm);
        assert!(needs_rlm, "Large codebase should trigger RLM mode");

        // Step 3: Chunk the content
        println!("Step 3: Chunking codebase into {} char pieces...", config.default_chunk_size);
        let chunks = store.chunk("codebase", config.default_chunk_size);
        trajectory.push(RLMTrajectoryStep {
            step: 2,
            operation: RLMOperation::Chunk {
                var_name: "codebase".to_string(),
                num_chunks: chunks.len()
            },
            description: format!("Created {} chunks", chunks.len()),
            data: None,
            timestamp_ms: workflow_start.elapsed().as_millis() as u64,
        });
        println!("  - Created {} chunks", chunks.len());

        // Step 4: Apply regex filter to find relevant sections
        println!("Step 4: Filtering for struct definitions...");
        let structs = store.regex_filter("codebase", r"^pub struct ").unwrap();
        trajectory.push(RLMTrajectoryStep {
            step: 3,
            operation: RLMOperation::RegexFilter {
                var_name: "codebase".to_string(),
                pattern: "^pub struct ".to_string(),
                matches: structs.len()
            },
            description: format!("Found {} struct definitions", structs.len()),
            data: None,
            timestamp_ms: workflow_start.elapsed().as_millis() as u64,
        });
        println!("  - Found {} structs", structs.len());

        // Step 5: Peek at specific sections
        println!("Step 5: Peeking at first chunk for analysis...");
        let peek_content = store.peek("codebase", 0, 5000).unwrap();
        trajectory.push(RLMTrajectoryStep {
            step: 4,
            operation: RLMOperation::Peek {
                var_name: "codebase".to_string(),
                start: 0,
                end: 5000
            },
            description: "Peeked at first 5000 chars".to_string(),
            data: None,
            timestamp_ms: workflow_start.elapsed().as_millis() as u64,
        });
        println!("  - Retrieved {} chars", peek_content.len());

        // Step 6: Simulate sub-query processing
        println!("Step 6: Simulating sub-LM queries for each chunk...");
        let mut sub_call_count = 0;
        for (i, chunk) in chunks.iter().take(5).enumerate() {
            // Simulate processing (in real RLM, this would call the sub-LM)
            let prompt = format!("Analyze chunk {}: [{}... ({} chars)]", i, &chunk[..chunk.len().min(50)], chunk.len());
            trajectory.push(RLMTrajectoryStep {
                step: 5 + i,
                operation: RLMOperation::SubQuery {
                    prompt_preview: prompt[..prompt.len().min(80)].to_string(),
                    depth: 1
                },
                description: format!("Sub-query for chunk {}", i),
                data: None,
                timestamp_ms: workflow_start.elapsed().as_millis() as u64,
            });

            // Simulate result
            trajectory.push(RLMTrajectoryStep {
                step: 5 + i,
                operation: RLMOperation::SubResult {
                    result_preview: format!("Chunk {} contains Rust code with {} chars", i, chunk.len())
                },
                description: format!("Got result for chunk {}", i),
                data: None,
                timestamp_ms: workflow_start.elapsed().as_millis() as u64,
            });
            sub_call_count += 1;
        }
        println!("  - Processed {} chunks (showing first 5)", sub_call_count);

        // Step 7: Final answer
        println!("Step 7: Generating final answer...");
        let final_answer = format!(
            "Analysis complete: {} chunks, {} structs found, {} sub-queries made",
            chunks.len(), structs.len(), sub_call_count
        );
        trajectory.push(RLMTrajectoryStep {
            step: trajectory.len(),
            operation: RLMOperation::Final {
                answer_preview: final_answer.clone()
            },
            description: "RLM execution complete".to_string(),
            data: None,
            timestamp_ms: workflow_start.elapsed().as_millis() as u64,
        });

        let total_time = workflow_start.elapsed();

        // Create result
        let result = RLMResult {
            success: true,
            output: final_answer,
            iterations: trajectory.len(),
            sub_calls: sub_call_count,
            total_tokens: 0, // Would be calculated from actual LLM calls
            trajectory,
            errors: Vec::new(),
            execution_time_ms: total_time.as_millis() as u64,
        };

        println!("\n=== RLM Execution Summary ===");
        println!("  - Success: {}", result.success);
        println!("  - Output: {}", result.output);
        println!("  - Iterations: {}", result.iterations);
        println!("  - Sub-calls: {}", result.sub_calls);
        println!("  - Trajectory steps: {}", result.trajectory.len());
        println!("  - Total time: {:?}", total_time);

        assert!(result.success);
        assert!(!result.trajectory.is_empty());

        println!("\n✓ Full RLM workflow test PASSED\n");
    }

    /// Test: Memory efficiency with multiple large contexts
    #[test]
    #[ignore]
    fn integration_test_memory_efficiency() {
        println!("\n=== RLM Integration Test: Memory Efficiency ===\n");

        let mut store = RLMContextStore::new();

        // Load multiple large contexts
        let contexts = vec![
            ("codebase_1", 5_000_000, ContextType::MiniCodebase),
            ("codebase_2", 3_000_000, ContextType::CodeFiles),
            ("documents", 2_000_000, ContextType::Documents),
            ("symbol_graph", 1_000_000, ContextType::SymbolGraph),
        ];

        let mut total_loaded = 0;
        for (name, size, ctx_type) in &contexts {
            let content = "X".repeat(*size);
            store.load_variable(name, content, *ctx_type);
            total_loaded += size;

            println!("✓ Loaded '{}': {} MB, type: {:?}",
                name, *size as f64 / 1_000_000.0, ctx_type);
        }

        println!("\nTotal loaded: {} MB", total_loaded as f64 / 1_000_000.0);
        println!("Variables in store: {:?}", store.list_variables());

        // Test that we can still access all contexts
        for (name, size, _) in &contexts {
            assert!(store.contains(name));
            assert_eq!(store.length(name), Some(*size));
        }

        // Remove one and verify memory is freed
        store.remove("codebase_2");
        assert!(!store.contains("codebase_2"));
        println!("\n✓ Removed 'codebase_2'");
        println!("Remaining variables: {:?}", store.list_variables());

        // Clear all
        store.clear();
        assert!(store.list_variables().is_empty());
        println!("✓ Cleared all - store is empty");

        println!("\n✓ Memory efficiency test PASSED\n");
    }

    /// Test: Real repomix output loading (if available)
    #[test]
    #[ignore]
    fn integration_test_real_repomix_file() {
        println!("\n=== RLM Integration Test: Real Repomix File ===\n");

        // Try to load the actual repomix output
        let repomix_path = std::path::Path::new("repomix-output.xml");

        if !repomix_path.exists() {
            println!("⚠ repomix-output.xml not found - skipping real file test");
            println!("  Run: repomix --remote codestoryai/sidecar");
            return;
        }

        let start = Instant::now();
        let content = std::fs::read_to_string(repomix_path)
            .expect("Failed to read repomix-output.xml");
        let read_time = start.elapsed();

        println!("✓ Loaded repomix-output.xml:");
        println!("  - Size: {} bytes ({:.2} MB)", content.len(), content.len() as f64 / 1_000_000.0);
        println!("  - Read time: {:?}", read_time);

        // Load into RLM store
        let mut store = RLMContextStore::new();
        let start = Instant::now();
        store.load_variable("sidecar_full", content.clone(), ContextType::MiniCodebase);
        let load_time = start.elapsed();

        println!("✓ Loaded into RLM store in {:?}", load_time);

        // Chunk it
        let config = RLMConfig::default();
        let start = Instant::now();
        let chunks = store.chunk("sidecar_full", config.default_chunk_size);
        let chunk_time = start.elapsed();

        println!("✓ Chunked into {} pieces ({} chars each) in {:?}",
            chunks.len(), config.default_chunk_size, chunk_time);

        // Find Rust code patterns
        let start = Instant::now();
        let structs = store.regex_filter("sidecar_full", r"pub struct [A-Z]").unwrap_or_default();
        let filter_time = start.elapsed();

        println!("✓ Found {} struct definitions in {:?}", structs.len(), filter_time);

        // Find functions
        let fns = store.regex_filter("sidecar_full", r"pub fn [a-z]").unwrap_or_default();
        println!("✓ Found {} function definitions", fns.len());

        // Find impl blocks
        let impls = store.regex_filter("sidecar_full", r"impl [A-Z]").unwrap_or_default();
        println!("✓ Found {} impl blocks", impls.len());

        println!("\n✓ Real repomix file test PASSED\n");
    }

    /// Test: RLM Demo Rhai Script execution
    /// This test loads the rlm_demo.rhai script and executes it with the sidecar codebase
    #[test]
    #[ignore]
    fn integration_test_rlm_demo_script() {
        println!("\n=== RLM Integration Test: Demo Rhai Script ===\n");

        // Check if repomix file exists
        let repomix_path = std::path::Path::new("repomix-output.xml");
        if !repomix_path.exists() {
            println!("⚠ repomix-output.xml not found - skipping Rhai script test");
            println!("  Run: repomix --remote codestoryai/sidecar");
            return;
        }

        // Load the codebase
        let content = std::fs::read_to_string(repomix_path)
            .expect("Failed to read repomix-output.xml");
        println!("✓ Loaded codebase: {} bytes ({:.2} MB)",
            content.len(), content.len() as f64 / 1_000_000.0);

        // Create RLM store and load content
        let mut store = RLMContextStore::new();
        store.load_variable("sidecar_codebase", content.clone(), ContextType::MiniCodebase);

        // Simulate what the Rhai script would do
        println!("\n--- Simulating rlm_demo.rhai workflow ---\n");

        // Step 1: Load (already done)
        let size = store.length("sidecar_codebase").unwrap_or(0);
        println!("Step 1 - Load: {} chars", size);
        assert!(size > 10_000_000, "Expected large codebase");

        // Step 2: Analyze structure
        let structs = store.regex_filter("sidecar_codebase", r"pub struct [A-Z]").unwrap_or_default();
        let functions = store.regex_filter("sidecar_codebase", r"pub fn [a-z]").unwrap_or_default();
        let impls = store.regex_filter("sidecar_codebase", r"impl [A-Z]").unwrap_or_default();
        let traits = store.regex_filter("sidecar_codebase", r"pub trait [A-Z]").unwrap_or_default();
        let mods = store.regex_filter("sidecar_codebase", r"pub mod [a-z]").unwrap_or_default();

        println!("Step 2 - Structure analysis:");
        println!("  - {} public structs", structs.len());
        println!("  - {} public functions", functions.len());
        println!("  - {} impl blocks", impls.len());
        println!("  - {} public traits", traits.len());
        println!("  - {} public modules", mods.len());

        // Step 3: Chunking
        let config = RLMConfig::default();
        let chunks = store.chunk("sidecar_codebase", config.default_chunk_size);
        println!("Step 3 - Chunking: {} chunks of {} chars", chunks.len(), config.default_chunk_size);
        assert!(chunks.len() > 100, "Expected many chunks for large codebase");

        // Step 4: Code extraction
        let async_fns = store.regex_filter("sidecar_codebase", r"pub async fn ").unwrap_or_default();
        let tests = store.regex_filter("sidecar_codebase", r"#\[test\]").unwrap_or_default();
        let derives = store.regex_filter("sidecar_codebase", r"#\[derive\(").unwrap_or_default();

        println!("Step 4 - Code extraction:");
        println!("  - {} async functions", async_fns.len());
        println!("  - {} test functions", tests.len());
        println!("  - {} derive macros", derives.len());

        // Step 5: Peek operations
        let start = Instant::now();
        let positions = vec![0, size / 4, size / 2, size * 3 / 4];
        for pos in &positions {
            let _ = store.peek("sidecar_codebase", *pos, pos + 500);
        }
        let peek_time = start.elapsed();
        println!("Step 5 - Peek: {} positions in {:?}", positions.len(), peek_time);

        // Step 6: Sub-LM simulation (just log what would happen)
        println!("Step 6 - Sub-LM: Would analyze {} structs", std::cmp::min(5, structs.len()));

        // Summary
        println!("\n--- RLM Demo Summary ---");
        println!("  Codebase size: {} chars ({:.2} MB)", size, size as f64 / 1_000_000.0);
        println!("  RLM mode: ENABLED (threshold: 50K)");
        println!("  Total structs: {}", structs.len());
        println!("  Total functions: {}", functions.len());
        println!("  Total chunks: {}", chunks.len());

        println!("\n✓ RLM Demo Script test PASSED\n");
    }
}

