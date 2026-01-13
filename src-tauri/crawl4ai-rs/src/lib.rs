pub mod models;
pub mod markdown;
pub mod content_filter;
pub mod extraction_strategy;

// Browser-based crawler (requires chromiumoxide)
#[cfg(feature = "browser")]
pub mod crawler;

// HTTP-only crawler (no browser dependency)
pub mod http_crawler;

// Re-exports
pub use models::*;
pub use markdown::*;
pub use content_filter::*;
pub use http_crawler::HttpCrawler;
