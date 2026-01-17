// Cerebras-MAKER: The Interrogator Agent
// PRD Section 2 (Phase A): Scans user requests for "Known Unknowns"

use super::{Agent, AgentContext, QuestionOutput};
use crate::llm::{LlmConfig, LlmProvider, Message};
use serde::{Deserialize, Serialize};

/// The Interrogator Agent
/// Scans user requests for ambiguity and halts execution to ask clarifying questions
pub struct Interrogator {
    /// Ambiguity threshold (0.0 - 1.0)
    /// If ambiguity score exceeds this, halt and ask user
    pub ambiguity_threshold: f32,
    /// LLM configuration for this agent
    llm_config: LlmConfig,
}

impl Default for Interrogator {
    fn default() -> Self {
        Self {
            ambiguity_threshold: 0.7,
            llm_config: LlmConfig::cerebras(), // Default to Cerebras for speed
        }
    }
}

impl Agent for Interrogator {
    fn name(&self) -> &str {
        "Interrogator"
    }

    fn system_prompt(&self) -> &str {
        r#"You are the Interrogator Agent in the Cerebras-MAKER system.

Your role is to analyze user requests and identify "Known Unknowns" - ambiguities, missing context, 
or unclear requirements that could lead to implementation errors.

For each request, you must:
1. Identify ambiguous terms or requirements
2. List missing context (files, APIs, dependencies)
3. Flag potential conflicts with existing code
4. Calculate an ambiguity score (0.0 = perfectly clear, 1.0 = completely ambiguous)

Output format (JSON):
{
    "ambiguity_score": 0.5,
    "unknowns": [
        {
            "type": "ambiguous_term",
            "description": "The term 'user data' is not defined",
            "question": "What specific user data fields should be included?"
        }
    ],
    "context_needed": ["user_model.rs", "database schema"],
    "proceed": true | false
}

If ambiguity_score > threshold, set proceed=false and we will ask the user for clarification."#
    }
}

impl Interrogator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(mut self, config: LlmConfig) -> Self {
        self.llm_config = config;
        self
    }

    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.ambiguity_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Analyze a user request for ambiguity
    /// Calls the LLM to identify "Known Unknowns" and returns structured result
    pub async fn analyze(&self, request: &str, context: &AgentContext) -> Result<InterrogatorResult, String> {
        // HIGH-2: Validate input before processing
        if request.trim().is_empty() {
            return Err("Request cannot be empty".to_string());
        }
        if context.workspace_path.is_empty() {
            return Err("Workspace path must be provided in context".to_string());
        }

        // Build the user prompt with context
        let user_prompt = self.build_user_prompt(request, context);

        // Create LLM provider
        let provider = LlmProvider::new(self.llm_config.clone())
            .map_err(|e| format!("Failed to create LLM provider: {}", e))?;

        // Build messages
        let messages = vec![
            Message::system(self.system_prompt()),
            Message::user(&user_prompt),
        ];

        // Execute the LLM call
        let response = provider
            .complete(messages)
            .await
            .map_err(|e| format!("LLM call failed: {}", e))?;

        // Parse the JSON response
        self.parse_response(&response.content)
    }

    /// Build the user prompt with context information
    fn build_user_prompt(&self, request: &str, context: &AgentContext) -> String {
        let mut prompt = format!("## User Request\n\n{}\n\n", request);

        prompt.push_str(&format!("## Workspace\n\n{}\n\n", context.workspace_path));

        if let Some(issue_id) = &context.issue_id {
            prompt.push_str(&format!("## Issue ID\n\n{}\n\n", issue_id));
        }

        if !context.previous_outputs.is_empty() {
            prompt.push_str("## Previous Context\n\n");
            for output in &context.previous_outputs {
                prompt.push_str(&format!("- {:?}\n", output));
            }
        }

        prompt.push_str("\nAnalyze this request and output your analysis as JSON.");
        prompt
    }

    /// Parse the LLM response into InterrogatorResult
    fn parse_response(&self, response: &str) -> Result<InterrogatorResult, String> {
        // Try to extract JSON from the response (it might be wrapped in markdown code blocks)
        let json_str = self.extract_json(response);

        // Parse the JSON
        let parsed: serde_json::Value = serde_json::from_str(&json_str)
            .map_err(|e| format!("Failed to parse LLM response as JSON: {}. Response: {}", e, response))?;

        // Extract fields with defaults
        let ambiguity_score = parsed.get("ambiguity_score")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(0.5);

        let proceed = parsed.get("proceed")
            .and_then(|v| v.as_bool())
            .unwrap_or(ambiguity_score <= self.ambiguity_threshold);

        let unknowns = parsed.get("unknowns")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        Some(Unknown {
                            unknown_type: item.get("type")
                                .or_else(|| item.get("unknown_type"))
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            description: item.get("description")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            question: item.get("question")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let context_needed = parsed.get("context_needed")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let questions = parsed.get("questions")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            });

        Ok(InterrogatorResult {
            ambiguity_score,
            unknowns,
            context_needed,
            proceed,
            questions,
        })
    }

    /// Extract JSON from a response that might be wrapped in markdown code blocks
    fn extract_json(&self, response: &str) -> String {
        // Try to find JSON in code blocks first
        if let Some(start) = response.find("```json") {
            if let Some(end) = response[start..].find("```\n").or_else(|| response[start..].rfind("```")) {
                let json_start = start + 7; // Skip "```json"
                let json_end = start + end;
                if json_start < json_end {
                    return response[json_start..json_end].trim().to_string();
                }
            }
        }

        // Try to find JSON in generic code blocks
        if let Some(start) = response.find("```") {
            let after_start = start + 3;
            // Skip the language identifier if present
            let content_start = response[after_start..]
                .find('\n')
                .map(|i| after_start + i + 1)
                .unwrap_or(after_start);
            if let Some(end) = response[content_start..].find("```") {
                return response[content_start..content_start + end].trim().to_string();
            }
        }

        // Try to find raw JSON (starts with { and ends with })
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                if start < end {
                    return response[start..=end].to_string();
                }
            }
        }

        // Return as-is if no JSON found
        response.to_string()
    }

    /// Check if the result requires user intervention
    pub fn needs_clarification(&self, result: &InterrogatorResult) -> bool {
        result.ambiguity_score > self.ambiguity_threshold || !result.proceed
    }

    /// Generate a question for the user
    pub fn generate_question(&self, result: &InterrogatorResult) -> Option<QuestionOutput> {
        if !self.needs_clarification(result) {
            return None;
        }

        let questions: Vec<String> = result.unknowns
            .iter()
            .map(|u| u.question.clone())
            .collect();

        Some(QuestionOutput {
            question_id: format!("q_{}", uuid_simple()),
            question: questions.join("\n"),
            context: result.context_needed.join(", "),
            options: None,
            ambiguity_score: result.ambiguity_score,
        })
    }
}

/// Result from interrogation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterrogatorResult {
    pub ambiguity_score: f32,
    pub unknowns: Vec<Unknown>,
    pub context_needed: Vec<String>,
    pub proceed: bool,
    pub questions: Option<Vec<String>>,
}

/// An identified unknown/ambiguity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Unknown {
    pub unknown_type: String,
    pub description: String,
    pub question: String,
}

/// Simple UUID generator
fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:x}", timestamp)
}

