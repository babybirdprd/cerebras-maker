// Cerebras-MAKER: LLM Provider Abstraction
// Unified interface wrapping rig-core for all LLM operations

use serde::{Deserialize, Serialize};
use std::env;

/// Supported LLM providers
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ProviderType {
    #[default]
    OpenAI,
    Anthropic,
    Cerebras,
    Ollama,
}

/// LLM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: ProviderType,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub temperature: f32,
    pub max_tokens: u32,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: ProviderType::OpenAI,
            model: "gpt-4o".to_string(),
            api_key: env::var("OPENAI_API_KEY").ok(),
            base_url: None,
            temperature: 0.7,
            max_tokens: 4096,
        }
    }
}

impl LlmConfig {
    pub fn cerebras() -> Self {
        Self {
            provider: ProviderType::Cerebras,
            model: "llama-4-scout-17b-16e-instruct".to_string(),
            api_key: env::var("CEREBRAS_API_KEY").ok(),
            base_url: Some("https://api.cerebras.ai/v1".to_string()),
            temperature: 0.7,
            max_tokens: 8192,
        }
    }

    pub fn anthropic() -> Self {
        Self {
            provider: ProviderType::Anthropic,
            model: "claude-sonnet-4-20250514".to_string(),
            api_key: env::var("ANTHROPIC_API_KEY").ok(),
            base_url: None,
            temperature: 0.7,
            max_tokens: 4096,
        }
    }
}

/// Message role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    System,
    User,
    Assistant,
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    pub fn system(content: &str) -> Self {
        Self { role: Role::System, content: content.to_string() }
    }

    pub fn user(content: &str) -> Self {
        Self { role: Role::User, content: content.to_string() }
    }

    pub fn assistant(content: &str) -> Self {
        Self { role: Role::Assistant, content: content.to_string() }
    }
}

/// LLM response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: String,
    pub model: String,
    pub tokens_used: Option<u32>,
    pub finish_reason: Option<String>,
}

/// Unified LLM provider
#[derive(Debug, Clone)]
pub struct LlmProvider {
    config: LlmConfig,
}

impl LlmProvider {
    pub fn new(config: LlmConfig) -> Result<Self, anyhow::Error> {
        if config.api_key.is_none() {
            anyhow::bail!("API key not configured for {:?}", config.provider);
        }
        Ok(Self { config })
    }

    /// Complete a chat conversation
    pub async fn complete(&self, messages: Vec<Message>) -> Result<LlmResponse, anyhow::Error> {
        // Build the prompt from messages
        let system_prompt = messages.iter()
            .find(|m| matches!(m.role, Role::System))
            .map(|m| m.content.as_str())
            .unwrap_or("");

        let user_messages: Vec<&str> = messages.iter()
            .filter(|m| matches!(m.role, Role::User))
            .map(|m| m.content.as_str())
            .collect();

        let user_prompt = user_messages.join("\n\n");

        // Use rig-core for the actual completion
        let content = self.call_llm(system_prompt, &user_prompt).await?;

        Ok(LlmResponse {
            content,
            model: self.config.model.clone(),
            tokens_used: None,
            finish_reason: Some("stop".to_string()),
        })
    }

    async fn call_llm(&self, system: &str, user: &str) -> Result<String, anyhow::Error> {
        use rig::client::{CompletionClient, ProviderClient};
        use rig::completion::Prompt;
        use rig::providers::openai;

        // Set API key in environment if provided in config
        if let Some(api_key) = &self.config.api_key {
            std::env::set_var("OPENAI_API_KEY", api_key);
        }

        // Use OpenAI-compatible API (works for Cerebras too with base_url override)
        // from_env() reads OPENAI_API_KEY from environment
        let client = openai::Client::from_env();

        let agent = client
            .agent(&self.config.model)
            .preamble(system)
            .build();

        let response = agent.prompt(user).await?;
        Ok(response)
    }
}

