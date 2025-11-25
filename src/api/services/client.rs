use anyhow::{Result, anyhow};
use chrono;
use reqwest::Client;
use std::time::Duration;
use tokio::time::sleep;

use crate::api::{
    ApiResponse, CrawlRequest, CrawlResponse, CrawlStartResponse, CrawlState, CrawlStatusResponse,
    OutputFormat, ScrapeData, ScrapeRequest, ScrapeResponse,
};
use crate::services::CrawlMonitorService;
use std::boxed::Box;

// Main HTTP client for interacting with the Firecrawl API
#[derive(Clone)]
pub struct FirecrawlClient {
    client: Client,          // Reqwest HTTP client
    base_url: String,        // Base URL for the API
    api_key: Option<String>, // Optional API key for authentication
}

impl FirecrawlClient {
    // Create a new FirecrawlClient with the given base URL and optional API key
    pub fn new(base_url: &str, api_key: Option<&str>) -> Result<Self> {
        // Build HTTP client with 5-minute timeout
        let client = Client::builder()
            .timeout(Duration::from_secs(300))
            .build()?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.map(|k| k.to_string()),
        })
    }

    // Add authorization header to requests if API key is available
    fn add_auth_headers(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(api_key) = &self.api_key {
            request.header("Authorization", format!("Bearer {}", api_key))
        } else {
            request
        }
    }

    // Scrape a single URL and return the extracted content
    pub async fn scrape(&self, url: &str) -> Result<ScrapeData> {
        // Build scrape request with multiple output formats
        let request = ScrapeRequest {
            url: url.to_string(),
            formats: vec![
                OutputFormat::Markdown,
                OutputFormat::RawHtml,
                OutputFormat::Html,
            ],
            only_main_content: Some(true),
            ..Default::default()
        };

        // Send scrape request to the API
        let response = self
            .add_auth_headers(
                self.client
                    .post(format!("{}/scrape", self.base_url))
                    .json(&request),
            )
            .send()
            .await?;

        // Handle error responses
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Scrape request failed: {} - {}",
                status,
                error_text
            ));
        }

        // Parse and return the response
        let api_response: ApiResponse<ScrapeData> = response.json().await?;

        if api_response.success {
            Ok(api_response.data)
        } else {
            Err(anyhow!("API request failed"))
        }
    }

    // Crawl a URL (with optional page limit) and return results from all crawled pages
    pub async fn crawl(&self, url: &str, limit: Option<u32>) -> Result<Vec<ScrapeData>> {
        // Start the crawl job
        let request = CrawlRequest {
            url: url.to_string(),
            limit,
            ..Default::default()
        };

        // Send crawl start request to the API
        let response = self
            .add_auth_headers(
                self.client
                    .post(format!("{}/crawl", self.base_url))
                    .json(&request),
            )
            .send()
            .await?;

        // Handle error responses
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Crawl start failed: {} - {}", status, error_text));
        }

        // Extract job ID from the response
        let start_response: CrawlStartResponse = response.json().await?;
        let job_id = start_response.job_id;

        // Poll for crawl completion
        loop {
            let state = self.check_crawl_status(&job_id).await?;

            match state {
                CrawlState::Completed { data, .. } => return Ok(data),
                CrawlState::Failed { error, .. } => return Err(anyhow!("Crawl failed: {}", error)),
                CrawlState::InProgress {
                    completed, total, ..
                } => {
                    // Display progress updates
                    println!("⏳ Progress: {}/{}", completed, total);
                }
                CrawlState::Started { .. } => {
                    println!("🚀 Crawl job started");
                }
            }

            // Wait 2 seconds before next status check
            sleep(Duration::from_secs(2)).await;
        }
    }

    // Check the status of a crawl job using its ID
    async fn check_crawl_status(&self, job_id: &str) -> Result<CrawlState> {
        // Send status check request to the API
        let response = self
            .add_auth_headers(
                self.client
                    .get(format!("{}/crawl/{}", self.base_url, job_id)),
            )
            .send()
            .await?;

        // Handle error responses
        if !response.status().is_success() {
            return Err(anyhow!("Status check failed"));
        }

        // Parse and categorize the response
        let status_response: CrawlStatusResponse = response.json().await?;

        match status_response.status.as_str() {
            "completed" => Ok(CrawlState::Completed {
                job_id: job_id.to_string(),
                data: status_response.data.unwrap_or_default(),
            }),
            "failed" => Ok(CrawlState::Failed {
                job_id: job_id.to_string(),
                error: status_response
                    .error
                    .unwrap_or_else(|| "Unknown error".to_string()),
            }),
            _ => Ok(CrawlState::InProgress {
                job_id: job_id.to_string(),
                status: status_response.status,
                completed: status_response.completed.unwrap_or(0),
                total: status_response.total.unwrap_or(0),
            }),
        }
    }

    // Alias method for compatibility with existing code
    pub async fn scrape_url(&self, url: &str) -> Result<ScrapeData> {
        self.scrape(url).await
    }

    // Alias method for compatibility with existing code
    pub async fn crawl_url(&self, request: CrawlRequest) -> Result<CrawlStartResponse> {
        // Start the crawl job
        let response = self
            .add_auth_headers(
                self.client
                    .post(format!("{}/crawl", self.base_url))
                    .json(&request),
            )
            .send()
            .await?;

        // Handle error responses
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Crawl start failed: {} - {}", status, error_text));
        }

        // Extract job ID from the response
        let start_response: CrawlStartResponse = response.json().await?;
        Ok(start_response)
    }
}

