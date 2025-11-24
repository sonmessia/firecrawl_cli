use std::path::PathBuf;

use crate::api::models::{scrape_model::ScrapeResponse, crawl_model::CrawlResponse};
use super::{ContentSaver, StorageError, StorageResult};

/// Markdown content saver
pub struct MarkdownSaver;

#[async_trait::async_trait]
impl ContentSaver for MarkdownSaver {
    async fn save_scrape_result(
        &self,
        result: &ScrapeResponse,
        url: &str,
        output_dir: &PathBuf,
    ) -> StorageResult<PathBuf> {
        self.ensure_directory(output_dir).await?;

        let filename = self.generate_filename(url, None);
        let file_path = output_dir.join(filename);

        let title = result.data
            .as_ref()
            .and_then(|d| d.metadata.title.as_ref())
            .unwrap_or(&"Untitled".to_string());

        let markdown = result.data
            .as_ref()
            .and_then(|d| d.markdown.as_ref())
            .unwrap_or(&"No content available".to_string());

        let content = format!(
            "# {}\n\n**Source:** {}\n\n**Timestamp:** {}\n\n---\n\n{}",
            title,
            url,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            markdown
        );

        self.write_file(&file_path, &content).await?;
        Ok(file_path)
    }

    async fn save_crawl_results(
        &self,
        results: &[CrawlResponse],
        url: &str,
        output_dir: &PathBuf,
    ) -> StorageResult<Vec<PathBuf>> {
        self.ensure_directory(output_dir).await?;
        let mut saved_files = Vec::new();

        for (index, result) in results.iter().enumerate() {
            let filename = self.generate_filename(&result.url, Some(index));
            let file_path = output_dir.join(filename);

            let content = format!(
                "# {}\n\n**Source:** {}\n\n**Crawl from:** {}\n\n**Timestamp:** {}\n\n---\n\n{}",
                result.metadata.title.as_ref().unwrap_or(&"Untitled".to_string()),
                result.url,
                url,
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
                result.markdown.as_ref().unwrap_or(&"No content available".to_string())
            );

            self.write_file(&file_path, &content).await?;
            saved_files.push(file_path);
        }

        Ok(saved_files)
    }

    fn file_extension(&self) -> &'static str {
        "md"
    }
}

/// HTML content saver
pub struct HtmlSaver;

#[async_trait::async_trait]
impl ContentSaver for HtmlSaver {
    async fn save_scrape_result(
        &self,
        result: &ScrapeResponse,
        url: &str,
        output_dir: &PathBuf,
    ) -> StorageResult<PathBuf> {
        self.ensure_directory(output_dir).await?;

        let filename = self.generate_filename(url, None);
        let file_path = output_dir.join(filename);

        let scrape_data = result.data.as_ref()
            .ok_or_else(|| StorageError::UnsupportedContentType("Scrape data not available".to_string()))?;

        let html_content = scrape_data.html.as_ref()
            .or(scrape_data.raw_html.as_ref())
            .ok_or_else(|| StorageError::UnsupportedContentType("HTML content not available".to_string()))?;

        self.write_file(&file_path, html_content).await?;
        Ok(file_path)
    }

    async fn save_crawl_results(
        &self,
        results: &[CrawlResponse],
        url: &str,
        output_dir: &PathBuf,
    ) -> StorageResult<Vec<PathBuf>> {
        self.ensure_directory(output_dir).await?;
        let mut saved_files = Vec::new();

        for (index, result) in results.iter().enumerate() {
            let filename = self.generate_filename(&result.url, Some(index));
            let file_path = output_dir.join(filename);

            let html_content = result.html.as_ref()
                .ok_or_else(|| StorageError::UnsupportedContentType(
                    format!("HTML content not available for {}", result.url)
                ))?;

            self.write_file(&file_path, html_content).await?;
            saved_files.push(file_path);
        }

        Ok(saved_files)
    }

    fn file_extension(&self) -> &'static str {
        "html"
    }
}

/// JSON content saver
pub struct JsonSaver;

#[async_trait::async_trait]
impl ContentSaver for JsonSaver {
    async fn save_scrape_result(
        &self,
        result: &ScrapeResponse,
        url: &str,
        output_dir: &PathBuf,
    ) -> StorageResult<PathBuf> {
        self.ensure_directory(output_dir).await?;

        let filename = self.generate_filename(url, None);
        let file_path = output_dir.join(filename);

        let json_content = serde_json::to_string_pretty(result)?;
        self.write_file(&file_path, &json_content).await?;
        Ok(file_path)
    }

    async fn save_crawl_results(
        &self,
        results: &[CrawlResponse],
        _base_url: &str,
        output_dir: &PathBuf,
    ) -> StorageResult<Vec<PathBuf>> {
        self.ensure_directory(output_dir).await?;

        // Save crawl results as a single JSON file containing all results
        let filename = format!("crawl_results_{}.json", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
        let file_path = output_dir.join(filename);

        let json_content = serde_json::to_string_pretty(results)?;
        self.write_file(&file_path, &json_content).await?;
        Ok(vec![file_path])
    }

    fn file_extension(&self) -> &'static str {
        "json"
    }
}

/// Raw text content saver
pub struct RawSaver;

#[async_trait::async_trait]
impl ContentSaver for RawSaver {
    async fn save_scrape_result(
        &self,
        result: &ScrapeResponse,
        url: &str,
        output_dir: &PathBuf,
    ) -> StorageResult<PathBuf> {
        self.ensure_directory(output_dir).await?;

        let filename = self.generate_filename(url, None);
        let file_path = output_dir.join(filename);

        let scrape_data = result.data.as_ref()
            .ok_or_else(|| StorageError::UnsupportedContentType("Scrape data not available".to_string()))?;

        let content = scrape_data.markdown.as_ref()
            .or(scrape_data.html.as_ref())
            .or(scrape_data.raw_html.as_ref())
            .map(|s| s.as_str())
            .unwrap_or("No content available");

        self.write_file(&file_path, content).await?;
        Ok(file_path)
    }

    async fn save_crawl_results(
        &self,
        results: &[CrawlResponse],
        _base_url: &str,
        output_dir: &PathBuf,
    ) -> StorageResult<Vec<PathBuf>> {
        self.ensure_directory(output_dir).await?;
        let mut saved_files = Vec::new();

        for (index, result) in results.iter().enumerate() {
            let filename = self.generate_filename(&result.url, Some(index));
            let file_path = output_dir.join(filename);

            let content = result.markdown.as_ref()
                .or(result.html.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("No content available");

            self.write_file(&file_path, content).await?;
            saved_files.push(file_path);
        }

        Ok(saved_files)
    }

    fn file_extension(&self) -> &'static str {
        "txt"
    }
}