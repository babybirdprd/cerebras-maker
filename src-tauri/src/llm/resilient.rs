// Cerebras-MAKER: Resilient LLM Provider
// Implements retry logic with exponential backoff and fallback chain
// for robust LLM operations when primary providers fail.

use super::{LlmConfig, LlmProvider, LlmResponse, Message};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for retry behavior with exponential backoff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts per provider
    pub max_retries: u32,
    /// Base delay between retries (will be multiplied exponentially)
    pub base_delay_ms: u64,
    /// Maximum delay between retries (caps exponential growth)
    pub max_delay_ms: u64,
    /// Jitter factor (0.0 to 1.0) to add randomness to delays
    pub jitter_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 1000,    // 1 second
            max_delay_ms: 30000,    // 30 seconds
            jitter_factor: 0.2,     // 20% jitter
        }
    }
}

impl RetryConfig {
    /// Create a retry config for fast-failing scenarios
    pub fn fast() -> Self {
        Self {
            max_retries: 2,
            base_delay_ms: 500,
            max_delay_ms: 5000,
            jitter_factor: 0.1,
        }
    }

    /// Create a retry config for high-reliability scenarios
    pub fn resilient() -> Self {
        Self {
            max_retries: 5,
            base_delay_ms: 2000,
            max_delay_ms: 60000,
            jitter_factor: 0.25,
        }
    }

    /// Calculate delay for a given attempt (0-indexed)
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let exponential_delay = self.base_delay_ms * 2u64.pow(attempt);
        let capped_delay = exponential_delay.min(self.max_delay_ms);

        // Add jitter to prevent thundering herd
        let jitter_range = (capped_delay as f64 * self.jitter_factor) as u64;
        let jitter = if jitter_range > 0 {
            fastrand::u64(0..jitter_range)
        } else {
            0
        };

        Duration::from_millis(capped_delay + jitter)
    }
}

/// Error classification for determining retry behavior
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    /// Transient errors that may succeed on retry (network issues, rate limits)
    Transient,
    /// Permanent errors that won't succeed on retry (auth failures, invalid requests)
    Permanent,
    /// Unknown error type - treat as transient with limited retries
    Unknown,
}

impl ErrorKind {
    /// Classify an error based on its string representation
    /// This heuristic approach works with anyhow::Error's Display
    pub fn classify(error: &anyhow::Error) -> Self {
        let error_str = error.to_string().to_lowercase();

        // Network and timeout errors are transient
        if error_str.contains("timeout")
            || error_str.contains("connection")
            || error_str.contains("network")
            || error_str.contains("timed out")
            || error_str.contains("temporarily")
            || error_str.contains("service unavailable")
            || error_str.contains("503")
            || error_str.contains("502")
            || error_str.contains("504")
        {
            return Self::Transient;
        }

        // Rate limiting is transient
        if error_str.contains("rate limit")
            || error_str.contains("too many requests")
            || error_str.contains("429")
            || error_str.contains("quota")
        {
            return Self::Transient;
        }

        // Server errors (5xx) are generally transient
        if error_str.contains("500")
            || error_str.contains("internal server error")
        {
            return Self::Transient;
        }

        // Authentication errors are permanent
        if error_str.contains("unauthorized")
            || error_str.contains("401")
            || error_str.contains("403")
            || error_str.contains("forbidden")
            || error_str.contains("invalid api key")
            || error_str.contains("authentication")
        {
            return Self::Permanent;
        }

        // Invalid request errors are permanent
        if error_str.contains("bad request")
            || error_str.contains("400")
            || error_str.contains("invalid")
            || error_str.contains("not found")
            || error_str.contains("404")
        {
            return Self::Permanent;
        }

        // Model-specific errors are often permanent
        if error_str.contains("model not found")
            || error_str.contains("unsupported model")
            || error_str.contains("context length")
            || error_str.contains("token limit")
        {
            return Self::Permanent;
        }

        Self::Unknown
    }

    /// Whether this error type should trigger a retry
    pub fn should_retry(&self) -> bool {
        matches!(self, Self::Transient | Self::Unknown)
    }
}

/// Statistics for resilient provider operations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResilienceStats {
    /// Total number of requests made
    pub total_requests: u64,
    /// Number of successful requests
    pub successful_requests: u64,
    /// Number of retries performed
    pub retry_count: u64,
    /// Number of fallbacks to secondary providers
    pub fallback_count: u64,
    /// Number of permanent failures
    pub permanent_failures: u64,
}

