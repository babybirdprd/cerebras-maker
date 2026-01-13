// Cerebras-MAKER: Rhai Script Generator
// LLM-powered generator that creates Rhai scripts from task descriptions

use super::{GenerationMetadata, GenerationResult, GeneratorError, ScriptGenerator};
use crate::agents::{MicroTask, ScriptOutput};
use crate::llm::{self, PromptContext, PromptTemplate, SystemPrompts};
use async_trait::async_trait;
use std::time::Instant;

/// LLM-powered Rhai script generator
pub struct RhaiScriptGenerator {
    /// Template for generating scripts
    template: PromptTemplate,
    /// Default k-threshold for consensus
    default_k: usize,
}

impl Default for RhaiScriptGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl RhaiScriptGenerator {
    pub fn new() -> Self {
        let template = PromptTemplate::new(
            "rhai_script",
            r#"Generate a Rhai script for the following task:

## Task ID: {{task_id}}
## Description: {{task_description}}
## Atom Type: {{atom_type}}
## Seed Symbols: {{seed_symbols}}

## Code Context:
{{code_context}}

## Requirements:
1. Create a snapshot before any modifications
2. Use consensus voting with k={{k_threshold}}
3. Check for red flags after code generation
4. Handle errors and rollback on failure
5. Return the result on success

Generate ONLY the Rhai script code, no explanations."#,
        );

        Self {
            template,
            default_k: 3,
        }
    }

    pub fn with_k_threshold(mut self, k: usize) -> Self {
        self.default_k = k;
        self
    }

    fn build_prompt(&self, task: &MicroTask, context: &PromptContext) -> Result<String, GeneratorError> {
        let mut vars = context.to_vars();
        vars.insert("task_id".to_string(), task.id.clone());
        vars.insert("task_description".to_string(), task.description.clone());
        vars.insert("atom_type".to_string(), task.atom_type.clone());
        vars.insert("seed_symbols".to_string(), task.seed_symbols.join(", "));
        vars.insert("k_threshold".to_string(), self.default_k.to_string());

        if !vars.contains_key("code_context") {
            vars.insert("code_context".to_string(), "(no context provided)".to_string());
        }

        self.template
            .render(&vars)
            .map_err(|e| GeneratorError::ContextError(e))
    }

    fn extract_script(&self, response: &str) -> String {
        // Try to extract code from markdown code blocks
        if let Some(start) = response.find("```rhai") {
            if let Some(end) = response[start + 7..].find("```") {
                return response[start + 7..start + 7 + end].trim().to_string();
            }
        }

        // Try generic code block
        if let Some(start) = response.find("```") {
            let after_start = start + 3;
            // Skip language identifier if present
            let code_start = response[after_start..]
                .find('\n')
                .map(|i| after_start + i + 1)
                .unwrap_or(after_start);

            if let Some(end) = response[code_start..].find("```") {
                return response[code_start..code_start + end].trim().to_string();
            }
        }

        // Return as-is if no code blocks found
        response.trim().to_string()
    }
}

#[async_trait]
impl ScriptGenerator for RhaiScriptGenerator {
    fn name(&self) -> &str {
        "rhai_llm_generator"
    }

    fn description(&self) -> &str {
        "LLM-powered generator that creates Rhai scripts from task descriptions"
    }

    fn can_handle(&self, _task: &MicroTask) -> bool {
        // This generator can handle any task
        true
    }

    fn priority(&self) -> i32 {
        // Default priority
        0
    }

    async fn generate(
        &self,
        task: &MicroTask,
        context: &PromptContext,
    ) -> Result<GenerationResult, GeneratorError> {
        let start = Instant::now();

        // Build the prompt
        let user_prompt = self.build_prompt(task, context)?;
        let system_prompt = SystemPrompts::script_generator();

        // Call LLM
        let response = llm::complete_with_system(system_prompt, &user_prompt)
            .await
            .map_err(|e| GeneratorError::LlmError(e.to_string()))?;

        // Extract the script
        let rhai_code = self.extract_script(&response);

        let elapsed = start.elapsed();

        Ok(GenerationResult {
            script: ScriptOutput {
                script_id: format!("gen_{}", uuid::Uuid::new_v4()),
                task_id: task.id.clone(),
                rhai_code,
            },
            confidence: 0.85, // Could be improved with validation
            warnings: vec![],
            metadata: GenerationMetadata {
                generator_name: self.name().to_string(),
                generation_time_ms: elapsed.as_millis() as u64,
                prompt_tokens: None,
                completion_tokens: None,
                model_used: None,
            },
        })
    }
}

