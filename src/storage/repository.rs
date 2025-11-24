use async_trait::async_trait;
use std::path::PathBuf;

use crate::api::models::{scrape_model::ScrapeResponse, crawl_model::CrawlResponse};
use crate::cli::OutputFormat;
use super::{StorageError, StorageResult};

/// Repository trait for abstracting file operations
#[async_trait]
pub trait ContentRepository: Send + Sync {
    /// Save scrape result in the specified format
    async fn save_scrape_result(
        &self,
        result: &ScrapeResponse,
        url: &str,
        format: OutputFormat,
        output_dir: &PathBuf,
    ) -> StorageResult<PathBuf>;

    /// Save crawl results in the specified format
    async fn save_crawl_results(
        &self,
        results: &[CrawlResponse],
        url: &str,
        format: OutputFormat,
        output_dir: &PathBuf,
    ) -> StorageResult<Vec<PathBuf>>;

    /// Create directory if it doesn't exist
    async fn ensure_directory(&self, path: &PathBuf) -> StorageResult<()>;

    /// Check if file exists
    async fn file_exists(&self, path: &PathBuf) -> bool;

    /// Generate filename from URL and format
    fn generate_filename(&self, url: &str, format: OutputFormat) -> String;
}

/// File system implementation of ContentRepository
pub struct FileSystemRepository {
    base_dir: PathBuf,
}

impl FileSystemRepository {
    /// Create a new FileSystemRepository with the given base directory
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Get the base directory
    pub fn base_dir(&self) -> &PathBuf {
        &self.base_dir
    }
}

#[async_trait]
impl ContentRepository for FileSystemRepository {
    async fn save_scrape_result(
        &self,
        result: &ScrapeResponse,
        url: &str,
        format: OutputFormat,
        output_dir: &PathBuf,
    ) -> StorageResult<PathBuf> {
        use super::content_saver::ContentSaver;
        use super::content_saver::savers::{MarkdownSaver, HtmlSaver, JsonSaver, RawSaver};

        let saver: Box<dyn ContentSaver> = match format {
            OutputFormat::Markdown => Box::new(MarkdownSaver),
            OutputFormat::Html => Box::new(HtmlSaver),
            OutputFormat::Json => Box::new(JsonSaver),
            OutputFormat::Raw => Box::new(RawSaver),
            OutputFormat::RawHtml => Box::new(HtmlSaver), // Use HtmlSaver for RawHtml
        };

        saver.save_scrape_result(result, url, output_dir).await
    }

    async fn save_crawl_results(
        &self,
        results: &[CrawlResponse],
        url: &str,
        format: OutputFormat,
        output_dir: &PathBuf,
    ) -> StorageResult<Vec<PathBuf>> {
        use super::content_saver::ContentSaver;
        use super::content_saver::savers::{MarkdownSaver, HtmlSaver, JsonSaver, RawSaver};

        let saver: Box<dyn ContentSaver> = match format {
            OutputFormat::Markdown => Box::new(MarkdownSaver),
            OutputFormat::Html => Box::new(HtmlSaver),
            OutputFormat::Json => Box::new(JsonSaver),
            OutputFormat::Raw => Box::new(RawSaver),
            OutputFormat::RawHtml => Box::new(HtmlSaver), // Use HtmlSaver for RawHtml
        };

        saver.save_crawl_results(results, url, output_dir).await
    }

    async fn ensure_directory(&self, path: &PathBuf) -> StorageResult<()> {
        if !path.exists() {
            tokio::fs::create_dir_all(path).await?;
        }
        Ok(())
    }

    async fn file_exists(&self, path: &PathBuf) -> bool {
        tokio::fs::metadata(path).await.is_ok()
    }

    fn generate_filename(&self, url: &str, format: OutputFormat) -> String {
        use slug::slugify;

        let slug = slugify(url);
        let extension = match format {
            OutputFormat::Markdown => "md",
            OutputFormat::Html => "html",
            OutputFormat::Json => "json",
            OutputFormat::Raw => "txt",
            OutputFormat::RawHtml => "html",
        };

        format!("{}.{}", slug, extension)
    }
}