// Implement CrawlMonitorService for FirecrawlClient
impl CrawlMonitorService for FirecrawlClient {
    fn monitor_crawl_job<'a>(
        &'a self,
        job_id: &'a str,
        mut progress_callback: Box<dyn FnMut(crate::services::CrawlProgress) + Send + 'a>,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = crate::errors::FirecrawlResult<Vec<CrawlResponse>>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(async move {
            let mut results = Vec::new();

            loop {
                let state = self.check_crawl_status(job_id).await.map_err(|e| {
                    crate::errors::FirecrawlError::ApiError(crate::errors::ApiError::Other(e))
                })?;

                match state {
                    CrawlState::Completed { data, .. } => {
                        // Convert ScrapeData to CrawlResponse
                        for (index, scrape_data) in data.into_iter().enumerate() {
                            let response = CrawlResponse {
                                id: format!("crawl-result-{}", index),
                                url: scrape_data.url.unwrap_or_else(|| "unknown".to_string()),
                                status: "completed".to_string(),
                                completed_at: Some(chrono::Utc::now()),
                                markdown: scrape_data.markdown,
                                html: scrape_data.html,
                                metadata: crate::api::models::crawl_model::CrawlMetadata {
                                    title: scrape_data.metadata.title.clone(),
                                    language: scrape_data.metadata.language,
                                    keywords: None, // This would need to be populated from extra metadata
                                    robots: None,
                                    og_image: None,
                                    page_title: scrape_data.metadata.title,
                                    author: None,
                                    published_date: None,
                                    modified_date: None,
                                    site_name: None,
                                },
                            };
                            results.push(response);
                        }
                        break Ok(results);
                    }
                    CrawlState::Failed { error, .. } => {
                        break Err(crate::errors::FirecrawlError::ApiError(
                            crate::errors::ApiError::Other(anyhow!("Crawl failed: {}", error)),
                        ));
                    }
                    CrawlState::InProgress {
                        completed, total, ..
                    } => {
                        let progress = crate::services::CrawlProgress {
                            completed,
                            total,
                            current_url: None,
                            status: "in_progress".to_string(),
                        };
                        progress_callback(progress);
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    }
                    CrawlState::Started { .. } => {
                        let progress = crate::services::CrawlProgress {
                            completed: 0,
                            total: 0,
                            current_url: None,
                            status: "started".to_string(),
                        };
                        progress_callback(progress);
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    }
                }
            }
        })
    }
}
