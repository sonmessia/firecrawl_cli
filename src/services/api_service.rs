use async_trait::async_trait;
use std::time::Duration;
use std::sync::Arc;

use crate::api::services::client::FirecrawlClient;
use super::CrawlProgress;
use crate::api::models::{
    scrape_model::{ScrapeRequest, ScrapeResponse, ScrapeOptions},
    crawl_model::{CrawlRequest, CrawlResponse, CrawlOptions},
};
use crate::config::{AppConfig, ApiConfig};
use crate::errors::{FirecrawlError, FirecrawlResult};

/// Trait for API operations abstraction
#[async_trait]
pub trait ApiService {
    /// Scrape a single URL
    async fn scrape_url(&self, request: ScrapeRequest) -> FirecrawlResult<ScrapeResponse>;

    /// Start crawling a URL
    async fn crawl_url(&self, request: CrawlRequest) -> FirecrawlResult<CrawlResponse>;

    /// Get API status
    async fn get_status(&self) -> FirecrawlResult<ApiStatus>;

    /// Check API key validity
    async fn validate_api_key(&self) -> FirecrawlResult<bool>;
}

/// Extension trait for crawl job monitoring
pub trait CrawlMonitorService {
    /// Monitor crawl progress
    fn monitor_crawl_job<'a>(
        &'a self,
        job_id: &'a str,
        progress_callback: Box<dyn FnMut(CrawlProgress) + Send + 'a>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = FirecrawlResult<Vec<CrawlResponse>>> + Send + 'a>>;
}

/// API status information
#[derive(Debug, Clone)]
pub struct ApiStatus {
    pub is_healthy: bool,
    pub rate_limit_remaining: Option<u32>,
    pub rate_limit_reset: Option<chrono::DateTime<chrono::Utc>>,
    pub response_time: Duration,
}

/// Default implementation of ApiService using FirecrawlClient
pub struct DefaultApiService {
    client: FirecrawlClient,
    config: ApiConfig,
}

impl DefaultApiService {
    /// Create a new DefaultApiService with the given configuration
    pub fn new(config: ApiConfig) -> FirecrawlResult<Self> {
        let client = FirecrawlClient::new(&config.base_url, config.api_key.as_deref())
            .map_err(|e| FirecrawlError::ConfigurationError(e.to_string()))?;

        Ok(Self { client, config })
    }

    /// Create from AppConfig
    pub fn from_app_config(app_config: &AppConfig) -> FirecrawlResult<Self> {
        Self::new(app_config.api.clone())
    }

    /// Create with environment variables
    pub fn from_env() -> FirecrawlResult<Self> {
        let config = crate::config::environment::load_from_env()?;
        Self::from_app_config(&config)
    }

    /// Get the underlying client (for advanced usage)
    pub fn client(&self) -> &FirecrawlClient {
        &self.client
    }
}

#[async_trait]
impl ApiService for DefaultApiService {
    async fn scrape_url(&self, request: ScrapeRequest) -> FirecrawlResult<ScrapeResponse> {
        let start_time = std::time::Instant::now();

        let scrape_data = self.client
            .scrape_url(&request.url)
            .await
            .map_err(|e| {
                // Add retry logic here if needed
                FirecrawlError::ApiError(crate::errors::ApiError::Other(e))
            })?;

        // Convert ScrapeData to ScrapeResponse
        let result = ScrapeResponse {
            success: true,
            data: Some(scrape_data),
            error: None,
        };

        // Log execution time if verbose logging is enabled
        log::debug!("Scrape operation completed in {:?}", start_time.elapsed());

        Ok(result)
    }

    async fn crawl_url(&self, request: CrawlRequest) -> FirecrawlResult<CrawlResponse> {
        let start_time = std::time::Instant::now();

        let start_response = self.client
            .crawl_url(request)
            .await
            .map_err(|e| FirecrawlError::ApiError(crate::errors::ApiError::Other(e)))?;

        // For now, we'll return a basic CrawlResponse indicating the crawl started
        // In a real implementation, you might want to monitor the crawl and return results
        let result = CrawlResponse {
            id: start_response.job_id.clone(),
            url: start_response.job_id, // This is a placeholder - would need proper URL tracking
            status: "started".to_string(),
            completed_at: None,
            markdown: None,
            html: None,
            metadata: crate::api::models::crawl_model::CrawlMetadata {
                title: None,
                description: None,
                language: None,
                keywords: None,
                robots: None,
                og_image: None,
                page_title: None,
                author: None,
                published_date: None,
                modified_date: None,
                site_name: None,
            },
        };

        log::debug!("Crawl operation started in {:?}", start_time.elapsed());

        Ok(result)
    }

  
    async fn get_status(&self) -> FirecrawlResult<ApiStatus> {
        let start_time = std::time::Instant::now();

        // This would be a real status check in production
        // For now, we'll simulate a basic health check
        let is_healthy = true;
        let response_time = start_time.elapsed();

        Ok(ApiStatus {
            is_healthy,
            rate_limit_remaining: None,
            rate_limit_reset: None,
            response_time,
        })
    }

