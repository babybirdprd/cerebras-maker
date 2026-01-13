// Cerebras-MAKER: L4 Atom Executor
// PRD Section 4.2: "The Atom" - Ephemeral, stateless agents with exactly one tool
// Takes a ContextPackage from L3 and executes a focused LLM call

use super::context_engineer::ContextPackage;
use crate::llm::{LlmConfig, LlmProvider, Message, SystemPrompts};
use crate::maker_core::{AtomResult, AtomType, SpawnFlags};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

/// Input to an Atom execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomInput {
    /// The type of atom to execute
    pub atom_type: AtomType,
    /// The task/prompt for the atom
    pub task: String,
    /// Context from L3 Context Engineer (optional - some atoms don't need it)
    pub context: Option<ContextPackage>,
    /// Spawn flags for execution control
    pub flags: SpawnFlags,
    /// Additional variables for prompt templating
    pub variables: HashMap<String, String>,
}

impl AtomInput {
    pub fn new(atom_type: AtomType, task: &str) -> Self {
        Self {
            atom_type,
            task: task.to_string(),
            context: None,
            flags: SpawnFlags::default(),
            variables: HashMap::new(),
        }
    }

    pub fn with_context(mut self, context: ContextPackage) -> Self {
        self.context = Some(context);
        self
    }

    pub fn with_flags(mut self, flags: SpawnFlags) -> Self {
        self.flags = flags;
        self
    }

    pub fn with_var(mut self, key: &str, value: &str) -> Self {
        self.variables.insert(key.to_string(), value.to_string());
        self
    }
}

