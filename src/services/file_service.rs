use async_trait::async_trait;
use std::path::{Path, PathBuf};

use crate::api::models::{crawl_model::CrawlResponse, scrape_model::ScrapeResponse};
use crate::cli::OutputFormat;
use crate::errors::{FirecrawlError, FirecrawlResult};
use crate::storage::ContentRepository;

/// Service for file operations that wraps the repository pattern
pub struct FileService {
    repository: Box<dyn ContentRepository + Send + Sync>,
}

impl FileService {
    /// Create a new FileService with the given repository
    pub fn new<R: ContentRepository + Send + Sync + 'static>(repository: R) -> Self {
        Self {
            repository: Box::new(repository),
        }
    }

    /// Create from a boxed repository
    pub fn from_boxed_repository(repository: Box<dyn ContentRepository + Send + Sync>) -> Self {
        Self { repository }
    }

    /// Get a reference to the underlying repository
    pub fn repository(&self) -> &dyn ContentRepository {
        self.repository.as_ref()
    }

    /// Save scrape result with automatic output directory management
    pub async fn save_scrape_result(
        &self,
        result: &ScrapeResponse,
        url: &str,
        format: OutputFormat,
        output_dir: &PathBuf,
    ) -> FirecrawlResult<PathBuf> {
        // Ensure output directory exists
        self.repository
            .ensure_directory(output_dir)
            .await
            .map_err(FirecrawlError::StorageError)?;

        // Save the result
        self.repository
            .save_scrape_result(result, url, format, output_dir)
            .await
            .map_err(FirecrawlError::StorageError)
    }

    /// Save crawl results with automatic output directory management
    pub async fn save_crawl_results(
        &self,
        results: &[CrawlResponse],
        url: &str,
        format: OutputFormat,
        output_dir: &PathBuf,
    ) -> FirecrawlResult<Vec<PathBuf>> {
        // Ensure output directory exists
        self.repository
            .ensure_directory(output_dir)
            .await
            .map_err(FirecrawlError::StorageError)?;

        // Save the results
        self.repository
            .save_crawl_results(results, url, format, output_dir)
            .await
            .map_err(FirecrawlError::StorageError)
    }

    /// Generate a filename for a URL
    pub fn generate_filename(&self, url: &str, format: OutputFormat) -> String {
        self.repository.generate_filename(url, format)
    }

    /// Check if a file exists
    pub async fn file_exists(&self, path: &PathBuf) -> bool {
        self.repository.file_exists(path).await
    }

    /// Ensure a directory exists
    pub async fn ensure_directory(&self, path: &PathBuf) -> FirecrawlResult<()> {
        self.repository
            .ensure_directory(path)
            .await
            .map_err(FirecrawlError::StorageError)
    }

    /// Create subdirectory within output directory
    pub async fn create_subdirectory(
        &self,
        base_dir: &Path,
        subdirectory: &str,
    ) -> FirecrawlResult<PathBuf> {
        let full_path = base_dir.join(subdirectory);
        self.ensure_directory(&full_path).await?;
        Ok(full_path)
    }

    /// Create date-based subdirectory
    pub async fn create_date_subdirectory(&self, base_dir: &Path) -> FirecrawlResult<PathBuf> {
        let date_str = chrono::Utc::now().format("%Y-%m-%d").to_string();
        self.create_subdirectory(base_dir, &date_str).await
    }

    /// Create subdirectory with URL-based naming
    pub async fn create_url_subdirectory(
        &self,
        base_dir: &PathBuf,
        url: &str,
    ) -> FirecrawlResult<PathBuf> {
        use slug::slugify;
        let url_slug = slugify(url);
        self.create_subdirectory(base_dir, &url_slug).await
    }

    /// Generate unique filename to avoid conflicts
    pub async fn generate_unique_filename(
        &self,
        url: &str,
        format: OutputFormat,
        output_dir: &Path,
    ) -> FirecrawlResult<String> {
        let mut filename = self.generate_filename(url, format.clone());
        let mut counter = 1;

        // Check if file exists and generate unique name if needed
        while self.file_exists(&output_dir.join(&filename)).await {
            let extension = match format {
                OutputFormat::Markdown => "md",
                OutputFormat::Html => "html",
                OutputFormat::Json => "json",
                OutputFormat::Raw => "txt",
                OutputFormat::RawHtml => "html",
                OutputFormat::Links => "json",
                OutputFormat::Images => "json",
            };

            let base_name = filename
                .strip_suffix(&format!(".{}", extension))
                .unwrap_or(&filename);

            filename = format!("{}-{}.{}", base_name, counter, extension);
            counter += 1;
        }

        Ok(filename)
    }

    /// Save scrape result with automatic conflict resolution
    pub async fn save_scrape_result_unique(
        &self,
        result: &ScrapeResponse,
        url: &str,
        format: OutputFormat,
        output_dir: &PathBuf,
    ) -> FirecrawlResult<PathBuf> {
        let filename = self
            .generate_unique_filename(url, format.clone(), output_dir)
            .await?;
        let file_path = output_dir.join(filename);

        // Use the repository to save with custom filename logic
        self.save_scrape_result(result, url, format, output_dir)
            .await
    }

    /// Create a backup of an existing file
    pub async fn backup_file(&self, file_path: &PathBuf) -> FirecrawlResult<PathBuf> {
        if !self.file_exists(file_path).await {
            return Err(FirecrawlError::StorageError(
                crate::storage::StorageError::FileNotFound(file_path.to_string_lossy().to_string()),
            ));
        }

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let original_name = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("backup");
        let extension = file_path.extension().and_then(|s| s.to_str()).unwrap_or("");

        let backup_name = format!("{}_backup{}.{}", original_name, timestamp, extension);
        let backup_path = file_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join(backup_name);

        tokio::fs::copy(file_path, &backup_path)
            .await
            .map_err(|e| {
                FirecrawlError::StorageError(crate::storage::StorageError::FileSystem(
                    e.to_string(),
                ))
            })?;

        Ok(backup_path)
    }

    /// Get file size in bytes
    pub async fn get_file_size(&self, file_path: &PathBuf) -> FirecrawlResult<u64> {
        let metadata = tokio::fs::metadata(file_path).await.map_err(|e| {
            FirecrawlError::StorageError(crate::storage::StorageError::FileSystem(e.to_string()))
        })?;
        Ok(metadata.len())
    }

    /// Clean up old files in a directory (older than specified duration)
    pub async fn cleanup_old_files(
        &self,
        directory: &PathBuf,
        older_than: chrono::Duration,
    ) -> FirecrawlResult<Vec<PathBuf>> {
        if !directory.exists() {
            return Ok(Vec::new());
        }

        let mut entries = tokio::fs::read_dir(directory).await.map_err(|e| {
            FirecrawlError::StorageError(crate::storage::StorageError::FileSystem(e.to_string()))
        })?;

        let mut removed_files = Vec::new();
        let cutoff_time = chrono::Utc::now() - older_than;

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_file() {
                if let Ok(metadata) = entry.metadata().await {
                    if let Ok(modified) = metadata.modified() {
                        let modified_time = chrono::DateTime::<chrono::Utc>::from(modified);
                        if modified_time < cutoff_time {
                            if tokio::fs::remove_file(&path).await.is_ok() {
                                removed_files.push(path);
                            }
                        }
                    }
                }
            }
        }

        Ok(removed_files)
    }
}

