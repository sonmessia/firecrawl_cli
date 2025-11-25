use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// Main CLI structure using clap for argument parsing
#[derive(Parser)]
#[command(name = "fc_cli")]
#[command(about = "Firecrawl Rust CLI Tool")]
pub struct Cli {
    // Base URL for the Firecrawl API (defaults to local instance or FIRE_API_URL env var)
    #[arg(long, env = "FIRE_API_URL", default_value = "http://localhost:3002/v2")]
    pub api_url: String,

    // Optional API key for authentication (or FIRE_API_KEY env var)
    #[arg(long, env = "FIRE_API_KEY")]
    pub api_key: Option<String>,

    // Flag to launch TUI mode instead of CLI commands
    #[arg(short, long, help = "Launch Terminal User Interface")]
    pub tui: bool,

    // Subcommands for different operations (scrape/crawl)
    #[command(subcommand)]
    pub command: Option<Commands>,
}

// Enumeration of available CLI commands
#[derive(Subcommand)]
pub enum Commands {
    // Scrape command for single-page content extraction
    Scrape {
        // Target URL to scrape
        url: String,
        // Output directory for saved files (defaults to ./output)
        #[arg(short, long, default_value = "./output")]
        output_dir: PathBuf,
    },
    // Crawl command for multi-page content extraction
    Crawl {
        // Starting URL for crawling
        url: String,
        // Maximum number of pages to crawl (defaults to 10)
        #[arg(short, long, default_value_t = 10)]
        limit: u32,
        // Output directory for saved files (defaults to ./output)
        #[arg(short, long, default_value = "./output")]
        output_dir: PathBuf,
    },
}

/// Output format options
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OutputFormat {
    #[serde(rename = "markdown")]
    Markdown,
    #[serde(rename = "html")]
    Html,
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "raw")]
    Raw,
    #[serde(rename = "rawHtml")]
    RawHtml,
    #[serde(rename = "links")]
    Links,
    #[serde(rename = "images")]
    Images,
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Markdown
    }
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Markdown => write!(f, "markdown"),
            OutputFormat::Html => write!(f, "html"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Raw => write!(f, "raw"),
            OutputFormat::RawHtml => write!(f, "rawHtml"),
            OutputFormat::Links => write!(f, "links"),
            OutputFormat::Images => write!(f, "images"),
        }
    }
}

/// Scrape operation options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrapeOptions {
    pub only_main_content: Option<bool>,
    pub include_tags: Option<Vec<String>>,
    pub exclude_tags: Option<Vec<String>>,
    pub formats: Option<Vec<OutputFormat>>,
}

impl Default for ScrapeOptions {
    fn default() -> Self {
        Self {
            only_main_content: None,
            include_tags: None,
            exclude_tags: None,
            formats: None,
        }
    }
}

/// Crawl operation options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrawlOptions {
    pub limit: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub formats: Option<Vec<OutputFormat>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub only_main_content: Option<bool>,
}

impl Default for CrawlOptions {
    fn default() -> Self {
        Self {
            limit: None,
            formats: None,
            only_main_content: None,
        }
    }
}

/// Action enum for task types
#[derive(Debug, Clone)]
pub enum Action {
    Scrape,
    Crawl,
}