/// Parsed output from an atom execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AtomOutput {
    /// Raw text output
    Text(String),
    /// JSON structured output
    Json(serde_json::Value),
    /// Code changes (file_path -> code)
    Code(Vec<CodeChange>),
    /// Review result
    Review(ReviewResult),
    /// Validation result
    Validation(ValidationResult),
    /// Search results
    Search(SearchResult),
    /// Plan/task decomposition
    Plan(Vec<PlanStep>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeChange {
    pub file_path: String,
    pub content: String,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    pub approved: bool,
    pub issues: Vec<String>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub violations: Vec<String>,
    pub score: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub files: Vec<String>,
    pub snippets: Vec<SearchSnippet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSnippet {
    pub file: String,
    pub line_start: usize,
    pub line_end: usize,
    pub content: String,
    pub relevance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub id: String,
    pub description: String,
    pub dependencies: Vec<String>,
    pub atom_type: Option<String>,
}

/// The L4 Atom Executor
/// Responsible for executing a single, focused LLM call with one tool/output type
pub struct AtomExecutor {
    llm_config: LlmConfig,
}

impl AtomExecutor {
    pub fn new(llm_config: LlmConfig) -> Self {
        Self { llm_config }
    }

    /// Execute an atom with the given input
    pub async fn execute(&self, input: AtomInput) -> Result<AtomResult, String> {
        let start = Instant::now();

        // Build the system prompt for this atom type
        let system_prompt = self.build_system_prompt(&input);

        // Build the user prompt with context and task
        let user_prompt = self.build_user_prompt(&input);

        // Create LLM provider
        let provider = LlmProvider::new(self.llm_config.clone())
            .map_err(|e| format!("Failed to create LLM provider: {}", e))?;

        // Build messages
        let messages = vec![
            Message::system(&system_prompt),
            Message::user(&user_prompt),
        ];

        // Execute the LLM call
        let response = provider
            .complete(messages)
            .await
            .map_err(|e| format!("LLM call failed: {}", e))?;

        let execution_time_ms = start.elapsed().as_millis() as u64;
        let tokens_used = response.tokens_used.unwrap_or(0) as usize;

        // Parse and validate the output
        let (output, valid, errors) = self.parse_output(&input.atom_type, &response.content, &input.flags);

        let mut result = if valid {
            AtomResult::success(input.atom_type, output, execution_time_ms, tokens_used)
        } else {
            AtomResult::failure(input.atom_type, output, errors)
        };

        // Check for red flags if enabled
        if input.flags.red_flag_check {
            if let Some(reason) = self.check_red_flags(&result, &input) {
                result.set_red_flagged(&reason);
            }
        }

        Ok(result)
    }

    /// Build the system prompt for the atom type
    fn build_system_prompt(&self, input: &AtomInput) -> String {
        let base_prompt = match input.atom_type {
            AtomType::Coder => SystemPrompts::atom_coder(),
            AtomType::Reviewer => SystemPrompts::atom_reviewer(),
            AtomType::Search => SystemPrompts::atom_search(),
            AtomType::Tester => SystemPrompts::atom_tester(),
            AtomType::GritsAnalyzer => SystemPrompts::atom_grits(),
            AtomType::Architect => SystemPrompts::architect(),
            AtomType::Planner => input.atom_type.system_prompt(),
            AtomType::Validator => input.atom_type.system_prompt(),
        };

        // Add JSON output instruction if required
        if input.flags.require_json {
            format!(
                "{}\n\n## Output Format\nYou MUST return valid JSON only. No markdown, no explanations.",
                base_prompt
            )
        } else {
            base_prompt.to_string()
        }
    }

    /// Build the user prompt with context and task
    fn build_user_prompt(&self, input: &AtomInput) -> String {
        let mut prompt = String::new();

        // Add context from L3 if available
        if let Some(ref ctx) = input.context {
            prompt.push_str(&ctx.markdown);
            prompt.push_str("\n---\n\n");

            // Add constraints
            if !ctx.constraints.is_empty() {
                prompt.push_str("## Constraints\n");
                for constraint in &ctx.constraints {
                    prompt.push_str(&format!("- {}\n", constraint));
                }
                prompt.push_str("\n");
            }
        }

        // Add the task
        prompt.push_str("## Task\n");
        prompt.push_str(&input.task);

        // Add any custom variables
        for (key, value) in &input.variables {
            let placeholder = format!("{{{{{}}}}}", key);
            prompt = prompt.replace(&placeholder, value);
        }

        prompt
    }

    /// Parse the LLM output based on atom type
    fn parse_output(
        &self,
        atom_type: &AtomType,
        raw_output: &str,
        flags: &SpawnFlags,
    ) -> (String, bool, Vec<String>) {
        let mut errors = Vec::new();

        // If JSON is required, validate it
        if flags.require_json {
            match serde_json::from_str::<serde_json::Value>(raw_output) {
                Ok(_) => return (raw_output.to_string(), true, errors),
                Err(e) => {
                    // Try to extract JSON from markdown code blocks
                    if let Some(json_str) = self.extract_json_from_markdown(raw_output) {
                        if serde_json::from_str::<serde_json::Value>(&json_str).is_ok() {
                            return (json_str, true, errors);
                        }
                    }
                    errors.push(format!("Invalid JSON output: {}", e));
                    return (raw_output.to_string(), false, errors);
                }
            }
        }

        // Atom-specific validation
        match atom_type {
            AtomType::Coder => {
                // Coder output should contain code blocks
                if !raw_output.contains("```") && !raw_output.contains("FILE:") {
                    errors.push("Coder output should contain code blocks".to_string());
                }
            }
            AtomType::Reviewer => {
                // Reviewer should return JSON with approved field
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(raw_output) {
                    if json.get("approved").is_none() {
                        errors.push("Reviewer output missing 'approved' field".to_string());
                    }
                } else if let Some(json_str) = self.extract_json_from_markdown(raw_output) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_str) {
                        if json.get("approved").is_some() {
                            return (json_str, true, errors);
                        }
                    }
                }
            }
            AtomType::Validator => {
                // Validator should return JSON with valid field
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(raw_output) {
                    if json.get("valid").is_none() {
                        errors.push("Validator output missing 'valid' field".to_string());
                    }
                }
            }
            _ => {}
        }

        (raw_output.to_string(), errors.is_empty(), errors)
    }

    /// Extract JSON from markdown code blocks
    fn extract_json_from_markdown(&self, text: &str) -> Option<String> {
        // Look for ```json ... ``` blocks
        let json_pattern = regex::Regex::new(r"```(?:json)?\s*\n([\s\S]*?)\n```").ok()?;
        if let Some(caps) = json_pattern.captures(text) {
            return Some(caps[1].trim().to_string());
        }
        None
    }

    /// Check for red flags in the output
    fn check_red_flags(&self, result: &AtomResult, input: &AtomInput) -> Option<String> {
        // Check for dangerous patterns in code output
        if matches!(input.atom_type, AtomType::Coder) {
            let dangerous_patterns = [
                ("rm -rf", "Potentially destructive file deletion"),
                ("DROP TABLE", "Database deletion command"),
                ("eval(", "Unsafe eval usage"),
                ("exec(", "Unsafe exec usage"),
                ("__import__", "Dynamic import (security risk)"),
                ("subprocess.call", "Shell command execution"),
                ("os.system", "System command execution"),
            ];

            for (pattern, reason) in dangerous_patterns {
                if result.output.contains(pattern) {
                    return Some(format!("Red flag: {} - found '{}'", reason, pattern));
                }
            }
        }

        // Check if constraints were violated
        if let Some(ref ctx) = input.context {
            for constraint in &ctx.constraints {
                if constraint.contains("Do not depend on:") {
                    // Extract the forbidden dependency
                    if let Some(dep) = constraint.strip_prefix("Do not depend on: ") {
                        if result.output.contains(dep) {
                            return Some(format!(
                                "Constraint violation: Code depends on forbidden module '{}'",
                                dep
                            ));
                        }
                    }
                }
            }
        }

        None
    }

    /// Parse code output into structured CodeChange objects
    pub fn parse_code_output(&self, raw_output: &str) -> Vec<CodeChange> {
        let mut changes = Vec::new();
        let mut current_file: Option<String> = None;
        let mut current_content = String::new();
        let mut current_lang: Option<String> = None;
        let mut in_code_block = false;

        for line in raw_output.lines() {
            if line.starts_with("FILE:") {
                // Save previous file if any
                if let Some(ref file) = current_file {
                    if !current_content.is_empty() {
                        changes.push(CodeChange {
                            file_path: file.clone(),
                            content: current_content.trim().to_string(),
                            language: current_lang.take(),
                        });
                    }
                }
                current_file = Some(line.trim_start_matches("FILE:").trim().to_string());
                current_content.clear();
            } else if line.starts_with("```") {
                if in_code_block {
                    in_code_block = false;
                } else {
                    in_code_block = true;
                    let lang = line.trim_start_matches("```").trim();
                    if !lang.is_empty() {
                        current_lang = Some(lang.to_string());
                    }
                }
            } else if in_code_block {
                current_content.push_str(line);
                current_content.push('\n');
            }
        }

        // Save last file
        if let Some(ref file) = current_file {
            if !current_content.is_empty() {
                changes.push(CodeChange {
                    file_path: file.clone(),
                    content: current_content.trim().to_string(),
                    language: current_lang,
                });
            }
        }

        changes
    }

    /// Parse review output into ReviewResult
    pub fn parse_review_output(&self, raw_output: &str) -> Result<ReviewResult, String> {
        // Try direct JSON parse
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(raw_output) {
            return self.json_to_review(&json);
        }

        // Try extracting from markdown
        if let Some(json_str) = self.extract_json_from_markdown(raw_output) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_str) {
                return self.json_to_review(&json);
            }
        }

        Err("Could not parse review output as JSON".to_string())
    }

    fn json_to_review(&self, json: &serde_json::Value) -> Result<ReviewResult, String> {
        Ok(ReviewResult {
            approved: json
                .get("approved")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            issues: json
                .get("issues")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            suggestions: json
                .get("suggestions")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
        })
    }

    /// Parse validation output into ValidationResult
    pub fn parse_validation_output(&self, raw_output: &str) -> Result<ValidationResult, String> {
        let json: serde_json::Value = serde_json::from_str(raw_output)
            .or_else(|_| {
                self.extract_json_from_markdown(raw_output)
                    .ok_or_else(|| "No JSON found".to_string())
                    .and_then(|s| serde_json::from_str(&s).map_err(|e| e.to_string()))
            })
            .map_err(|e| format!("Could not parse validation output: {}", e))?;

        Ok(ValidationResult {
            valid: json.get("valid").and_then(|v| v.as_bool()).unwrap_or(false),
            violations: json
                .get("violations")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            score: json.get("score").and_then(|v| v.as_f64()).map(|f| f as f32),
        })
    }
}

