use anyhow::{Result, anyhow};
use reqwest::Client;
use std::time::Duration;
use tokio::time::sleep;

use crate::api::{
    ApiResponse, CrawlRequest, CrawlStartResponse, CrawlState, CrawlStatusResponse, OutputFormat,
    ScrapeData, ScrapeRequest,
};

pub struct FirecrawlClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
}

impl FirecrawlClient {
    pub fn new(base_url: &str, api_key: Option<&str>) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(300))
            .build()?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.map(|k| k.to_string()),
        })
    }

    fn add_auth_headers(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(api_key) = &self.api_key {
            request.header("Authorization", format!("Bearer {}", api_key))
        } else {
            request
        }
    }

    pub async fn scrape(&self, url: &str) -> Result<ScrapeData> {
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

        let response = self
            .add_auth_headers(
                self.client
                    .post(format!("{}/scrape", self.base_url))
                    .json(&request),
            )
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Scrape request failed: {} - {}",
                status,
                error_text
            ));
        }

        let api_response: ApiResponse<ScrapeData> = response.json().await?;

        if api_response.success {
            Ok(api_response.data)
        } else {
            Err(anyhow!("API request failed"))
        }
    }

    pub async fn crawl(&self, url: &str, limit: Option<u32>) -> Result<Vec<ScrapeData>> {
        // Start crawl job
        let request = CrawlRequest {
            url: url.to_string(),
            limit,
            ..Default::default()
        };

        let response = self
            .add_auth_headers(
                self.client
                    .post(format!("{}/crawl", self.base_url))
                    .json(&request),
            )
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Crawl start failed: {} - {}", status, error_text));
        }

        let start_response: CrawlStartResponse = response.json().await?;
        let job_id = start_response.job_id;

        // Poll for completion
        loop {
            let state = self.check_crawl_status(&job_id).await?;

            match state {
                CrawlState::Completed { data, .. } => return Ok(data),
                CrawlState::Failed { error, .. } => return Err(anyhow!("Crawl failed: {}", error)),
                CrawlState::InProgress {
                    completed, total, ..
                } => {
                    println!("⏳ Progress: {}/{}", completed, total);
                }
                CrawlState::Started { .. } => {
                    println!("🚀 Crawl job started");
                }
            }

            sleep(Duration::from_secs(2)).await;
        }
    }

    async fn check_crawl_status(&self, job_id: &str) -> Result<CrawlState> {
        let response = self
            .add_auth_headers(
                self.client
                    .get(format!("{}/crawl/{}", self.base_url, job_id)),
            )
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Status check failed"));
        }

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
}
