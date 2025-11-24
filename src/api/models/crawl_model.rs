use super::scrape_model::{OutputFormat, ScrapeData};
use serde::{Deserialize, Serialize};
use chrono;

// Re-export the CLI CrawlOptions to maintain consistency
pub use crate::cli::CrawlOptions;

// Main crawl request structure
#[derive(Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct CrawlRequest {
    pub url: String, // Starting URL for crawling

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>, // Maximum number of pages to crawl

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub formats: Option<Vec<OutputFormat>>, // Output formats for each crawled page

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub only_main_content: Option<bool>, // Extract only main content for each page
}

// Response received when starting a new crawl job
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CrawlStartResponse {
    pub job_id: String, // Unique identifier for the crawl job
}

// Crawl response structure for individual crawled pages
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CrawlResponse {
    pub id: String,
    pub url: String,
    pub status: String,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub markdown: Option<String>,
    pub html: Option<String>,
    pub metadata: CrawlMetadata,
}

// Metadata for crawl responses
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CrawlMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub robots: Option<String>,
    pub og_image: Option<String>,
    pub page_title: Option<String>,
    pub author: Option<String>,
    pub published_date: Option<chrono::DateTime<chrono::Utc>>,
    pub modified_date: Option<chrono::DateTime<chrono::Utc>>,
    pub site_name: Option<String>,
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

// Builder for CrawlRequest
pub struct CrawlRequestBuilder {
    url: String,
    limit: Option<u32>,
    formats: Option<Vec<OutputFormat>>,
    only_main_content: Option<bool>,
}

impl CrawlRequestBuilder {
    pub fn new(url: String) -> Self {
        Self {
            url,
            limit: None,
            formats: None,
            only_main_content: None,
        }
    }

    pub fn url(mut self, url: String) -> Self {
        self.url = url;
        self
    }

    pub fn limit(mut self, limit: Option<u32>) -> Self {
        self.limit = limit;
        self
    }

    pub fn formats(mut self, formats: Option<Vec<OutputFormat>>) -> Self {
        self.formats = formats;
        self
    }

    pub fn only_main_content(mut self, only_main_content: Option<bool>) -> Self {
        self.only_main_content = only_main_content;
        self
    }

    pub fn build(self) -> Result<CrawlRequest, String> {
        Ok(CrawlRequest {
            url: self.url,
            limit: self.limit,
            formats: self.formats,
            only_main_content: self.only_main_content,
        })
    }
}

impl CrawlRequest {
    pub fn builder() -> CrawlRequestBuilder {
        CrawlRequestBuilder {
            url: String::new(),
            limit: None,
            formats: None,
            only_main_content: None,
        }
    }
}
