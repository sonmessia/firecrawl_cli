use async_trait::async_trait;
use std::path::PathBuf;

use crate::api::models::crawl_model::{CrawlOptions, CrawlRequest, CrawlResponse};
use crate::api::services::client::FirecrawlClient;
use crate::cli::OutputFormat;
use crate::commands::{Command, CommandObserver, CommandResult, NoOpObserver};
use crate::errors::{FirecrawlError, FirecrawlResult};
use crate::storage::ContentRepository;
use crate::services::CrawlMonitorService;

/// Command for crawling a URL
#[derive(Debug, Clone)]
pub struct CrawlCommand {
    pub url: String,
    pub options: Option<CrawlOptions>,
    pub output_format: OutputFormat,
}

impl CrawlCommand {
    /// Create a new crawl command
    pub fn new(url: String, options: Option<CrawlOptions>, output_format: OutputFormat) -> Self {
        Self {
            url,
            options,
            output_format,
        }
    }

    /// Create a builder for crawl command
    pub fn builder() -> CrawlCommandBuilder {
        CrawlCommandBuilder::new()
    }

    /// Execute the crawl operation with the provided client
    async fn execute_crawl(&self, client: &FirecrawlClient) -> FirecrawlResult<Vec<CrawlResponse>> {
        let request = if let Some(options) = &self.options {
            CrawlRequest::builder()
                .url(self.url.clone())
                .limit(options.limit)
                .only_main_content(options.only_main_content)
                .build()
                .map_err(|e| FirecrawlError::ValidationError(e))?
        } else {
            CrawlRequest::builder()
                .url(self.url.clone())
                .build()
                .map_err(|e| FirecrawlError::ValidationError(e))?
        };

        let crawl_result = client
            .crawl_url(request)
            .await
            .map_err(FirecrawlError::ApiError)?;

        // Wait for crawl to complete and get results
        let monitor_service = client as &dyn CrawlMonitorService;
        monitor_service
            .monitor_crawl_job(&crawl_result.job_id, Box::new(|progress| {
                // Progress callback could be used by observer
                // For now, we'll just ignore progress updates
            }))
            .await?
    }
}

#[async_trait]
impl Command for CrawlCommand {
    type Result = CommandResult;

    async fn execute(
        &self,
        repository: &dyn ContentRepository,
        output_dir: &PathBuf,
    ) -> FirecrawlResult<Self::Result> {
        // Create client
        let api_key = std::env::var("FIRECRAWL_API_KEY").ok();
        let client = FirecrawlClient::new("https://api.firecrawl.dev", api_key.as_deref())
            .map_err(|e| FirecrawlError::ConfigurationError(e.to_string()))?;

        // Notify start
        let observer = NoOpObserver; // Could be injected
        observer.on_command_started(self);

        // Execute crawl
        let crawl_results = self.execute_crawl(&client).await.map_err(|e| {
            observer.on_command_failed(self, &e);
            e
        })?;

        // Save results
        let file_paths = repository
            .save_crawl_results(&crawl_results, &self.url, self.output_format, output_dir)
            .await
            .map_err(FirecrawlError::StorageError)?;

        let result = CommandResult::Crawl {
            url: self.url.clone(),
            file_paths,
        };

        observer.on_command_completed(self, &result);
        Ok(result)
    }

    fn description(&self) -> String {
        format!("Crawl {} as {}", self.url, self.output_format)
    }

    fn url(&self) -> &str {
        &self.url
    }

    fn output_format(&self) -> OutputFormat {
        self.output_format.clone()
    }
}

/// Builder for CrawlCommand
pub struct CrawlCommandBuilder {
    url: Option<String>,
    options: Option<CrawlOptions>,
    output_format: OutputFormat,
}

impl CrawlCommandBuilder {
    pub fn new() -> Self {
        Self {
            url: None,
            options: None,
            output_format: OutputFormat::Markdown,
        }
    }

    pub fn url(mut self, url: String) -> Self {
        self.url = Some(url);
        self
    }

    pub fn options(mut self, options: CrawlOptions) -> Self {
        self.options = Some(options);
        self
    }

    pub fn output_format(mut self, format: OutputFormat) -> Self {
        self.output_format = format;
        self
    }

    pub fn build(self) -> FirecrawlResult<CrawlCommand> {
        let url = self
            .url
            .ok_or_else(|| FirecrawlError::ValidationError("URL is required".to_string()))?;

        Ok(CrawlCommand {
            url,
            options: self.options,
            output_format: self.output_format,
        })
    }
}
