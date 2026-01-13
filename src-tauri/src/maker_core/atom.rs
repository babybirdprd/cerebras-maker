// Cerebras-MAKER: Atom Types and Ephemeral Agents
// PRD Section 4.2: AtomType - Strictly typed worker definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Atom types for the MAKER framework
/// Each atom is an ephemeral, stateless agent with exactly one tool
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AtomType {
    /// Search atom - searches the codebase
    Search,
    /// Coder atom - generates code
    Coder,
    /// Reviewer atom - reviews code for issues
    Reviewer,
    /// Planner atom - creates task decompositions
    Planner,
    /// Validator atom - validates code against requirements
    Validator,
}

impl AtomType {
    /// Get the system prompt for this atom type
    pub fn system_prompt(&self) -> &'static str {
        match self {
            AtomType::Search => {
                "You are a Search Atom. Your only job is to find relevant code in the codebase. \
                 Return results as JSON with 'files' and 'snippets' arrays."
            }
            AtomType::Coder => {
                "You are a Coder Atom. Your only job is to write a single piece of code. \
                 Return valid code only, no explanations. Code must be syntactically correct."
            }
            AtomType::Reviewer => {
                "You are a Reviewer Atom. Your only job is to review code for bugs and issues. \
                 Return a JSON object with 'approved' (boolean) and 'issues' (array of strings)."
            }
            AtomType::Planner => {
                "You are a Planner Atom. Your only job is to break down a task into atomic steps. \
                 Return a JSON array of step objects with 'id', 'description', and 'dependencies'."
            }
            AtomType::Validator => {
                "You are a Validator Atom. Your only job is to check if code meets requirements. \
                 Return a JSON object with 'valid' (boolean) and 'violations' (array of strings)."
            }
        }
    }

    /// Get the max output tokens for this atom type
    pub fn max_tokens(&self) -> usize {
        match self {
            AtomType::Search => 500,
            AtomType::Coder => 2000,
            AtomType::Reviewer => 750,
            AtomType::Planner => 1000,
            AtomType::Validator => 500,
        }
    }
}

/// Result from an Atom execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomResult {
    /// The atom type that generated this result
    pub atom_type: AtomType,
    /// The raw output from the LLM
    pub output: String,
    /// Whether the output passed validation
    pub valid: bool,
    /// Validation errors if any
    pub errors: Vec<String>,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Token usage
    pub tokens_used: usize,
    /// Metadata from execution
    pub metadata: HashMap<String, String>,
}

impl AtomResult {
    /// Create a new valid result
    pub fn success(atom_type: AtomType, output: String, execution_time_ms: u64, tokens_used: usize) -> Self {
        Self {
            atom_type,
            output,
            valid: true,
            errors: Vec::new(),
            execution_time_ms,
            tokens_used,
            metadata: HashMap::new(),
        }
    }

    /// Create a new invalid result
    pub fn failure(atom_type: AtomType, output: String, errors: Vec<String>) -> Self {
        Self {
            atom_type,
            output,
            valid: false,
            errors,
            execution_time_ms: 0,
            tokens_used: 0,
            metadata: HashMap::new(),
        }
    }

    /// Check if this result is a red flag (contains architectural violations)
    pub fn is_red_flagged(&self) -> bool {
        self.metadata.get("red_flagged").map(|v| v == "true").unwrap_or(false)
    }

    /// Mark this result as red flagged
    pub fn set_red_flagged(&mut self, reason: &str) {
        self.metadata.insert("red_flagged".to_string(), "true".to_string());
        self.metadata.insert("red_flag_reason".to_string(), reason.to_string());
    }
}

/// Spawn flags for atom execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnFlags {
    /// Require JSON output
    pub require_json: bool,
    /// Max output tokens (override default)
    pub max_tokens: Option<usize>,
    /// Temperature for generation
    pub temperature: f32,
    /// Enable red-flag checking
    pub red_flag_check: bool,
}

impl Default for SpawnFlags {
    fn default() -> Self {
        Self {
            require_json: false,
            max_tokens: None,
            temperature: 0.1,
            red_flag_check: true,
        }
    }
}

