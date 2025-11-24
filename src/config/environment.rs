use std::env;
use std::path::PathBuf;
use std::time::Duration;

use crate::config::AppConfig;
use crate::cli::OutputFormat;
use crate::errors::FirecrawlResult;

/// Environment variable names
pub mod env_vars {
    pub const API_URL: &str = "FIRECRAWL_API_URL";
    pub const API_KEY: &str = "FIRECRAWL_API_KEY";
    pub const TIMEOUT: &str = "FIRECRAWL_TIMEOUT";
    pub const MAX_RETRIES: &str = "FIRECRAWL_MAX_RETRIES";
    pub const RETRY_DELAY: &str = "FIRECRAWL_RETRY_DELAY";
    pub const OUTPUT_DIR: &str = "FIRECRAWL_OUTPUT_DIR";
    pub const DEFAULT_FORMAT: &str = "FIRECRAWL_DEFAULT_FORMAT";
    pub const MAX_CONCURRENT_TASKS: &str = "FIRECRAWL_MAX_CONCURRENT_TASKS";
    pub const VERBOSE_LOGGING: &str = "FIRECRAWL_VERBOSE";
    pub const CACHE_ENABLED: &str = "FIRECRAWL_CACHE_ENABLED";
    pub const CACHE_DIR: &str = "FIRECRAWL_CACHE_DIR";
    pub const PROXY_URL: &str = "FIRECRAWL_PROXY_URL";
    pub const USER_AGENT: &str = "FIRECRAWL_USER_AGENT";
    pub const ENABLE_COLORS: &str = "FIRECRAWL_COLORS";
}

/// Load configuration from environment variables
pub fn load_from_env() -> FirecrawlResult<AppConfig> {
    let mut config = AppConfig::default();

    // API configuration
    if let Ok(url) = env::var(env_vars::API_URL) {
        config.api.base_url = url;
    }

    if let Ok(key) = env::var(env_vars::API_KEY) {
        config.api.api_key = Some(key);
    }

    if let Ok(timeout_str) = env::var(env_vars::TIMEOUT) {
        if let Ok(secs) = timeout_str.parse::<u64>() {
            config.api.timeout = Duration::from_secs(secs);
        }
    }

    if let Ok(retries_str) = env::var(env_vars::MAX_RETRIES) {
        if let Ok(retries) = retries_str.parse::<u32>() {
            config.api.max_retries = retries;
        }
    }

    if let Ok(delay_str) = env::var(env_vars::RETRY_DELAY) {
        if let Ok(millis) = delay_str.parse::<u64>() {
            config.api.retry_delay = Duration::from_millis(millis);
        }
    }

    if let Ok(user_agent) = env::var(env_vars::USER_AGENT) {
        config.api.user_agent = Some(user_agent);
    }

    // Proxy configuration
    if let Ok(proxy_url) = env::var(env_vars::PROXY_URL) {
        config.api.proxy = Some(super::ProxyConfig {
            url: proxy_url,
            username: env::var("FIRECRAWL_PROXY_USERNAME").ok(),
            password: env::var("FIRECRAWL_PROXY_PASSWORD").ok(),
        });
    }

    // Output configuration
    if let Ok(output_dir) = env::var(env_vars::OUTPUT_DIR) {
        config.output.default_directory = PathBuf::from(output_dir);
    }

    if let Ok(format_str) = env::var(env_vars::DEFAULT_FORMAT) {
        if let Ok(format) = parse_output_format(&format_str) {
            config.output.default_format = format;
        }
    }

    // Execution configuration
    if let Ok(max_tasks_str) = env::var(env_vars::MAX_CONCURRENT_TASKS) {
        if let Ok(max_tasks) = max_tasks_str.parse::<usize>() {
            config.execution.max_concurrent_tasks = max_tasks;
        }
    }

    if let Ok(verbose_str) = env::var(env_vars::VERBOSE_LOGGING) {
        config.execution.verbose_logging = parse_bool(&verbose_str);
    }

    // Cache configuration
    if let Ok(cache_str) = env::var(env_vars::CACHE_ENABLED) {
        config.execution.cache.enabled = parse_bool(&cache_str);
    }

    if let Ok(cache_dir) = env::var(env_vars::CACHE_DIR) {
        config.execution.cache.directory = PathBuf::from(cache_dir);
    }

    // UI configuration
    if let Ok(colors_str) = env::var(env_vars::ENABLE_COLORS) {
        config.ui.enable_colors = parse_bool(&colors_str);
    }

    // Validate the loaded configuration
    config.validate()?;

    Ok(config)
}

