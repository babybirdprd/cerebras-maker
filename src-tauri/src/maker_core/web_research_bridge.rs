//! Web Research Bridge - Safe Async-to-Sync Bridging for Crawl4AI Operations
//!
//! This module provides sync wrappers for the crawl4ai-rs async functions,
//! enabling Rhai scripts to perform web research as part of autonomous agent workflows.
//!
//! Architecture:
//! ```text
//! ┌──────────────────┐     mpsc::channel      ┌──────────────────────┐
//! │   Rhai Runtime   │ ─────────────────────> │  WebResearchWorker   │
//! │   (sync calls)   │                        │  (dedicated tokio    │
//! │                  │ <───────────────────── │   runtime thread)    │
//! └──────────────────┘     oneshot::channel   └──────────────────────┘
//! ```

use std::sync::Mutex;
use tokio::sync::{mpsc, oneshot};
use once_cell::sync::Lazy;

/// Types of web research requests
pub enum WebResearchRequest {
    CrawlUrl {
        url: String,
        convert_to_markdown: bool,
        response_tx: oneshot::Sender<Result<serde_json::Value, String>>,
    },
    ResearchDocs {
        urls: Vec<String>,
        response_tx: oneshot::Sender<Result<serde_json::Value, String>>,
    },
    ExtractContent {
        url: String,
        strategy_type: String,
        schema: serde_json::Value,
        response_tx: oneshot::Sender<Result<serde_json::Value, String>>,
    },
}

/// Worker pool handle for sending web research requests
pub struct WebResearchWorker {
    request_tx: mpsc::Sender<WebResearchRequest>,
}

impl WebResearchWorker {
    /// Create a new web research worker with dedicated runtime thread
    pub fn new() -> Self {
        let (request_tx, mut request_rx) = mpsc::channel::<WebResearchRequest>(64);

        // Spawn dedicated runtime thread for web research operations
        std::thread::Builder::new()
            .name("web-research-worker".to_string())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .thread_name("web-research")
                    .enable_all()
                    .build()
                    .expect("Failed to create web research runtime");

                rt.block_on(async move {
                    while let Some(request) = request_rx.recv().await {
                        match request {
                            WebResearchRequest::CrawlUrl { url, convert_to_markdown, response_tx } => {
                                let result = Self::do_crawl_url(url, convert_to_markdown).await;
                                let _ = response_tx.send(result);
                            }
                            WebResearchRequest::ResearchDocs { urls, response_tx } => {
                                let result = Self::do_research_docs(urls).await;
                                let _ = response_tx.send(result);
                            }
                            WebResearchRequest::ExtractContent { url, strategy_type, schema, response_tx } => {
                                let result = Self::do_extract_content(url, strategy_type, schema).await;
                                let _ = response_tx.send(result);
                            }
                        }
                    }
                });
            })
            .expect("Failed to spawn web research worker thread");

        Self { request_tx }
    }

    /// Crawl a single URL (async implementation)
    async fn do_crawl_url(url: String, convert_to_markdown: bool) -> Result<serde_json::Value, String> {
        use crawl4ai::{HttpCrawler, HttpCrawlConfig};

        let config = HttpCrawlConfig {
            convert_to_markdown,
            filter_content: true,
            ..Default::default()
        };

        let crawler = HttpCrawler::with_config(config)
            .map_err(|e| format!("Failed to create crawler: {}", e))?;

        let result = crawler.crawl(&url).await
            .map_err(|e| format!("Crawl failed: {}", e))?;

        Ok(serde_json::json!({
            "url": result.url,
            "status_code": result.status_code,
            "title": result.title,
            "markdown": result.markdown,
            "cleaned_content": result.cleaned_content,
            "duration_ms": result.duration_ms
        }))
    }

    /// Research multiple URLs (async implementation)
    async fn do_research_docs(urls: Vec<String>) -> Result<serde_json::Value, String> {
        use crawl4ai::{HttpCrawler, HttpCrawlConfig};

        let config = HttpCrawlConfig {
            convert_to_markdown: true,
            filter_content: true,
            timeout_secs: 30,
            ..Default::default()
        };

        let crawler = HttpCrawler::with_config(config)
            .map_err(|e| format!("Failed to create crawler: {}", e))?;

        let url_refs: Vec<&str> = urls.iter().map(|s| s.as_str()).collect();
        let results = crawler.crawl_many(&url_refs).await;

        let mut documents = Vec::new();
        let mut errors = Vec::new();

        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(r) => documents.push(serde_json::json!({
                    "url": r.url,
                    "title": r.title,
                    "markdown": r.markdown,
                    "status_code": r.status_code
                })),
                Err(e) => errors.push(serde_json::json!({
                    "url": urls.get(i).cloned().unwrap_or_default(),
                    "error": e.to_string()
                })),
            }
        }

        Ok(serde_json::json!({
            "documents": documents,
            "errors": errors,
            "total_urls": urls.len(),
            "success_count": documents.len(),
            "error_count": errors.len()
        }))
    }

    /// Extract structured content (async implementation)
    async fn do_extract_content(
        url: String,
        strategy_type: String,
        schema: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        use crawl4ai::{HttpCrawler, HttpCrawlConfig, JsonCssExtractionStrategy, JsonXPathExtractionStrategy};

        let config = HttpCrawlConfig::default();
        let crawler = HttpCrawler::with_config(config)
            .map_err(|e| format!("Failed to create crawler: {}", e))?;

        let result = crawler.crawl(&url).await
            .map_err(|e| format!("Crawl failed: {}", e))?;

        let extracted = match strategy_type.as_str() {
            "css" => {
                let strategy = JsonCssExtractionStrategy::new(schema);
                strategy.extract(&result.html)
            },
            "xpath" => {
                let strategy = JsonXPathExtractionStrategy::new(schema);
                strategy.extract(&result.html)
            },
            _ => return Err(format!("Unknown strategy type: {}. Use 'css' or 'xpath'", strategy_type))
        };

        Ok(serde_json::json!({
            "url": result.url,
            "title": result.title,
            "extracted": extracted,
            "count": extracted.len()
        }))
    }

    /// Sync wrapper: Crawl a single URL
    pub fn crawl_url_sync(&self, url: String, convert_to_markdown: bool) -> Result<serde_json::Value, String> {
        let (response_tx, response_rx) = oneshot::channel();

        self.request_tx.blocking_send(WebResearchRequest::CrawlUrl {
            url,
            convert_to_markdown,
            response_tx,
        }).map_err(|e| format!("Failed to send request: {}", e))?;

        response_rx.blocking_recv()
            .map_err(|_| "Worker dropped response channel".to_string())?
    }

    /// Sync wrapper: Research multiple documentation URLs
    pub fn research_docs_sync(&self, urls: Vec<String>) -> Result<serde_json::Value, String> {
        let (response_tx, response_rx) = oneshot::channel();

        self.request_tx.blocking_send(WebResearchRequest::ResearchDocs {
            urls,
            response_tx,
        }).map_err(|e| format!("Failed to send request: {}", e))?;

        response_rx.blocking_recv()
            .map_err(|_| "Worker dropped response channel".to_string())?
    }

    /// Sync wrapper: Extract structured content from URL
    pub fn extract_content_sync(
        &self,
        url: String,
        strategy_type: String,
        schema: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let (response_tx, response_rx) = oneshot::channel();

        self.request_tx.blocking_send(WebResearchRequest::ExtractContent {
            url,
            strategy_type,
            schema,
            response_tx,
        }).map_err(|e| format!("Failed to send request: {}", e))?;

        response_rx.blocking_recv()
            .map_err(|_| "Worker dropped response channel".to_string())?
    }
}

