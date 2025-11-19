use super::scrape_model::{OutputFormat, ScrapeData};
use serde::{Deserialize, Serialize};

// Main crawl request structure
#[derive(Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct CrawlRequest {
    pub url: String, // Starting URL for crawling

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>, // Maximum number of pages to crawl

    // Embed scrape options to configure how each crawled page is processed
    #[serde(flatten)]
    pub crawl_options: CrawlOptions,
}

// Crawl-specific options that apply to all pages during crawling
#[derive(Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct CrawlOptions {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub formats: Vec<OutputFormat>, // Output formats for each crawled page

    #[serde(skip_serializing_if = "Option::is_none")]
    pub only_main_content: Option<bool>, // Extract only main content for each page

                                         // Additional crawl options can be added here as needed
                                         // pub include_tags: Option<Vec<String>>,
                                         // pub exclude_tags: Option<Vec<String>>,
}

// Response received when starting a new crawl job
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CrawlStartResponse {
    pub job_id: String, // Unique identifier for the crawl job
}

// Enum representing the different states a crawl job can be in
#[derive(Debug, Clone)]
pub enum CrawlState {
    // Crawl job has just been initiated
    Started {
        job_id: String,
    },
    // Crawl job is currently processing pages
    InProgress {
        job_id: String,
        status: String, // Current status text from API
        completed: u32, // Number of pages completed
        total: u32,     // Total number of pages expected
    },
    // Crawl job has completed successfully
    Completed {
        job_id: String,
        data: Vec<ScrapeData>, // All scraped page data
    },
    // Crawl job has failed
    Failed {
        job_id: String,
        error: String, // Error description
    },
}

// Response structure for checking crawl job status
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CrawlStatusResponse {
    pub status: String, // Current status ("started", "completed", "failed", etc.)
    pub completed: Option<u32>, // Number of pages completed
    pub total: Option<u32>, // Total number of pages expected
    pub data: Option<Vec<ScrapeData>>, // Scrape data when completed
    pub error: Option<String>, // Error message when failed
}