/// Parse output format from string
fn parse_output_format(format_str: &str) -> Result<OutputFormat, ()> {
    match format_str.to_lowercase().as_str() {
        "markdown" | "md" => Ok(OutputFormat::Markdown),
        "html" => Ok(OutputFormat::Html),
        "json" => Ok(OutputFormat::Json),
        "raw" | "txt" => Ok(OutputFormat::Raw),
        _ => Err(()),
    }
}

/// Parse boolean value from string
fn parse_bool(value: &str) -> bool {
    match value.to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" | "enabled" => true,
        "false" | "0" | "no" | "off" | "disabled" => false,
        _ => false,
    }
}

/// Get environment documentation
pub fn get_env_docs() -> String {
    format!(
        r#"Environment Variables for Firecrawl CLI:

API Configuration:
  {}              Base URL for the Firecrawl API (default: https://api.firecrawl.dev)
  {}                API key for authentication
  {}              Request timeout in seconds (default: 30)
  {}           Maximum number of retry attempts (default: 3)
  {}          Delay between retries in milliseconds (default: 1000)
  {}        Custom User-Agent string
  {}            Proxy URL (e.g., http://proxy.example.com:8080)

Output Configuration:
  {}          Default output directory (default: ./output)
  {}       Default output format (markdown, html, json, raw)

Execution Configuration:
  {}   Maximum number of concurrent tasks (default: 4)
  {}         Enable verbose logging (true/false)
  {}     Enable result caching (true/false)
  {}        Cache directory for storing results

UI Configuration:
  {}         Enable colored output (true/false)

Proxy Authentication:
  FIRECRAWL_PROXY_USERNAME   Proxy username (optional)
  FIRECRAWL_PROXY_PASSWORD   Proxy password (optional)

Examples:
  export FIRECRAWL_API_KEY="your-api-key-here"
  export FIRECRAWL_OUTPUT_DIR="./my-output"
  export FIRECRAWL_MAX_CONCURRENT_TASKS=8
  export FIRECRAWL_VERBOSE=true
"#,
        env_vars::API_URL,
        env_vars::API_KEY,
        env_vars::TIMEOUT,
        env_vars::MAX_RETRIES,
        env_vars::RETRY_DELAY,
        env_vars::USER_AGENT,
        env_vars::PROXY_URL,
        env_vars::OUTPUT_DIR,
        env_vars::DEFAULT_FORMAT,
        env_vars::MAX_CONCURRENT_TASKS,
        env_vars::VERBOSE_LOGGING,
        env_vars::CACHE_ENABLED,
        env_vars::CACHE_DIR,
        env_vars::ENABLE_COLORS
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_parse_output_format() {
        assert_eq!(parse_output_format("markdown"), Ok(OutputFormat::Markdown));
        assert_eq!(parse_output_format("MD"), Ok(OutputFormat::Markdown));
        assert_eq!(parse_output_format("html"), Ok(OutputFormat::Html));
        assert_eq!(parse_output_format("HTML"), Ok(OutputFormat::Html));
        assert_eq!(parse_output_format("json"), Ok(OutputFormat::Json));
        assert_eq!(parse_output_format("raw"), Ok(OutputFormat::Raw));
        assert_eq!(parse_output_format("txt"), Ok(OutputFormat::Raw));
        assert!(parse_output_format("invalid").is_err());
    }

    #[test]
    fn test_parse_bool() {
        assert!(parse_bool("true"));
        assert!(parse_bool("TRUE"));
        assert!(parse_bool("1"));
        assert!(parse_bool("yes"));
        assert!(parse_bool("YES"));
        assert!(parse_bool("on"));
        assert!(parse_bool("enabled"));

        assert!(!parse_bool("false"));
        assert!(!parse_bool("FALSE"));
        assert!(!parse_bool("0"));
        assert!(!parse_bool("no"));
        assert!(!parse_bool("off"));
        assert!(!parse_bool("disabled"));
        assert!(!parse_bool("invalid"));
    }

    #[test]
    fn test_load_from_env() {
        // Set some test environment variables
        env::set_var(env_vars::API_KEY, "test-key");
        env::set_var(env_vars::TIMEOUT, "60");
        env::set_var(env_vars::MAX_CONCURRENT_TASKS, "8");
        env::set_var(env_vars::VERBOSE_LOGGING, "true");

        let config = load_from_env().unwrap();

        assert_eq!(config.api.api_key, Some("test-key".to_string()));
        assert_eq!(config.api.timeout, Duration::from_secs(60));
        assert_eq!(config.execution.max_concurrent_tasks, 8);
        assert!(config.execution.verbose_logging);

        // Clean up
        env::remove_var(env_vars::API_KEY);
        env::remove_var(env_vars::TIMEOUT);
        env::remove_var(env_vars::MAX_CONCURRENT_TASKS);
        env::remove_var(env_vars::VERBOSE_LOGGING);
    }
}