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
    /// Tester atom - generates and runs tests
    Tester,
    /// Architect atom - designs structures and interfaces
    Architect,
    /// GritsAnalyzer atom - performs topology analysis
    GritsAnalyzer,
    /// RLMProcessor atom - processes large contexts using Recursive LM patterns
    /// Based on: "Recursive Language Models: Scaling Context with Recursive Prompt Decomposition"
    RLMProcessor,
    /// WebResearcher atom - performs web research using crawl4ai
    /// Capable of crawling URLs, researching documentation, and extracting content
    WebResearcher,
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
            AtomType::Tester => {
                "You are a Tester Atom. Your only job is to write test code for the given implementation. \
                 Return test code that follows the existing test patterns. Include assertions."
            }
            AtomType::Architect => {
                "You are an Architect Atom. Your only job is to design structures and interfaces. \
                 Return a JSON object with 'interfaces', 'structs', and 'relationships' arrays."
            }
            AtomType::GritsAnalyzer => {
                "You are a GritsAnalyzer Atom. Your only job is to analyze code topology. \
                 Return a JSON object with 'cycles', 'layers', 'violations', and 'red_flags' arrays."
            }
            AtomType::RLMProcessor => {
                "You are an RLM Processor Atom. You can interact with large contexts programmatically. \
                 Use peek_context(), chunk_context(), and llm_query() to process context variables. \
                 Return a JSON object with 'answer', 'iterations', and 'sub_calls' fields."
            }
            AtomType::WebResearcher => {
                "You are a WebResearcher Atom. Your only job is to gather information from the web. \
                 Use crawl_url(url) to fetch page content, research_docs(topic) to search multiple sources, \
                 and extract_content(url, selector) for structured extraction. \
                 Return a JSON object with 'sources', 'findings', and 'summary' fields."
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
            AtomType::Tester => 2000,
            AtomType::Architect => 1500,
            AtomType::GritsAnalyzer => 1000,
            AtomType::RLMProcessor => 4000, // RLM needs more output for complex reasoning
            AtomType::WebResearcher => 3000, // Web research may return substantial findings
        }
    }

    /// Convert from string representation
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "search" => Some(AtomType::Search),
            "coder" => Some(AtomType::Coder),
            "reviewer" => Some(AtomType::Reviewer),
            "planner" => Some(AtomType::Planner),
            "validator" => Some(AtomType::Validator),
            "tester" => Some(AtomType::Tester),
            "architect" => Some(AtomType::Architect),
            "gritsanalyzer" | "grits" => Some(AtomType::GritsAnalyzer),
            "rlmprocessor" | "rlm" => Some(AtomType::RLMProcessor),
            "webresearcher" | "researcher" | "web" => Some(AtomType::WebResearcher),
            _ => None,
        }
    }

    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            AtomType::Search => "Search",
            AtomType::Coder => "Coder",
            AtomType::Reviewer => "Reviewer",
            AtomType::Planner => "Planner",
            AtomType::Validator => "Validator",
            AtomType::Tester => "Tester",
            AtomType::Architect => "Architect",
            AtomType::GritsAnalyzer => "GritsAnalyzer",
            AtomType::RLMProcessor => "RLMProcessor",
            AtomType::WebResearcher => "WebResearcher",
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

impl SpawnFlags {
    /// HIGH-11: Validate spawn flags before execution
    pub fn validate(&self) -> Result<(), String> {
        // Temperature must be between 0.0 and 2.0
        if self.temperature < 0.0 || self.temperature > 2.0 {
            return Err(format!(
                "Temperature must be between 0.0 and 2.0, got {}",
                self.temperature
            ));
        }

        // Max tokens must be reasonable if specified
        if let Some(max_tokens) = self.max_tokens {
            if max_tokens == 0 {
                return Err("max_tokens cannot be 0".to_string());
            }
            if max_tokens > 100_000 {
                return Err(format!(
                    "max_tokens {} exceeds maximum allowed (100,000)",
                    max_tokens
                ));
            }
        }

        Ok(())
    }

    /// Create validated spawn flags, returning error if invalid
    pub fn new_validated(
        require_json: bool,
        max_tokens: Option<usize>,
        temperature: f32,
        red_flag_check: bool,
    ) -> Result<Self, String> {
        let flags = Self {
            require_json,
            max_tokens,
            temperature,
            red_flag_check,
        };
        flags.validate()?;
        Ok(flags)
    }
}