impl Default for WebResearchWorker {
    fn default() -> Self {
        Self::new()
    }
}

/// Global web research worker instance
static WEB_RESEARCH_WORKER: Lazy<Mutex<Option<WebResearchWorker>>> = Lazy::new(|| Mutex::new(None));

/// Initialize the global web research worker
pub fn init_web_research_worker() {
    let mut worker = WEB_RESEARCH_WORKER.lock().unwrap();
    if worker.is_none() {
        *worker = Some(WebResearchWorker::new());
    }
}

/// Check if the worker is initialized
pub fn is_worker_initialized() -> bool {
    WEB_RESEARCH_WORKER.lock().map(|w| w.is_some()).unwrap_or(false)
}

/// Crawl a URL synchronously using the global worker
pub fn crawl_url_sync(url: String, convert_to_markdown: bool) -> Result<serde_json::Value, String> {
    let worker = WEB_RESEARCH_WORKER.lock()
        .map_err(|_| "Failed to acquire web research worker lock")?;

    worker.as_ref()
        .ok_or_else(|| "Web research worker not initialized. Call init_web_research_worker first.".to_string())?
        .crawl_url_sync(url, convert_to_markdown)
}

/// Research docs synchronously using the global worker
pub fn research_docs_sync(urls: Vec<String>) -> Result<serde_json::Value, String> {
    let worker = WEB_RESEARCH_WORKER.lock()
        .map_err(|_| "Failed to acquire web research worker lock")?;

    worker.as_ref()
        .ok_or_else(|| "Web research worker not initialized".to_string())?
        .research_docs_sync(urls)
}

/// Extract content synchronously using the global worker
pub fn extract_content_sync(
    url: String,
    strategy_type: String,
    schema: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let worker = WEB_RESEARCH_WORKER.lock()
        .map_err(|_| "Failed to acquire web research worker lock")?;

    worker.as_ref()
        .ok_or_else(|| "Web research worker not initialized".to_string())?
        .extract_content_sync(url, strategy_type, schema)
}
