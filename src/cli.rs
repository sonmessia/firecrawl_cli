use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "fc_cli")]
#[command(about = "Firecrawl Rust CLI Tool")]
pub struct Cli {
    #[arg(long, default_value = "http://localhost:3002/v2")]
    pub api_url: String,

    #[arg(long)]
    pub api_key: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Scrape {
        url: String,
        #[arg(short, long, default_value = "./output")]
        output_dir: PathBuf,
    },
    Crawl {
        url: String,
        #[arg(short, long, default_value_t = 10)]
        limit: u32,
        #[arg(short, long, default_value = "./output")]
        output_dir: PathBuf,
    },
}
