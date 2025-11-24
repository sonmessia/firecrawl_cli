pub mod task_service;
pub mod api_service;
pub mod file_service;
pub mod progress_service;
pub mod cache_service;

pub use task_service::*;
pub use api_service::*;
pub use file_service::*;
pub use progress_service::*;
pub use cache_service::*;

/// Crawl progress information for monitoring crawl jobs
#[derive(Debug, Clone)]
pub struct CrawlProgress {
    pub completed: u32,
    pub total: u32,
    pub current_url: Option<String>,
    pub status: String,
}