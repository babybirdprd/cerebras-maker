// Cerebras-MAKER: LLM Provider Abstraction
// Unified interface for all LLM operations:
// - Cerebras: via cerebras-rs (native high-speed client)
// - Anthropic: via rig-core
// - OpenAI/OpenRouter/Compatible: via rig-core with custom base_url

use serde::{Deserialize, Serialize};
use std::env;

/// Supported LLM providers
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum ProviderType {
    #[default]
    OpenAI,
    Anthropic,
    Cerebras,
    /// OpenRouter.ai - unified API for multiple models
    OpenRouter,
    /// Any OpenAI-compatible API (requires base_url)
    OpenAICompatible,
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
    /// Cerebras configuration using cerebras-rs native client
    pub fn cerebras() -> Self {
        Self {
            provider: ProviderType::Cerebras,
            model: "llama-4-scout-17b-16e-instruct".to_string(),
            api_key: env::var("CEREBRAS_API_KEY").ok(),
            base_url: None, // cerebras-rs handles the endpoint internally
            temperature: 0.7,
            max_tokens: 8192,
        }
    }

    /// Anthropic Claude configuration
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

    /// OpenRouter configuration (access multiple models via one API)
    pub fn openrouter() -> Self {
        Self {
            provider: ProviderType::OpenRouter,
            model: "anthropic/claude-sonnet-4".to_string(),
            api_key: env::var("OPENROUTER_API_KEY").ok(),
            base_url: Some("https://openrouter.ai/api/v1".to_string()),
            temperature: 0.7,
            max_tokens: 4096,
        }
    }

    /// Custom OpenAI-compatible API (e.g., local LLM servers)
    pub fn openai_compatible(base_url: &str, model: &str, api_key: Option<String>) -> Self {
        Self {
            provider: ProviderType::OpenAICompatible,
            model: model.to_string(),
            api_key,
            base_url: Some(base_url.to_string()),
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
        match self.config.provider {
            ProviderType::Cerebras => self.call_cerebras(system, user).await,
            ProviderType::Anthropic => self.call_anthropic(system, user).await,
            ProviderType::OpenAI => self.call_openai(system, user, None).await,
            ProviderType::OpenRouter => self.call_openrouter(system, user).await,
            ProviderType::OpenAICompatible => {
                let base_url = self.config.base_url.as_deref()
                    .ok_or_else(|| anyhow::anyhow!("base_url required for OpenAICompatible"))?;
                self.call_openai(system, user, Some(base_url)).await
            }
        }
    }

    /// Call Cerebras using native cerebras-rs client (high-speed)
    async fn call_cerebras(&self, system: &str, user: &str) -> Result<String, anyhow::Error> {
        use cerebras_rs::{Client, ChatCompletionRequest, ModelIdentifier};

        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Cerebras API key not configured"))?;

        let client = Client::new(api_key);

        // Map model string to ModelIdentifier
        let model = match self.config.model.as_str() {
            "llama-4-scout-17b-16e-instruct" => ModelIdentifier::Llama4Scout17b16eInstruct,
            "llama-3.1-8b" | "llama3.1-8b" => ModelIdentifier::Llama3Period18b,
            "llama-3.3-70b" | "llama3.3-70b" => ModelIdentifier::Llama3Period370b,
            "qwen-3-32b" | "qwen3-32b" => ModelIdentifier::Qwen332b,
            "deepseek-r1-distill-llama-70b" => ModelIdentifier::DeepseekR1DistillLlama70b,
            _ => ModelIdentifier::Llama4Scout17b16eInstruct, // Default
        };

        let request = ChatCompletionRequest::builder(model)
            .system_message(system)
            .user_message(user)
            .temperature(self.config.temperature as f64)
            .max_tokens(self.config.max_tokens)
            .build();

        let response = client.chat_completion(request).await?;

        // Extract content from response
        let content = response.choices
            .and_then(|choices| choices.into_iter().next())
            .and_then(|choice| choice.message)
            .map(|msg| msg.content)
            .unwrap_or_default();

        Ok(content)
    }

    /// Call Anthropic using rig-core
    async fn call_anthropic(&self, system: &str, user: &str) -> Result<String, anyhow::Error> {
        use rig::client::{CompletionClient, ProviderClient};
        use rig::completion::Prompt;
        use rig::providers::anthropic;

        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Anthropic API key not configured"))?;

        // Set API key in environment for rig-core
        std::env::set_var("ANTHROPIC_API_KEY", api_key);

        let client = anthropic::Client::from_env();
        let agent = client
            .agent(&self.config.model)
            .preamble(system)
            .build();

        let response = agent.prompt(user).await?;
        Ok(response)
    }

    /// Call OpenRouter using rig-core's native openrouter provider
    async fn call_openrouter(&self, system: &str, user: &str) -> Result<String, anyhow::Error> {
        use rig::client::CompletionClient;
        use rig::completion::Prompt;
        use rig::providers::openrouter;

        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("OpenRouter API key not configured"))?;

        let client: openrouter::Client = openrouter::Client::new(api_key)?;
        let agent = client
            .agent(&self.config.model)
            .preamble(system)
            .build();

        let response = agent.prompt(user).await?;
        Ok(response)
    }

    /// Call OpenAI or OpenAI-compatible API using rig-core
    async fn call_openai(&self, system: &str, user: &str, base_url: Option<&str>) -> Result<String, anyhow::Error> {
        use rig::client::CompletionClient;
        use rig::completion::Prompt;
        use rig::providers::openai;

        let api_key = self.config.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("OpenAI API key not configured"))?;

        // Create client with optional custom base URL using builder pattern
        let client: openai::Client = match base_url {
            Some(url) => openai::Client::builder()
                .api_key(api_key)
                .base_url(url)
                .build()?,
            None => openai::Client::new(api_key)?,
        };

        let agent = client
            .agent(&self.config.model)
            .preamble(system)
            .build();

        let response = agent.prompt(user).await?;
        Ok(response)
    }
}

