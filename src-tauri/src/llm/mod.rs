// Cerebras-MAKER: Unified LLM API Layer
// Provides a consistent interface for all LLM operations using rig-core

pub mod provider;
pub mod prompts;

pub use provider::{LlmProvider, LlmConfig, LlmResponse, Message, Role, ProviderType};
pub use prompts::{PromptTemplate, PromptContext, SystemPrompts};

use std::sync::Arc;
use tokio::sync::RwLock;

/// Global LLM provider instance
static LLM_PROVIDER: once_cell::sync::Lazy<Arc<RwLock<Option<LlmProvider>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(None)));

/// Initialize the global LLM provider
pub async fn init_provider(config: LlmConfig) -> Result<(), anyhow::Error> {
    let provider = LlmProvider::new(config)?;
    let mut guard = LLM_PROVIDER.write().await;
    *guard = Some(provider);
    Ok(())
}

/// Get the global LLM provider
pub async fn get_provider() -> Option<LlmProvider> {
    let guard = LLM_PROVIDER.read().await;
    guard.clone()
}

/// Complete a prompt using the global provider
pub async fn complete(messages: Vec<Message>) -> Result<LlmResponse, anyhow::Error> {
    let guard = LLM_PROVIDER.read().await;
    let provider = guard.as_ref().ok_or_else(|| anyhow::anyhow!("LLM provider not initialized"))?;
    provider.complete(messages).await
}

/// Complete with a system prompt and user message
pub async fn complete_with_system(system: &str, user: &str) -> Result<String, anyhow::Error> {
    let messages = vec![
        Message::system(system),
        Message::user(user),
    ];
    let response = complete(messages).await?;
    Ok(response.content)
}

