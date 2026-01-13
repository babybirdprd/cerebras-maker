// HTTP-only web crawler for Cerebras-MAKER
// Lightweight alternative to browser-based crawling

use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use crate::markdown::DefaultMarkdownGenerator;
use crate::content_filter::PruningContentFilter;

/// Configuration for HTTP crawling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpCrawlConfig {
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// User agent string
    pub user_agent: String,
    /// Whether to follow redirects
    pub follow_redirects: bool,
    /// Maximum redirects to follow
    pub max_redirects: usize,
    /// Whether to convert HTML to markdown
    pub convert_to_markdown: bool,
    /// Whether to apply content filtering
    pub filter_content: bool,
}

impl Default for HttpCrawlConfig {
    fn default() -> Self {
        Self {
            timeout_secs: 30,
            user_agent: "Mozilla/5.0 (compatible; Crawl4AI-RS/0.1; +https://github.com/babybirdprd/crawl4ai)".to_string(),
            follow_redirects: true,
            max_redirects: 10,
            convert_to_markdown: true,
            filter_content: true,
        }
    }
}

/// Result of an HTTP crawl
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpCrawlResult {
    /// The URL that was crawled
    pub url: String,
    /// HTTP status code
    pub status_code: u16,
    /// Raw HTML content
    pub html: String,
    /// Markdown content (if conversion enabled)
    pub markdown: Option<String>,
    /// Filtered/cleaned content
    pub cleaned_content: Option<String>,
    /// Page title
    pub title: Option<String>,
    /// Response headers
    pub headers: std::collections::HashMap<String, String>,
    /// Crawl duration in milliseconds
    pub duration_ms: u64,
}

/// HTTP-based web crawler
pub struct HttpCrawler {
    client: Client,
    config: HttpCrawlConfig,
}

impl HttpCrawler {
    /// Create a new HTTP crawler with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(HttpCrawlConfig::default())
    }

    /// Create a new HTTP crawler with custom configuration
    pub fn with_config(config: HttpCrawlConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .user_agent(&config.user_agent)
            .redirect(if config.follow_redirects {
                reqwest::redirect::Policy::limited(config.max_redirects)
            } else {
                reqwest::redirect::Policy::none()
            })
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        Ok(Self { client, config })
    }

    /// Crawl a single URL
    pub async fn crawl(&self, url: &str) -> Result<HttpCrawlResult> {
        let start = std::time::Instant::now();

        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| anyhow!("Request failed: {}", e))?;

        let status_code = response.status().as_u16();
        let headers: std::collections::HashMap<String, String> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let html = response
            .text()
            .await
            .map_err(|e| anyhow!("Failed to read response body: {}", e))?;

        // Extract title
        let title = self.extract_title(&html);

        // Convert to markdown if enabled
        let markdown = if self.config.convert_to_markdown {
            Some(self.html_to_markdown(&html))
        } else {
            None
        };

        // Filter content if enabled
        let cleaned_content = if self.config.filter_content {
            let filter = PruningContentFilter::default();
            Some(filter.filter(&html))
        } else {
            None
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(HttpCrawlResult {
            url: url.to_string(),
            status_code,
            html,
            markdown,
            cleaned_content,
            title,
            headers,
            duration_ms,
        })
    }

    /// Crawl multiple URLs
    pub async fn crawl_many(&self, urls: &[&str]) -> Vec<Result<HttpCrawlResult>> {
        let mut results = Vec::with_capacity(urls.len());
        for url in urls {
            results.push(self.crawl(url).await);
        }
        results
    }

    fn extract_title(&self, html: &str) -> Option<String> {
        let title_start = html.find("<title>")?;
        let title_end = html[title_start..].find("</title>")?;
        let title = &html[title_start + 7..title_start + title_end];
        Some(html_escape::decode_html_entities(title).to_string())
    }

    fn html_to_markdown(&self, html: &str) -> String {
        let generator = DefaultMarkdownGenerator::default();
        generator.generate(html)
    }
}

impl Default for HttpCrawler {
    fn default() -> Self {
        Self::new().expect("Failed to create default HttpCrawler")
    }
}

