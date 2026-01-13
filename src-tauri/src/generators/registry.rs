// Cerebras-MAKER: Generator Registry
// Plugin system for registering and selecting script generators

use super::{GenerationResult, GeneratorError, ScriptGenerator};
use crate::agents::MicroTask;
use crate::llm::PromptContext;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// A registered generator plugin
pub struct GeneratorPlugin {
    pub generator: Arc<dyn ScriptGenerator>,
    pub enabled: bool,
}

impl GeneratorPlugin {
    pub fn new(generator: impl ScriptGenerator + 'static) -> Self {
        Self {
            generator: Arc::new(generator),
            enabled: true,
        }
    }
}

/// Registry for script generators
pub struct GeneratorRegistry {
    generators: RwLock<HashMap<String, GeneratorPlugin>>,
}

impl Default for GeneratorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl GeneratorRegistry {
    pub fn new() -> Self {
        Self {
            generators: RwLock::new(HashMap::new()),
        }
    }

    /// Register a new generator
    pub async fn register(&self, generator: impl ScriptGenerator + 'static) {
        let name = generator.name().to_string();
        let plugin = GeneratorPlugin::new(generator);
        let mut generators = self.generators.write().await;
        generators.insert(name, plugin);
    }

    /// Unregister a generator by name
    pub async fn unregister(&self, name: &str) -> bool {
        let mut generators = self.generators.write().await;
        generators.remove(name).is_some()
    }

    /// Enable or disable a generator
    pub async fn set_enabled(&self, name: &str, enabled: bool) -> bool {
        let mut generators = self.generators.write().await;
        if let Some(plugin) = generators.get_mut(name) {
            plugin.enabled = enabled;
            true
        } else {
            false
        }
    }

    /// List all registered generators
    pub async fn list(&self) -> Vec<(String, String, bool)> {
        let generators = self.generators.read().await;
        generators
            .iter()
            .map(|(name, plugin)| {
                (
                    name.clone(),
                    plugin.generator.description().to_string(),
                    plugin.enabled,
                )
            })
            .collect()
    }

    /// Find the best generator for a task
    pub async fn find_generator(&self, task: &MicroTask) -> Option<Arc<dyn ScriptGenerator>> {
        let generators = self.generators.read().await;

        let mut candidates: Vec<_> = generators
            .values()
            .filter(|p| p.enabled && p.generator.can_handle(task))
            .collect();

        // Sort by priority (highest first)
        candidates.sort_by(|a, b| b.generator.priority().cmp(&a.generator.priority()));

        candidates.first().map(|p| Arc::clone(&p.generator))
    }

    /// Generate a script using the best available generator
    pub async fn generate(
        &self,
        task: &MicroTask,
        context: &PromptContext,
    ) -> Result<GenerationResult, GeneratorError> {
        let generator = self
            .find_generator(task)
            .await
            .ok_or(GeneratorError::NoGeneratorFound)?;

        generator.generate(task, context).await
    }

    /// Generate scripts for multiple tasks
    pub async fn generate_batch(
        &self,
        tasks: &[MicroTask],
        context: &PromptContext,
    ) -> Vec<Result<GenerationResult, GeneratorError>> {
        let mut results = Vec::with_capacity(tasks.len());

        for task in tasks {
            let result = self.generate(task, context).await;
            results.push(result);
        }

        results
    }
}

/// Global registry instance
static REGISTRY: once_cell::sync::Lazy<GeneratorRegistry> =
    once_cell::sync::Lazy::new(GeneratorRegistry::new);

/// Get the global generator registry
pub fn global_registry() -> &'static GeneratorRegistry {
    &REGISTRY
}

