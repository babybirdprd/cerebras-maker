// Cerebras-MAKER: Agent System
// PRD Section 2: Architecture - The Dual-Graph System

pub mod interrogator;
pub mod architect;
pub mod orchestrator;
pub mod context_engineer;

// Re-exports
pub use interrogator::Interrogator;
pub use architect::Architect;
pub use orchestrator::Orchestrator;
pub use context_engineer::ContextEngineer;

use serde::{Deserialize, Serialize};

/// Base trait for all agents
pub trait Agent: Send + Sync {
    /// Get the agent's name
    fn name(&self) -> &str;
    
    /// Get the agent's system prompt
    fn system_prompt(&self) -> &str;
}

/// Agent output types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentOutput {
    /// Natural language plan
    Plan(PlanOutput),
    /// Architecture specification
    Architecture(ArchitectureOutput),
    /// Rhai script for execution
    Script(ScriptOutput),
    /// Question requiring user input
    Question(QuestionOutput),
}

/// Plan output from planning phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanOutput {
    pub plan_id: String,
    pub title: String,
    pub description: String,
    pub micro_tasks: Vec<MicroTask>,
    pub dependencies: Vec<(String, String)>, // (task_id, depends_on_id)
}

/// A micro-task (atomic unit of work)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroTask {
    pub id: String,
    pub description: String,
    pub atom_type: String,
    pub estimated_complexity: u8, // 1-5
    pub seed_symbols: Vec<String>,
}

/// Architecture specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureOutput {
    pub arch_id: String,
    pub layers: Vec<LayerSpec>,
    pub modules: Vec<ModuleSpec>,
    pub constraints: Vec<String>,
}

/// Layer specification for architecture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerSpec {
    pub name: String,
    pub level: u8,
    pub allowed_deps: Vec<String>,
}

/// Module specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSpec {
    pub name: String,
    pub layer: String,
    pub files: Vec<String>,
}

/// Script output from orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptOutput {
    pub script_id: String,
    pub task_id: String,
    pub rhai_code: String,
}

/// Question requiring user input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionOutput {
    pub question_id: String,
    pub question: String,
    pub context: String,
    pub options: Option<Vec<String>>,
    pub ambiguity_score: f32,
}

/// Agent execution context
#[derive(Debug, Clone)]
pub struct AgentContext {
    pub workspace_path: String,
    pub issue_id: Option<String>,
    pub previous_outputs: Vec<AgentOutput>,
}

impl AgentContext {
    pub fn new(workspace_path: &str) -> Self {
        Self {
            workspace_path: workspace_path.to_string(),
            issue_id: None,
            previous_outputs: Vec::new(),
        }
    }
    
    pub fn with_issue(mut self, issue_id: &str) -> Self {
        self.issue_id = Some(issue_id.to_string());
        self
    }
}

