use std::path::{Path, PathBuf};
use std::fs;
use std::time::Duration;

use crate::config::AppConfig;
use crate::errors::{FirecrawlError, FirecrawlResult};

/// Configuration file loader
pub struct ConfigLoader;

impl ConfigLoader {
    /// Default configuration file locations (in order of preference)
    pub fn default_config_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Current directory
        paths.push(PathBuf::from("firecrawl.toml"));
        paths.push(PathBuf::from("firecrawl.yaml"));
        paths.push(PathBuf::from("firecrawl.yml"));
        paths.push(PathBuf::from(".firecrawl.toml"));
        paths.push(PathBuf::from(".firecrawl.yaml"));
        paths.push(PathBuf::from(".firecrawl.yml"));

        // User home directory
        if let Some(home_dir) = dirs::home_dir() {
            paths.push(home_dir.join(".config").join("firecrawl").join("config.toml"));
            paths.push(home_dir.join(".config").join("firecrawl").join("config.yaml"));
            paths.push(home_dir.join(".config").join("firecrawl").join("config.yml"));
            paths.push(home_dir.join(".firecrawl.toml"));
            paths.push(home_dir.join(".firecrawl.yaml"));
            paths.push(home_dir.join(".firecrawl.yml"));
        }

        // System-wide configuration
        paths.push(PathBuf::from("/etc/firecrawl/config.toml"));
        paths.push(PathBuf::from("/etc/firecrawl/config.yaml"));
        paths.push(PathBuf::from("/etc/firecrawl/config.yml"));

