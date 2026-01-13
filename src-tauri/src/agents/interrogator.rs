// Cerebras-MAKER: The Interrogator Agent
// PRD Section 2 (Phase A): Scans user requests for "Known Unknowns"

use super::{Agent, AgentContext, QuestionOutput};
use serde::{Deserialize, Serialize};

/// The Interrogator Agent
/// Scans user requests for ambiguity and halts execution to ask clarifying questions
pub struct Interrogator {
    /// Ambiguity threshold (0.0 - 1.0)
    /// If ambiguity score exceeds this, halt and ask user
    pub ambiguity_threshold: f32,
}

impl Default for Interrogator {
    fn default() -> Self {
        Self {
            ambiguity_threshold: 0.7,
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

    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.ambiguity_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Analyze a user request for ambiguity
    /// Returns AgentOutput::Question if clarification needed
    pub fn analyze(&self, request: &str, _context: &AgentContext) -> InterrogatorResult {
        // In production, this calls the LLM via rig-core
        // For now, return a placeholder
        InterrogatorResult {
            ambiguity_score: 0.3,
            unknowns: Vec::new(),
            context_needed: Vec::new(),
            proceed: true,
            questions: None,
        }
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

