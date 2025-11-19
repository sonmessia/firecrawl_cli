use clap::{Parser, Subcommand};
use std::path::PathBuf;

// Main CLI structure using clap for argument parsing
#[derive(Parser)]
#[command(name = "fc_cli")]
#[command(about = "Firecrawl Rust CLI Tool")]
pub struct Cli {
    // Base URL for the Firecrawl API (defaults to local instance or FIRE_API_URL env var)
    #[arg(
        long,
        env = "FIRE_API_URL",
        default_value = "http://localhost:3002/v2"
    )]
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