        paths
    }

    /// Load configuration from the first found file
    pub fn load() -> FirecrawlResult<AppConfig> {
        let paths = Self::default_config_paths();

        for path in &paths {
            if path.exists() {
                match Self::load_from_file(path) {
                    Ok(config) => return Ok(config),
                    Err(e) => {
                        eprintln!("Warning: Failed to load config from {}: {}", path.display(), e);
                        // Continue to the next file
                    }
                }
            }
        }

        // No configuration file found, return default with environment overrides
        super::environment::load_from_env()
    }

    /// Load configuration from a specific file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> FirecrawlResult<AppConfig> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .map_err(|e| FirecrawlError::ConfigurationError(
                format!("Failed to read config file {}: {}", path.display(), e)
            ))?;

        let config = match path.extension().and_then(|ext| ext.to_str()) {
            Some("toml") => Self::parse_toml(&content)?,
            Some("yaml") | Some("yml") => Self::parse_yaml(&content)?,
            Some(ext) => {
                return Err(FirecrawlError::ConfigurationError(
                    format!("Unsupported config file format: {}", ext)
                ));
            }
            None => {
                return Err(FirecrawlError::ConfigurationError(
                    "Config file has no extension".to_string()
                ));
            }
        };

        // Override with environment variables
        Self::apply_env_overrides(config)
    }

    /// Save configuration to a file
    pub fn save_to_file<P: AsRef<Path>>(config: &AppConfig, path: P) -> FirecrawlResult<()> {
        let path = path.as_ref();

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| FirecrawlError::ConfigurationError(
                    format!("Failed to create directory {}: {}", parent.display(), e)
                ))?;
        }

        let content = match path.extension().and_then(|ext| ext.to_str()) {
            Some("toml") => Self::serialize_toml(config)?,
            Some("yaml") | Some("yml") => Self::serialize_yaml(config)?,
            Some(ext) => {
                return Err(FirecrawlError::ConfigurationError(
                    format!("Unsupported config file format: {}", ext)
                ));
            }
            None => {
                return Err(FirecrawlError::ConfigurationError(
                    "Config file has no extension".to_string()
                ));
            }
        };

        fs::write(path, content)
            .map_err(|e| FirecrawlError::ConfigurationError(
                format!("Failed to write config file {}: {}", path.display(), e)
            ))?;

        Ok(())
    }

    /// Parse TOML configuration
    fn parse_toml(content: &str) -> FirecrawlResult<AppConfig> {
        toml::from_str(content)
            .map_err(|e| FirecrawlError::ConfigurationError(
                format!("Failed to parse TOML config: {}", e)
            ))
    }

    /// Parse YAML configuration
    fn parse_yaml(content: &str) -> FirecrawlResult<AppConfig> {
        serde_yaml::from_str(content)
            .map_err(|e| FirecrawlError::ConfigurationError(
                format!("Failed to parse YAML config: {}", e)
            ))
    }

    /// Serialize configuration to TOML
    fn serialize_toml(config: &AppConfig) -> FirecrawlResult<String> {
        toml::to_string_pretty(config)
            .map_err(|e| FirecrawlError::ConfigurationError(
                format!("Failed to serialize TOML config: {}", e)
            ))
    }

    /// Serialize configuration to YAML
    fn serialize_yaml(config: &AppConfig) -> FirecrawlResult<String> {
        serde_yaml::to_string(config)
            .map_err(|e| FirecrawlError::ConfigurationError(
                format!("Failed to serialize YAML config: {}", e)
            ))
    }

    /// Apply environment variable overrides to the loaded configuration
    fn apply_env_overrides(mut config: AppConfig) -> FirecrawlResult<AppConfig> {
        // Environment variables take precedence over file configuration
        let env_config = super::environment::load_from_env()?;

        // Override with environment values
        if env_config.api.api_key.is_some() {
            config.api.api_key = env_config.api.api_key;
        }
        if env_config.api.base_url != "https://api.firecrawl.dev" {
            config.api.base_url = env_config.api.base_url;
        }
        if env_config.api.timeout != Duration::from_secs(30) {
            config.api.timeout = env_config.api.timeout;
        }

        if env_config.output.default_directory != PathBuf::from("./output") {
            config.output.default_directory = env_config.output.default_directory;
        }
        if env_config.output.default_format != crate::cli::OutputFormat::Markdown {
            config.output.default_format = env_config.output.default_format;
        }

        if env_config.execution.max_concurrent_tasks != 4 {
            config.execution.max_concurrent_tasks = env_config.execution.max_concurrent_tasks;
        }
        if env_config.execution.verbose_logging {
            config.execution.verbose_logging = env_config.execution.verbose_logging;
        }

        config.validate()?;
        Ok(config)
    }

    /// Generate a sample configuration file
    pub fn generate_sample_config() -> String {
        let sample_config = AppConfig::default();
        Self::serialize_toml(&sample_config).unwrap_or_else(|_| {
            r#"# Firecrawl CLI Configuration

[api]
base_url = "https://api.firecrawl.dev"
# api_key = "your-api-key-here"
timeout = 30  # seconds
max_retries = 3
retry_delay = 1000  # milliseconds

[output]
default_directory = "./output"
default_format = "markdown"
create_date_subdirectories = false
# filename_prefix = "firecrawl_"
overwrite_existing = false
max_filename_length = 255

[execution]
max_concurrent_tasks = 4
# default_crawl_limit = 10
progress_update_interval = 500  # milliseconds
verbose_logging = false

[execution.cache]
enabled = false
directory = "./cache"
ttl = 3600  # seconds
max_size_mb = 100

[ui]
enable_colors = true

[ui.theme]
color_scheme = "default"
use_unicode = true

[ui.tui]
refresh_rate = 100  # milliseconds
max_log_lines = 1000
show_help_by_default = true
"#
            .to_string()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_save_and_load_toml() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("test_config.toml");

        let original_config = AppConfig::default();

        // Save configuration
        ConfigLoader::save_to_file(&original_config, &config_path).unwrap();
        assert!(config_path.exists());

        // Load configuration
        let loaded_config = ConfigLoader::load_from_file(&config_path).unwrap();

        // Compare (basic check)
        assert_eq!(original_config.api.base_url, loaded_config.api.base_url);
        assert_eq!(original_config.output.default_format, loaded_config.output.default_format);
    }

    #[test]
    fn test_generate_sample_config() {
        let sample = ConfigLoader::generate_sample_config();
        assert!(!sample.is_empty());
        assert!(sample.contains("[api]"));
        assert!(sample.contains("[output]"));
    }

    #[test]
    fn test_default_config_paths() {
        let paths = ConfigLoader::default_config_paths();
        assert!(!paths.is_empty());

        // Check that we have both local and home directory paths
        assert!(paths.iter().any(|p| p.to_string_lossy().contains("firecrawl.toml")));
    }
}