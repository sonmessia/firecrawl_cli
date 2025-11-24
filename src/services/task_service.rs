use std::sync::Arc;

use crate::cli::{OutputFormat, CrawlOptions, ScrapeOptions};
use crate::commands::{Command, CommandResult, ScrapeCommand, CrawlCommand, TaskQueueFactory};
use crate::storage::ContentRepository;
use crate::services::{ApiService, ProgressService, CacheService};
use crate::config::AppConfig;
use crate::errors::{FirecrawlError, FirecrawlResult};

/// Service for managing and executing tasks
pub struct TaskService {
    api_service: Arc<dyn ApiService + Send + Sync>,
    progress_service: Arc<dyn ProgressService + Send + Sync>,
    cache_service: Option<Arc<dyn CacheService + Send + Sync>>,
    repository: Arc<dyn ContentRepository + Send + Sync>,
    config: AppConfig,
}

impl TaskService {
    /// Create a new TaskService with dependency injection
    pub fn new(
        api_service: Arc<dyn ApiService + Send + Sync>,
        progress_service: Arc<dyn ProgressService + Send + Sync>,
        cache_service: Option<Arc<dyn CacheService + Send + Sync>>,
        repository: Arc<dyn ContentRepository + Send + Sync>,
        config: AppConfig,
    ) -> Self {
        Self {
            api_service,
            progress_service,
            cache_service,
            repository,
            config,
        }
    }

    /// Execute a single scrape task
    pub async fn execute_scrape(
        &self,
        url: String,
        options: Option<ScrapeOptions>,
        format: OutputFormat,
    ) -> FirecrawlResult<CommandResult> {
        let command = ScrapeCommand::new(url.clone(), options, format);

        // Check cache first if enabled
        if let Some(cache_service) = &self.cache_service {
            if let Some(cached_result) = cache_service.get_scrape_result(&url, &format).await? {
                return Ok(cached_result);
            }
        }

        // Notify progress
        self.progress_service.notify_task_started(&url, "scrape").await;

        // Execute command
        let result = match command
            .execute(self.repository.as_ref(), &self.config.get_effective_output_dir())
            .await {
                Ok(result) => result,
                Err(e) => {
                    self.progress_service.notify_task_failed(&url, "scrape", &e).await;
                    return Err(e);
                }
            };

        // Cache result if enabled
        if let Some(cache_service) = &self.cache_service {
            cache_service.store_scrape_result(&url, &format, &result).await?;
        }

        // Notify completion
        self.progress_service.notify_task_completed(&url, "scrape").await;

        Ok(result)
    }

    /// Execute a single crawl task
    pub async fn execute_crawl(
        &self,
        url: String,
        options: Option<CrawlOptions>,
        format: OutputFormat,
    ) -> FirecrawlResult<CommandResult> {
        let command = CrawlCommand::new(url.clone(), options, format);

        // Check cache first if enabled
        if let Some(cache_service) = &self.cache_service {
            if let Some(cached_result) = cache_service.get_crawl_result(&url, &format).await? {
                return Ok(cached_result);
            }
        }

        // Notify progress
        self.progress_service.notify_task_started(&url, "crawl").await;

        // Execute command
        let result = match command
            .execute(self.repository.as_ref(), &self.config.get_effective_output_dir())
            .await {
                Ok(result) => result,
                Err(e) => {
                    self.progress_service.notify_task_failed(&url, "crawl", &e).await;
                    return Err(e);
                }
            };

        // Cache result if enabled
        if let Some(cache_service) = &self.cache_service {
            cache_service.store_crawl_result(&url, &format, &result).await?;
        }

        // Notify completion
        self.progress_service.notify_task_completed(&url, "crawl").await;

        Ok(result)
    }

    /// Execute multiple tasks concurrently
    pub async fn execute_batch(
        &self,
        tasks: Vec<TaskDefinition>,
    ) -> FirecrawlResult<Vec<CommandResult>> {
        // Create task queue based on configuration
        let queue = TaskQueueFactory::create_normal();

        // Add tasks to queue
        for task in tasks {
            let command: Box<dyn Command<Result = CommandResult> + Send + Sync> = match task {
                TaskDefinition::Scrape { url, options, format } => {
                    Box::new(ScrapeCommand::new(url, options, format))
                }
                TaskDefinition::Crawl { url, options, format } => {
                    Box::new(CrawlCommand::new(url, options, format))
                }
            };
            queue.enqueue(command).await;
        }

        // Execute all tasks
        queue.execute_all(self.repository.as_ref(), &self.config.get_effective_output_dir())
            .await
    }

