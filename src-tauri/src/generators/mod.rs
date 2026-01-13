// Cerebras-MAKER: Script Generator Framework
// Plugin architecture for generating Rhai scripts from task descriptions

pub mod registry;
pub mod rhai_generator;
pub mod task_generator;

pub use registry::{GeneratorRegistry, GeneratorPlugin};
pub use rhai_generator::RhaiScriptGenerator;
pub use task_generator::TaskScriptGenerator;

use crate::agents::{MicroTask, ScriptOutput};
use crate::llm::PromptContext;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Result of script generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResult {
    pub script: ScriptOutput,
    pub confidence: f32,
    pub warnings: Vec<String>,
    pub metadata: GenerationMetadata,
}

/// Metadata about the generation process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationMetadata {
    pub generator_name: String,
    pub generation_time_ms: u64,
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub model_used: Option<String>,
}

/// Trait for script generators
#[async_trait]
pub trait ScriptGenerator: Send + Sync {
    /// Get the generator's name
    fn name(&self) -> &str;

    /// Get the generator's description
    fn description(&self) -> &str;

    /// Check if this generator can handle the given task
    fn can_handle(&self, task: &MicroTask) -> bool;

    /// Generate a Rhai script for the given task
    async fn generate(
        &self,
        task: &MicroTask,
        context: &PromptContext,
    ) -> Result<GenerationResult, GeneratorError>;

    /// Priority for generator selection (higher = preferred)
    fn priority(&self) -> i32 {
        0
    }
}

/// Errors that can occur during script generation
#[derive(Debug, thiserror::Error)]
pub enum GeneratorError {
    #[error("LLM error: {0}")]
    LlmError(String),

    #[error("Invalid task: {0}")]
    InvalidTask(String),

    #[error("Generation failed: {0}")]
    GenerationFailed(String),

    #[error("No suitable generator found for task")]
    NoGeneratorFound,

    #[error("Context error: {0}")]
    ContextError(String),
}

impl From<anyhow::Error> for GeneratorError {
    fn from(err: anyhow::Error) -> Self {
        GeneratorError::GenerationFailed(err.to_string())
    }
}

