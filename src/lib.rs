//! # Firecrawl CLI
//!
//! A comprehensive Rust-based CLI tool for web scraping and crawling using the Firecrawl API.
//! This library provides both programmatic access and command-line interface functionality.
//!
//! ## Features
//!
//! - **Multiple Output Formats**: Support for Markdown, HTML, JSON, and raw text
//! - **Batch Processing**: Concurrent scraping and crawling with configurable limits
//! - **Caching**: Optional result caching to improve performance
//! - **Progress Monitoring**: Real-time progress tracking and notifications
//! - **Configuration Management**: Flexible configuration via files, environment variables, and CLI arguments
//! - **Error Handling**: Comprehensive error handling with retry logic
//! - **Terminal UI**: Interactive terminal user interface for task management
//!
//! ## Architecture
//!
//! This library is built using several design patterns:
//!
//! - **Repository Pattern**: For file storage operations
//! - **Strategy Pattern**: For different output formatters
//! - **Command Pattern**: For task execution
//! - **Observer Pattern**: For progress monitoring
//! - **Builder Pattern**: For flexible configuration
//! - **Dependency Injection**: For testable and modular components
//!
//! ## Quick Start
//!
//! ```rust
//! use firecrawl_cli::{
//!     services::{TaskService, TaskServiceBuilder},
//!     config::{AppConfig, ConfigLoader},
//!     cli::OutputFormat,
//! };
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load configuration
//!     let config = ConfigLoader::load()?;
//!
//!     // Create task service with all dependencies
//!     let task_service = TaskServiceBuilder::new()
//!         .with_config(config)
//!         .build()?;
//!
//!     // Execute a scrape task
//!     let result = task_service.execute_scrape(
//!         "https://example.com".to_string(),
//!         None,
//!         OutputFormat::Markdown,
//!     ).await?;
//!
//!     println!("Scraped successfully: {:?}", result);
//!     Ok(())
//! }
//! ```

pub mod api;
pub mod cli;
pub mod commands;
pub mod config;
pub mod errors;
pub mod services;
pub mod storage;
pub mod tui;
pub mod utils;

// Re-export commonly used types for convenience
pub use crate::api::client_builder::{FirecrawlClientBuilder, FirecrawlClientFactory};
pub use crate::api::services::client::FirecrawlClient;
pub use crate::cli::{Action, Cli, CrawlOptions, OutputFormat, ScrapeOptions};
pub use crate::commands::{Command, CommandResult, TaskQueue, TaskQueueFactory};
pub use crate::config::{AppConfig, ConfigLoader};
pub use crate::errors::{ContextualError, ErrorContext, FirecrawlError, FirecrawlResult};
pub use crate::services::{
    ApiService, ApiServiceFactory, CacheService, CacheServiceFactory, FileService,
    FileServiceFactory, ProgressService, ProgressServiceFactory, TaskService, TaskServiceBuilder,
};
pub use crate::storage::{ContentRepository, ContentSaver, FileSystemRepository};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default API base URL
pub const DEFAULT_API_URL: &str = "https://api.firecrawl.dev";