    /// Get task execution statistics
    pub async fn get_statistics(&self) -> TaskStatistics {
        self.progress_service.get_statistics().await
    }

    /// Clear cache if caching is enabled
    pub async fn clear_cache(&self) -> FirecrawlResult<()> {
        if let Some(cache_service) = &self.cache_service {
            cache_service.clear().await?;
        }
        Ok(())
    }

    /// Check if a URL result is cached
    pub async fn is_cached(&self, url: &str, format: &OutputFormat) -> bool {
        if let Some(cache_service) = &self.cache_service {
            cache_service.exists(url, format).await.unwrap_or(false)
        } else {
            false
        }
    }
}

/// Task definition for batch operations
#[derive(Debug, Clone)]
pub enum TaskDefinition {
    Scrape {
        url: String,
        options: Option<ScrapeOptions>,
        format: OutputFormat,
    },
    Crawl {
        url: String,
        options: Option<CrawlOptions>,
        format: OutputFormat,
    },
}

impl TaskDefinition {
    /// Get the URL for this task
    pub fn url(&self) -> &str {
        match self {
            TaskDefinition::Scrape { url, .. } => url,
            TaskDefinition::Crawl { url, .. } => url,
        }
    }

    /// Get the output format for this task
    pub fn format(&self) -> &OutputFormat {
        match self {
            TaskDefinition::Scrape { format, .. } => format,
            TaskDefinition::Crawl { format, .. } => format,
        }
    }

    /// Get the task type
    pub fn task_type(&self) -> &str {
        match self {
            TaskDefinition::Scrape { .. } => "scrape",
            TaskDefinition::Crawl { .. } => "crawl",
        }
    }
}

/// Task execution statistics
#[derive(Debug, Clone, Default)]
pub struct TaskStatistics {
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub scrape_tasks: usize,
    pub crawl_tasks: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub average_execution_time: std::time::Duration,
}

impl TaskStatistics {
    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_tasks == 0 {
            0.0
        } else {
            (self.completed_tasks as f64 / self.total_tasks as f64) * 100.0
        }
    }

    /// Get cache hit rate as percentage
    pub fn cache_hit_rate(&self) -> f64 {
        let total_cache_operations = self.cache_hits + self.cache_misses;
        if total_cache_operations == 0 {
            0.0
        } else {
            (self.cache_hits as f64 / total_cache_operations as f64) * 100.0
        }
    }
}

/// Builder for TaskService
pub struct TaskServiceBuilder {
    api_service: Option<Arc<dyn ApiService + Send + Sync>>,
    progress_service: Option<Arc<dyn ProgressService + Send + Sync>>,
    cache_service: Option<Arc<dyn CacheService + Send + Sync>>,
    repository: Option<Arc<dyn ContentRepository + Send + Sync>>,
    config: Option<AppConfig>,
}

impl TaskServiceBuilder {
    pub fn new() -> Self {
        Self {
            api_service: None,
            progress_service: None,
            cache_service: None,
            repository: None,
            config: None,
        }
    }

    pub fn with_api_service(mut self, service: Arc<dyn ApiService + Send + Sync>) -> Self {
        self.api_service = Some(service);
        self
    }

    pub fn with_progress_service(mut self, service: Arc<dyn ProgressService + Send + Sync>) -> Self {
        self.progress_service = Some(service);
        self
    }

    pub fn with_cache_service(mut self, service: Option<Arc<dyn CacheService + Send + Sync>>) -> Self {
        self.cache_service = service;
        self
    }

    pub fn with_repository(mut self, repository: Arc<dyn ContentRepository + Send + Sync>) -> Self {
        self.repository = Some(repository);
        self
    }

    pub fn with_config(mut self, config: AppConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn build(self) -> FirecrawlResult<TaskService> {
        let api_service = self.api_service
            .ok_or_else(|| FirecrawlError::ConfigurationError(
                "ApiService is required".to_string()
            ))?;

        let progress_service = self.progress_service
            .ok_or_else(|| FirecrawlError::ConfigurationError(
                "ProgressService is required".to_string()
            ))?;

        let repository = self.repository
            .ok_or_else(|| FirecrawlError::ConfigurationError(
                "ContentRepository is required".to_string()
            ))?;

        let config = self.config
            .ok_or_else(|| FirecrawlError::ConfigurationError(
                "AppConfig is required".to_string()
            ))?;

        Ok(TaskService {
            api_service,
            progress_service,
            cache_service: self.cache_service,
            repository,
            config,
        })
    }
}

impl Default for TaskServiceBuilder {
    fn default() -> Self {
        Self::new()
    }
}