/// Factory for creating file services
pub struct FileServiceFactory;

impl FileServiceFactory {
    /// Create a file service with filesystem repository
    pub fn create_filesystem_service(base_dir: PathBuf) -> FileService {
        let repository = crate::storage::FileSystemRepository::new(base_dir);
        FileService::new(repository)
    }

    /// Create a file service with default output directory
    pub fn create_default_service() -> FileService {
        let base_dir = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("output");
        Self::create_filesystem_service(base_dir)
    }

    /// Create a file service from configuration
    pub fn from_config(config: &crate::config::AppConfig) -> FileService {
        Self::create_filesystem_service(config.output.default_directory.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_file_service_basic_operations() {
        let temp_dir = tempdir().unwrap();
        let service = FileServiceFactory::create_filesystem_service(temp_dir.path().to_path_buf());

        // Test directory creation
        let subdir = temp_dir.path().join("test_subdir");
        assert!(!subdir.exists());

        service.ensure_directory(&subdir).await.unwrap();
        assert!(subdir.exists());

        // Test filename generation
        let filename =
            service.generate_filename("https://example.com/test", OutputFormat::Markdown);
        assert_eq!(filename, "https-example-com-test.md");
    }

    #[tokio::test]
    async fn test_unique_filename_generation() {
        let temp_dir = tempdir().unwrap();
        let service = FileServiceFactory::create_filesystem_service(temp_dir.path().to_path_buf());

        // First call should generate regular filename
        let filename1 = service
            .generate_unique_filename(
                "https://example.com",
                OutputFormat::Markdown,
                temp_dir.path(),
            )
            .await
            .unwrap();
        assert_eq!(filename1, "https-example-com.md");

        // Create a file with that name
        let file_path = temp_dir.path().join(&filename1);
        tokio::fs::write(&file_path, "test content").await.unwrap();

        // Second call should generate a unique filename
        let filename2 = service
            .generate_unique_filename(
                "https://example.com",
                OutputFormat::Markdown,
                temp_dir.path(),
            )
            .await
            .unwrap();
        assert!(filename2 != filename1);
        assert!(filename2.starts_with("https-example-com-1"));
    }

    #[tokio::test]
    async fn test_date_subdirectory() {
        let temp_dir = tempdir().unwrap();
        let service = FileServiceFactory::create_filesystem_service(temp_dir.path().to_path_buf());

        let date_dir = service
            .create_date_subdirectory(temp_dir.path())
            .await
            .unwrap();
        let date_str = chrono::Utc::now().format("%Y-%m-%d").to_string();
        assert_eq!(date_dir.file_name().unwrap().to_str().unwrap(), date_str);
        assert!(date_dir.exists());
    }
}

