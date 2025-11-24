use std::path::PathBuf;
use std::time::Duration;
use serde::{Deserialize, Serialize};

use crate::cli::OutputFormat;
use super::errors::{FirecrawlError, FirecrawlResult};

pub mod loader;
pub mod environment;

pub use loader::*;
pub use environment::*;

/// Application configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// API configuration
    pub api: ApiConfig,

    /// Output configuration
    pub output: OutputConfig,

    /// Execution configuration
    pub execution: ExecutionConfig,

    /// UI configuration
    pub ui: UiConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api: ApiConfig::default(),
            output: OutputConfig::default(),
            execution: ExecutionConfig::default(),
            ui: UiConfig::default(),
        }
    }
}

/// API-related configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Base URL for the Firecrawl API
    pub base_url: String,

    /// API key for authentication
    pub api_key: Option<String>,

    /// Request timeout in seconds
    pub timeout: Duration,

    /// Maximum number of retry attempts
    pub max_retries: u32,

    /// Delay between retries in milliseconds
    pub retry_delay: Duration,

    /// User agent string
    pub user_agent: Option<String>,

    /// Proxy configuration
    pub proxy: Option<ProxyConfig>,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            base_url: "https://api.firecrawl.dev".to_string(),
            api_key: None,
            timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_delay: Duration::from_millis(1000),
            user_agent: None,
            proxy: None,
        }
    }
}

/// Proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Proxy URL
    pub url: String,

    /// Optional username for authentication
    pub username: Option<String>,

    /// Optional password for authentication
    pub password: Option<String>,
}

/// Output-related configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Default output directory
    pub default_directory: PathBuf,

    /// Default output format
    pub default_format: OutputFormat,

    /// Whether to create date-based subdirectories
    pub create_date_subdirectories: bool,

    /// Filename prefix
    pub filename_prefix: Option<String>,

    /// Whether to overwrite existing files
    pub overwrite_existing: bool,

    /// Maximum filename length
    pub max_filename_length: usize,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            default_directory: PathBuf::from("./output"),
            default_format: OutputFormat::Markdown,
            create_date_subdirectories: false,
            filename_prefix: None,
            overwrite_existing: false,
            max_filename_length: 255,
        }
    }
}

/// Execution-related configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// Maximum number of concurrent tasks
    pub max_concurrent_tasks: usize,

    /// Default crawl limit
    pub default_crawl_limit: Option<u32>,

    /// Progress update interval in milliseconds
    pub progress_update_interval: Duration,

    /// Whether to enable verbose logging
    pub verbose_logging: bool,

    /// Cache configuration
    pub cache: CacheConfig,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 4,
            default_crawl_limit: Some(10),
            progress_update_interval: Duration::from_millis(500),
            verbose_logging: false,
            cache: CacheConfig::default(),
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Whether to enable caching
    pub enabled: bool,

    /// Cache directory
    pub directory: PathBuf,

    /// Cache TTL in seconds
    pub ttl: Duration,

    /// Maximum cache size in MB
    pub max_size_mb: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            directory: PathBuf::from("./cache"),
            ttl: Duration::from_secs(3600), // 1 hour
            max_size_mb: 100,
        }
    }
}

/// UI-related configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Whether to enable colors
    pub enable_colors: bool,

    /// Theme configuration
    pub theme: ThemeConfig,

    /// TUI configuration
    pub tui: TuiConfig,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            enable_colors: true,
            theme: ThemeConfig::default(),
            tui: TuiConfig::default(),
        }
    }
}

/// Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Color scheme
    pub color_scheme: String,

    /// Whether to use Unicode characters
    pub use_unicode: bool,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            color_scheme: "default".to_string(),
            use_unicode: true,
        }
    }
}

/// TUI-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiConfig {
    /// Refresh rate in milliseconds
    pub refresh_rate: Duration,

    /// Maximum log lines to display
    pub max_log_lines: usize,

    /// Whether to show the help panel by default
    pub show_help_by_default: bool,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            refresh_rate: Duration::from_millis(100),
            max_log_lines: 1000,
            show_help_by_default: true,
        }
    }
}

impl AppConfig {
    /// Validate the configuration
    pub fn validate(&self) -> FirecrawlResult<()> {
        // Validate API URL
        if self.api.base_url.is_empty() {
            return Err(FirecrawlError::ConfigurationError(
                "API base URL cannot be empty".to_string()
            ));
        }

        // Validate timeout
        if self.api.timeout.as_secs() == 0 {
            return Err(FirecrawlError::ConfigurationError(
                "API timeout must be greater than 0".to_string()
            ));
        }

        // Validate concurrent tasks
        if self.execution.max_concurrent_tasks == 0 {
            return Err(FirecrawlError::ConfigurationError(
                "Max concurrent tasks must be greater than 0".to_string()
            ));
        }

        // Validate max filename length
        if self.output.max_filename_length == 0 {
            return Err(FirecrawlError::ConfigurationError(
                "Max filename length must be greater than 0".to_string()
            ));
        }

        // Validate proxy configuration if present
        if let Some(proxy) = &self.api.proxy {
            if proxy.url.is_empty() {
                return Err(FirecrawlError::ConfigurationError(
                    "Proxy URL cannot be empty".to_string()
                ));
            }
        }

        Ok(())
    }

    /// Get the effective output directory (taking into account date subdirectories)
    pub fn get_effective_output_dir(&self) -> PathBuf {
        let mut dir = self.output.default_directory.clone();

        if self.output.create_date_subdirectories {
            let date_str = chrono::Utc::now().format("%Y-%m-%d").to_string();
            dir.push(date_str);
        }

        dir
    }

    /// Create builder for configuration
    pub fn builder() -> AppConfigBuilder {
        AppConfigBuilder::new()
    }
}

/// Builder pattern for AppConfig
pub struct AppConfigBuilder {
    config: AppConfig,
}

impl AppConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: AppConfig::default(),
        }
    }

    pub fn api_url(mut self, url: String) -> Self {
        self.config.api.base_url = url;
        self
    }

    pub fn api_key(mut self, key: Option<String>) -> Self {
        self.config.api.api_key = key;
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.api.timeout = timeout;
        self
    }

    pub fn output_directory(mut self, dir: PathBuf) -> Self {
        self.config.output.default_directory = dir;
        self
    }

    pub fn output_format(mut self, format: OutputFormat) -> Self {
        self.config.output.default_format = format;
        self
    }

    pub fn max_concurrent_tasks(mut self, max: usize) -> Self {
        self.config.execution.max_concurrent_tasks = max;
        self
    }

    pub fn enable_caching(mut self, enabled: bool) -> Self {
        self.config.execution.cache.enabled = enabled;
        self
    }

    pub fn verbose_logging(mut self, enabled: bool) -> Self {
        self.config.execution.verbose_logging = enabled;
        self
    }

    pub fn build(self) -> FirecrawlResult<AppConfig> {
        self.config.validate()?;
        Ok(self.config)
    }
}