    async fn validate_api_key(&self) -> FirecrawlResult<bool> {
        // In a real implementation, this would make a test API call
        // For now, we'll just check if we have an API key configured
        Ok(self.config.api_key.is_some())
    }
}

impl CrawlMonitorService for DefaultApiService {
    fn monitor_crawl_job<'a>(
        &'a self,
        job_id: &'a str,
        progress_callback: Box<dyn FnMut(CrawlProgress) + Send + 'a>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = FirecrawlResult<Vec<CrawlResponse>>> + Send + 'a>> {
        Box::pin(async move {
            let start_time = std::time::Instant::now();

            let results = self.client
                .monitor_crawl_job(job_id, progress_callback)
                .await?;

            log::debug!("Crawl job {} completed in {:?}", job_id, start_time.elapsed());

            Ok(results)
        })
    }
}

/// Factory for creating API services
pub struct ApiServiceFactory;

impl ApiServiceFactory {
    /// Create API service from configuration
    pub fn create_from_config(config: &AppConfig) -> FirecrawlResult<Arc<dyn ApiService + Send + Sync>> {
        let service = DefaultApiService::from_app_config(config)?;
        Ok(Arc::new(service))
    }

    /// Create API service from environment variables
    pub fn create_from_env() -> FirecrawlResult<Arc<dyn ApiService + Send + Sync>> {
        let service = DefaultApiService::from_env()?;
        Ok(Arc::new(service))
    }

    /// Create API service with custom client (for testing)
    pub fn create_with_client(
        client: FirecrawlClient,
        config: ApiConfig,
    ) -> Arc<dyn ApiService + Send + Sync> {
        let service = DefaultApiService { client, config };
        Arc::new(service)
    }

    /// Create mock API service (for testing)
    #[cfg(test)]
    pub fn create_mock() -> Arc<dyn ApiService + Send + Sync> {
        Arc::new(MockApiService)
    }
}

/// Mock API service for testing
#[cfg(test)]
pub struct MockApiService;

#[cfg(test)]
impl CrawlMonitorService for MockApiService {
    fn monitor_crawl_job<'a>(
        &'a self,
        _job_id: &'a str,
        mut _progress_callback: Box<dyn FnMut(CrawlProgress) + Send + 'a>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = FirecrawlResult<Vec<CrawlResponse>>> + Send + 'a>> {
        Box::pin(async move {
            Ok(vec![])
        })
    }
}

#[cfg(test)]
#[async_trait]
impl ApiService for MockApiService {
    async fn scrape_url(&self, _request: ScrapeRequest) -> FirecrawlResult<ScrapeResponse> {
        Ok(ScrapeResponse {
            success: true,
            data: None,
            error: None,
        })
    }

    async fn crawl_url(&self, _request: CrawlRequest) -> FirecrawlResult<CrawlResponse> {
        Ok(CrawlResponse {
            id: "mock-crawl-id".to_string(),
            url: "https://example.com".to_string(),
            status: "completed".to_string(),
            completed_at: Some(chrono::Utc::now()),
            markdown: Some("# Mock Content".to_string()),
            html: Some("<h1>Mock Content</h1>".to_string()),
            metadata: crate::api::models::crawl_model::CrawlMetadata {
                title: Some("Mock Title".to_string()),
                description: Some("Mock Description".to_string()),
                language: Some("en".to_string()),
                keywords: Some(vec!["mock".to_string()]),
                robots: Some("all".to_string()),
                og_image: Some("https://example.com/image.jpg".to_string()),
                page_title: Some("Mock Page Title".to_string()),
                author: Some("Mock Author".to_string()),
                published_date: None,
                modified_date: None,
                site_name: None,
            },
        })
    }

    fn monitor_crawl_job<'a>(
        &'a self,
        _job_id: &'a str,
        mut _progress_callback: Box<dyn FnMut(CrawlProgress) + Send + 'a>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = FirecrawlResult<Vec<CrawlResponse>>> + Send + 'a>> {
        Box::pin(async move {
            Ok(vec![])
        })
    }

    async fn get_status(&self) -> FirecrawlResult<ApiStatus> {
        Ok(ApiStatus {
            is_healthy: true,
            rate_limit_remaining: Some(100),
            rate_limit_reset: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
            response_time: Duration::from_millis(100),
        })
    }

    async fn validate_api_key(&self) -> FirecrawlResult<bool> {
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_api_service_factory() {
        let service = ApiServiceFactory::create_mock();
        let status = service.get_status().await.unwrap();
        assert!(status.is_healthy);
    }

    #[tokio::test]
    async fn test_mock_api_service() {
        let service = MockApiService;

        let scrape_request = ScrapeRequest::builder()
            .url("https://example.com".to_string())
            .build()
            .unwrap();

        let result = service.scrape_url(scrape_request).await.unwrap();
        assert!(result.success);

        let crawl_request = CrawlRequest::builder()
            .url("https://example.com".to_string())
            .build()
            .unwrap();

        let result = service.crawl_url(crawl_request).await.unwrap();
        assert_eq!(result.status, "completed");
    }
}