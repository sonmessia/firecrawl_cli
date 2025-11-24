use async_trait::async_trait;
use std::path::PathBuf;

use crate::api::models::scrape_model::{ScrapeRequest, ScrapeResponse, ScrapeOptions};
use crate::api::services::client::FirecrawlClient;
use crate::cli::OutputFormat;
use crate::commands::{Command, CommandResult, CommandObserver, NoOpObserver};
use crate::storage::ContentRepository;
use crate::errors::{FirecrawlError, FirecrawlResult};

/// Command for scraping a single URL
#[derive(Debug, Clone)]
pub struct ScrapeCommand {
    pub url: String,
    pub options: Option<ScrapeOptions>,
    pub output_format: OutputFormat,
}

impl ScrapeCommand {
    /// Create a new scrape command
    pub fn new(url: String, options: Option<ScrapeOptions>, output_format: OutputFormat) -> Self {
        Self {
            url,
            options,
            output_format,
        }
    }

    /// Create a builder for scrape command
    pub fn builder() -> ScrapeCommandBuilder {
        ScrapeCommandBuilder::new()
    }

    /// Execute the scrape operation with the provided client
    async fn execute_scrape(&self, client: &FirecrawlClient) -> FirecrawlResult<ScrapeResponse> {
        let request = if let Some(options) = &self.options {
            ScrapeRequest::builder()
                .url(self.url.clone())
                .formats(Some(vec![self.output_format.clone()]))
                .only_main_content(options.only_main_content)
                .include_tags(options.include_tags.clone())
                .exclude_tags(options.exclude_tags.clone())
                .build()
                .map_err(|e| FirecrawlError::ValidationError(e.to_string()))?
        } else {
            ScrapeRequest::builder()
                .url(self.url.clone())
                .formats(Some(vec![self.output_format.clone()]))
                .build()
                .map_err(|e| FirecrawlError::ValidationError(e.to_string()))?
        };

        client.scrape_url(request).await
            .map_err(FirecrawlError::ApiError)
    }
}

#[async_trait]
impl Command for ScrapeCommand {
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

        // Execute scrape
        let scrape_result = self.execute_scrape(&client).await
            .map_err(|e| {
                observer.on_command_failed(self, &e);
                e
            })?;

        // Save result
        let file_path = repository
            .save_scrape_result(&scrape_result, &self.url, self.output_format, output_dir)
            .await
            .map_err(FirecrawlError::StorageError)?;

        let result = CommandResult::Scrape {
            url: self.url.clone(),
            file_path,
        };

        observer.on_command_completed(self, &result);
        Ok(result)
    }

    fn description(&self) -> String {
        format!("Scrape {} as {}", self.url, self.output_format)
    }

    fn url(&self) -> &str {
        &self.url
    }

    fn output_format(&self) -> OutputFormat {
        self.output_format.clone()
    }
}

/// Builder for ScrapeCommand
pub struct ScrapeCommandBuilder {
    url: Option<String>,
    options: Option<ScrapeOptions>,
    output_format: OutputFormat,
}

impl ScrapeCommandBuilder {
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

    pub fn options(mut self, options: ScrapeOptions) -> Self {
        self.options = Some(options);
        self
    }

    pub fn output_format(mut self, format: OutputFormat) -> Self {
        self.output_format = format;
        self
    }

    pub fn build(self) -> FirecrawlResult<ScrapeCommand> {
        let url = self.url.ok_or_else(|| {
            FirecrawlError::ValidationError("URL is required".to_string())
        })?;

        Ok(ScrapeCommand {
            url,
            options: self.options,
            output_format: self.output_format,
        })
    }
}