/// A resilient LLM provider that wraps multiple providers with retry and fallback logic
pub struct ResilientLlmProvider {
    /// Ordered list of provider configurations (primary first, then fallbacks)
    provider_configs: Vec<LlmConfig>,
    /// Retry configuration
    retry_config: RetryConfig,
    /// Statistics tracking (protected by mutex for interior mutability)
    stats: std::sync::Mutex<ResilienceStats>,
}

impl ResilientLlmProvider {
    /// Create a new resilient provider with a primary config and fallback configs
    pub fn new(primary: LlmConfig, fallbacks: Vec<LlmConfig>, retry_config: RetryConfig) -> Self {
        let mut provider_configs = vec![primary];
        provider_configs.extend(fallbacks);

        Self {
            provider_configs,
            retry_config,
            stats: std::sync::Mutex::new(ResilienceStats::default()),
        }
    }

    /// Create a resilient provider with just a primary (no fallbacks)
    pub fn with_primary(config: LlmConfig) -> Self {
        Self::new(config, vec![], RetryConfig::default())
    }

    /// Create from a list of configs with custom retry settings
    pub fn with_configs(configs: Vec<LlmConfig>, retry_config: RetryConfig) -> Result<Self, anyhow::Error> {
        if configs.is_empty() {
            anyhow::bail!("At least one provider configuration is required");
        }

        let primary = configs[0].clone();
        let fallbacks = configs[1..].to_vec();

        Ok(Self::new(primary, fallbacks, retry_config))
    }

    /// Get current resilience statistics
    pub fn stats(&self) -> ResilienceStats {
        self.stats.lock().unwrap().clone()
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        let mut stats = self.stats.lock().unwrap();
        *stats = ResilienceStats::default();
    }

    /// Complete a request with retry and fallback logic
    pub async fn complete(&self, messages: Vec<Message>) -> Result<LlmResponse, anyhow::Error> {
        self.increment_total_requests();

        let mut last_error: Option<anyhow::Error> = None;

        // Try each provider in order
        for (provider_idx, config) in self.provider_configs.iter().enumerate() {
            let provider_name = format!("{:?}", config.provider);

            if provider_idx > 0 {
                self.increment_fallback_count();
                log::warn!(
                    "Falling back to provider {} ({}/{})",
                    provider_name,
                    provider_idx + 1,
                    self.provider_configs.len()
                );
            }

            // Try this provider with retries
            match self.try_provider_with_retries(config, &messages, &provider_name).await {
                Ok(response) => {
                    self.increment_successful_requests();
                    if provider_idx > 0 {
                        log::info!(
                            "Fallback to {} succeeded after {} provider(s) failed",
                            provider_name,
                            provider_idx
                        );
                    }
                    return Ok(response);
                }
                Err(e) => {
                    let error_kind = ErrorKind::classify(&e);
                    log::warn!(
                        "Provider {} failed with {:?} error: {}",
                        provider_name,
                        error_kind,
                        e
                    );

                    // If it's a permanent error, don't try fallbacks for same issue
                    if error_kind == ErrorKind::Permanent {
                        self.increment_permanent_failures();
                        // Continue to next provider anyway - might be config-specific
                    }

                    last_error = Some(e);
                }
            }
        }

        // All providers failed
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("No providers configured")))
    }
}


impl ResilientLlmProvider {
    /// Try a single provider with retry logic
    async fn try_provider_with_retries(
        &self,
        config: &LlmConfig,
        messages: &[Message],
        provider_name: &str,
    ) -> Result<LlmResponse, anyhow::Error> {
        let provider = LlmProvider::new(config.clone())?;
        let mut last_error: Option<anyhow::Error> = None;

        for attempt in 0..=self.retry_config.max_retries {
            if attempt > 0 {
                let delay = self.retry_config.calculate_delay(attempt - 1);
                log::debug!(
                    "Retry attempt {}/{} for {} after {:?} delay",
                    attempt,
                    self.retry_config.max_retries,
                    provider_name,
                    delay
                );
                self.increment_retry_count();
                tokio::time::sleep(delay).await;
            }

            match provider.complete(messages.to_vec()).await {
                Ok(response) => {
                    if attempt > 0 {
                        log::info!(
                            "Provider {} succeeded after {} retries",
                            provider_name,
                            attempt
                        );
                    }
                    return Ok(response);
                }
                Err(e) => {
                    let error_kind = ErrorKind::classify(&e);

                    log::debug!(
                        "Provider {} attempt {}/{} failed ({:?}): {}",
                        provider_name,
                        attempt + 1,
                        self.retry_config.max_retries + 1,
                        error_kind,
                        e
                    );

                    // Don't retry permanent errors
                    if !error_kind.should_retry() {
                        log::warn!(
                            "Provider {} encountered permanent error, not retrying: {}",
                            provider_name,
                            e
                        );
                        return Err(e);
                    }

                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            anyhow::anyhow!("Provider {} exhausted all retries", provider_name)
        }))
    }

