use async_trait::async_trait;
use std::path::PathBuf;

use crate::api::models::{scrape_model::ScrapeResponse, crawl_model::CrawlResponse};
use super::{StorageError, StorageResult};

pub mod savers;

/// Strategy pattern for different content saving approaches
#[async_trait]
pub trait ContentSaver: Send + Sync {
    /// Save a single scrape result
    async fn save_scrape_result(
        &self,
        result: &ScrapeResponse,
        url: &str,
        output_dir: &PathBuf,
    ) -> StorageResult<PathBuf>;

    /// Save multiple crawl results
    async fn save_crawl_results(
        &self,
        results: &[CrawlResponse],
        url: &str,
        output_dir: &PathBuf,
    ) -> StorageResult<Vec<PathBuf>>;

    /// Get the file extension for this format
    fn file_extension(&self) -> &'static str;

    /// Generate filename from URL
    fn generate_filename(&self, url: &str, index: Option<usize>) -> String {
        use slug::slugify;

        let slug = slugify(url);
        match index {
            Some(i) => format!("{}-{}.{}", slug, i, self.file_extension()),
            None => format!("{}.{}", slug, self.file_extension()),
        }
    }

    /// Ensure output directory exists
    async fn ensure_directory(&self, output_dir: &PathBuf) -> StorageResult<()> {
        if !output_dir.exists() {
            tokio::fs::create_dir_all(output_dir).await?;
        }
        Ok(())
    }

    /// Write content to file
    async fn write_file(&self, path: &PathBuf, content: &str) -> StorageResult<()> {
        tokio::fs::write(path, content).await?;
        Ok(())
    }
}