    // Statistics helper methods
    fn increment_total_requests(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.total_requests += 1;
        }
    }

    fn increment_successful_requests(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.successful_requests += 1;
        }
    }

    fn increment_retry_count(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.retry_count += 1;
        }
    }

    fn increment_fallback_count(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.fallback_count += 1;
        }
    }

    fn increment_permanent_failures(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.permanent_failures += 1;
        }
    }
}

/// Builder pattern for creating ResilientLlmProvider
pub struct ResilientLlmProviderBuilder {
    primary: Option<LlmConfig>,
    fallbacks: Vec<LlmConfig>,
    retry_config: RetryConfig,
}

impl Default for ResilientLlmProviderBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ResilientLlmProviderBuilder {
    pub fn new() -> Self {
        Self {
            primary: None,
            fallbacks: Vec::new(),
            retry_config: RetryConfig::default(),
        }
    }

    /// Set the primary provider
    pub fn primary(mut self, config: LlmConfig) -> Self {
        self.primary = Some(config);
        self
    }

    /// Add a fallback provider
    pub fn fallback(mut self, config: LlmConfig) -> Self {
        self.fallbacks.push(config);
        self
    }

    /// Add multiple fallback providers
    pub fn fallbacks(mut self, configs: Vec<LlmConfig>) -> Self {
        self.fallbacks.extend(configs);
        self
    }

    /// Set retry configuration
    pub fn retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    /// Build the resilient provider
    pub fn build(self) -> Result<ResilientLlmProvider, anyhow::Error> {
        let primary = self.primary
            .ok_or_else(|| anyhow::anyhow!("Primary provider configuration is required"))?;

        Ok(ResilientLlmProvider::new(primary, self.fallbacks, self.retry_config))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_delay_calculation() {
        let config = RetryConfig {
            max_retries: 3,
            base_delay_ms: 1000,
            max_delay_ms: 10000,
            jitter_factor: 0.0, // No jitter for deterministic test
        };

        // Attempt 0: 1000ms
        let delay0 = config.calculate_delay(0);
        assert_eq!(delay0.as_millis(), 1000);

        // Attempt 1: 2000ms
        let delay1 = config.calculate_delay(1);
        assert_eq!(delay1.as_millis(), 2000);

        // Attempt 2: 4000ms
        let delay2 = config.calculate_delay(2);
        assert_eq!(delay2.as_millis(), 4000);

        // Attempt 3: 8000ms
        let delay3 = config.calculate_delay(3);
        assert_eq!(delay3.as_millis(), 8000);

        // Attempt 4: Would be 16000ms but capped at 10000ms
        let delay4 = config.calculate_delay(4);
        assert_eq!(delay4.as_millis(), 10000);
    }

    #[test]
    fn test_error_classification() {
        // Transient errors
        assert_eq!(
            ErrorKind::classify(&anyhow::anyhow!("Connection timeout")),
            ErrorKind::Transient
        );
        assert_eq!(
            ErrorKind::classify(&anyhow::anyhow!("Rate limit exceeded (429)")),
            ErrorKind::Transient
        );
        assert_eq!(
            ErrorKind::classify(&anyhow::anyhow!("Service temporarily unavailable")),
            ErrorKind::Transient
        );

        // Permanent errors
        assert_eq!(
            ErrorKind::classify(&anyhow::anyhow!("Unauthorized: Invalid API key")),
            ErrorKind::Permanent
        );
        assert_eq!(
            ErrorKind::classify(&anyhow::anyhow!("Bad request: invalid parameter")),
            ErrorKind::Permanent
        );

        // Unknown errors
        assert_eq!(
            ErrorKind::classify(&anyhow::anyhow!("Something went wrong")),
            ErrorKind::Unknown
        );
    }

    #[test]
    fn test_should_retry() {
        assert!(ErrorKind::Transient.should_retry());
        assert!(ErrorKind::Unknown.should_retry());
        assert!(!ErrorKind::Permanent.should_retry());
    }
}

