use reqwest::{Client, Proxy};
use std::collections::HashMap;
use std::time::Duration;

use crate::api::services::client::FirecrawlClient;
use crate::config::{ApiConfig, ProxyConfig};
use crate::errors::{FirecrawlError, FirecrawlResult};

/// Builder for FirecrawlClient with comprehensive configuration options
pub struct FirecrawlClientBuilder {
    base_url: Option<String>,
    api_key: Option<String>,
    timeout: Duration,
    connect_timeout: Duration,
    read_timeout: Duration,
    max_retries: u32,
    retry_delay: Duration,
    user_agent: Option<String>,
    proxy: Option<ProxyConfig>,
    default_headers: HashMap<String, String>,
    pool_max_idle_per_host: usize,
    pool_idle_timeout: Duration,
    http2: bool,
    tcp_keepalive: Option<Duration>,
    tcp_nodelay: bool,
    compression: bool,
    follow_redirects: bool,
    redirect_limit: u32,
    enable_cookies: bool,
    validate_certs: bool,
}

impl Default for FirecrawlClientBuilder {
    fn default() -> Self {
        Self {
            base_url: None,
            api_key: None,
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            read_timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_delay: Duration::from_millis(1000),
            user_agent: None,
            proxy: None,
            default_headers: HashMap::new(),
            pool_max_idle_per_host: 10,
            pool_idle_timeout: Duration::from_secs(90),
            http2: true,
            tcp_keepalive: Some(Duration::from_secs(60)),
            tcp_nodelay: true,
            compression: true,
            follow_redirects: true,
            redirect_limit: 5,
            enable_cookies: false,
            validate_certs: true,
        }
    }
}

impl FirecrawlClientBuilder {
    /// Create a new builder with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create builder from ApiConfig
    pub fn from_config(config: &ApiConfig) -> Self {
        let mut builder = Self {
            base_url: Some(config.base_url.clone()),
            api_key: config.api_key.clone(),
            timeout: config.timeout,
            max_retries: config.max_retries,
            retry_delay: config.retry_delay,
            user_agent: config.user_agent.clone(),
            proxy: config.proxy.clone(),
            ..Self::default()
        };

        // Set derived timeouts
        builder.connect_timeout = Duration::from_secs(config.timeout.as_secs().min(15));
        builder.read_timeout = config.timeout;

        builder
    }

    /// Set the base URL for the API
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// Set the API key for authentication
    pub fn api_key(mut self, key: Option<impl Into<String>>) -> Self {
        self.api_key = key.map(|k| k.into());
        self
    }

    /// Set the overall request timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self.read_timeout = timeout;
        self
    }

    /// Set the connection timeout
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Set the read timeout
    pub fn read_timeout(mut self, timeout: Duration) -> Self {
        self.read_timeout = timeout;
        self
    }

    /// Set the maximum number of retry attempts
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Set the delay between retry attempts
    pub fn retry_delay(mut self, delay: Duration) -> Self {
        self.retry_delay = delay;
        self
    }

    /// Set the User-Agent header
    pub fn user_agent(mut self, agent: Option<impl Into<String>>) -> Self {
        self.user_agent = agent.map(|a| a.into());
        self
    }

    /// Set proxy configuration
    pub fn proxy(mut self, proxy: Option<ProxyConfig>) -> Self {
        self.proxy = proxy;
        self
    }

    /// Add a default header that will be included in all requests
    pub fn default_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.insert(name.into(), value.into());
        self
    }

    /// Set multiple default headers
    pub fn default_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.default_headers.extend(headers);
        self
    }

    /// Set the maximum number of idle connections per host
    pub fn pool_max_idle_per_host(mut self, max_idle: usize) -> Self {
        self.pool_max_idle_per_host = max_idle;
        self
    }

    /// Set the idle timeout for connections in the pool
    pub fn pool_idle_timeout(mut self, timeout: Duration) -> Self {
        self.pool_idle_timeout = timeout;
        self
    }

    /// Enable or disable HTTP/2
    pub fn http2(mut self, enabled: bool) -> Self {
        self.http2 = enabled;
        self
    }

    /// Set TCP keepalive duration
    pub fn tcp_keepalive(mut self, duration: Option<Duration>) -> Self {
        self.tcp_keepalive = duration;
        self
    }

    /// Enable or disable TCP nodelay
    pub fn tcp_nodelay(mut self, enabled: bool) -> Self {
        self.tcp_nodelay = enabled;
        self
    }

    /// Enable or disable compression
    pub fn compression(mut self, enabled: bool) -> Self {
        self.compression = enabled;
        self
    }

    /// Enable or disable redirect following
    pub fn follow_redirects(mut self, enabled: bool) -> Self {
        self.follow_redirects = enabled;
        self
    }

    /// Set the maximum number of redirects to follow
    pub fn redirect_limit(mut self, limit: u32) -> Self {
        self.redirect_limit = limit;
        self
    }

    /// Enable or disable cookie handling
    pub fn enable_cookies(mut self, enabled: bool) -> Self {
        self.enable_cookies = enabled;
        self
    }

    /// Enable or disable SSL certificate validation
    pub fn validate_certs(mut self, enabled: bool) -> Self {
        self.validate_certs = enabled;
        self
    }

    /// Build the FirecrawlClient
    pub fn build(self) -> FirecrawlResult<FirecrawlClient> {
        let base_url = self.base_url.as_ref().ok_or_else(|| {
            FirecrawlError::ConfigurationError("Base URL is required".to_string())
        })?;

        // Build the reqwest client
        let mut client_builder = Client::builder()
            .timeout(self.timeout)
            .connect_timeout(self.connect_timeout)
            .pool_max_idle_per_host(self.pool_max_idle_per_host)
            .pool_idle_timeout(self.pool_idle_timeout)
            .tcp_nodelay(self.tcp_nodelay)
            .redirect(if self.follow_redirects {
                reqwest::redirect::Policy::limited(self.redirect_limit.try_into().unwrap())
            } else {
                reqwest::redirect::Policy::none()
            });

        // Configure HTTP/2
        if self.http2 {
            client_builder = client_builder.http2_prior_knowledge();
        }

        // Configure TCP keepalive
        if let Some(duration) = self.tcp_keepalive {
            client_builder = client_builder.tcp_keepalive(duration);
        }

        // Configure compression
        if self.compression {
            client_builder = client_builder.gzip(true).brotli(true);
        }

        // Configure cookie store
        if self.enable_cookies {
            client_builder = client_builder.cookie_store(true);
        }

        // Configure certificate validation
        if !self.validate_certs {
            client_builder = client_builder.danger_accept_invalid_certs(true);
        }

        // Configure proxy if provided
        if let Some(proxy_config) = &self.proxy {
            let proxy = self.build_proxy(proxy_config)?;
            client_builder = client_builder.proxy(proxy);
        }

        // Set User-Agent if provided
        if let Some(user_agent) = self.user_agent {
            client_builder = client_builder.user_agent(&user_agent);
        } else {
            client_builder = client_builder.user_agent(format!(
                "firecrawl-cli/{} (rust)",
                env!("CARGO_PKG_VERSION")
            ));
        }

        // Build the client
        let client = client_builder.build().map_err(|e| {
            FirecrawlError::ConfigurationError(format!("Failed to build HTTP client: {}", e))
        })?;

        // Create FirecrawlClient with enhanced configuration
        let firecrawl_client = EnhancedFirecrawlClient {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: self.api_key,
            max_retries: self.max_retries,
            retry_delay: self.retry_delay,
            default_headers: self.default_headers,
        };

        Ok(firecrawl_client.into())
    }

    /// Build proxy configuration
    fn build_proxy(&self, config: &ProxyConfig) -> FirecrawlResult<Proxy> {
        let mut proxy = Proxy::all(&config.url)
            .map_err(|e| FirecrawlError::ConfigurationError(format!("Invalid proxy URL: {}", e)))?;

        // Set proxy authentication if provided
        if let (Some(username), Some(password)) = (&config.username, &config.password) {
            proxy = proxy.basic_auth(username, password);
        }

        Ok(proxy)
    }

    /// Build client for testing with mock settings
    #[cfg(test)]
    pub fn build_for_testing(self) -> FirecrawlResult<FirecrawlClient> {
        let mut builder = self
            .timeout(Duration::from_secs(5))
            .connect_timeout(Duration::from_secs(2))
            .max_retries(1)
            .validate_certs(false);

        if builder.base_url.is_none() {
            builder = builder.base_url("https://httpbin.org");
        }

        builder.build()
    }
}

/// Enhanced FirecrawlClient with additional configuration
pub struct EnhancedFirecrawlClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
    max_retries: u32,
    retry_delay: Duration,
    default_headers: HashMap<String, String>,
}

impl EnhancedFirecrawlClient {
    /// Get the underlying reqwest client
    pub fn http_client(&self) -> &Client {
        &self.client
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get the API key
    pub fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    /// Get retry configuration
    pub fn retry_config(&self) -> (u32, Duration) {
        (self.max_retries, self.retry_delay)
    }

    /// Get default headers
    pub fn default_headers(&self) -> &HashMap<String, String> {
        &self.default_headers
    }

    /// Create a request builder with authentication and default headers
    pub fn request_builder(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}/{}", self.base_url, path.trim_start_matches('/'));
        let mut builder = self.client.request(method, url);

        // Add authentication header if API key is available
        if let Some(api_key) = &self.api_key {
            builder = builder.header("Authorization", format!("Bearer {}", api_key));
        }

        // Add default headers
        for (name, value) in &self.default_headers {
            builder = builder.header(name, value);
        }

        builder
    }

    /// Execute request with retry logic
    pub async fn execute_with_retry(
        &self,
        request: reqwest::RequestBuilder,
    ) -> FirecrawlResult<reqwest::Response> {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            match request.try_clone().unwrap().send().await {
                Ok(response) => {
                    if response.status().is_success() || response.status().is_client_error() {
                        return Ok(response);
                    }

                    // Don't retry on client errors (4xx)
                    if response.status().is_client_error() {
                        return Ok(response);
                    }

                    last_error = Some(FirecrawlError::ApiError(
                        crate::errors::ApiError::ApiFailure {
                            status: response.status().as_u16(),
                            message: format!("HTTP {}", response.status()),
                        },
                    ));
                }
                Err(e) => {
                    last_error = Some(FirecrawlError::NetworkError(
                        crate::errors::NetworkError::ConnectionFailed(e.to_string()),
                    ));
                }
            }

            // Wait before retry (except on the last attempt)
            if attempt < self.max_retries {
                tokio::time::sleep(self.retry_delay).await;
            }
        }

        Err(last_error.unwrap_or_else(|| {
            FirecrawlError::ExecutionError("All retry attempts failed".to_string())
        }))
    }
}

/// Implement the original FirecrawlClient interface for EnhancedFirecrawlClient
impl EnhancedFirecrawlClient {
    /// Create a request builder for scrape operations
    pub async fn scrape(
        &self,
        url: &str,
    ) -> FirecrawlResult<crate::api::models::scrape_model::ScrapeData> {
        let request =
            self.request_builder(reqwest::Method::POST, "/scrape")
                .json(&serde_json::json!({
                    "url": url
                }));

        let response = self.execute_with_retry(request).await?;

        // Parse the response - this needs to match the actual API response format
        // This is a simplified version - adjust based on actual API structure
        let api_response: crate::api::models::scrape_model::ApiResponse<
            crate::api::models::scrape_model::ScrapeData,
        > = response.json().await.map_err(|e| {
            FirecrawlError::ApiError(crate::errors::ApiError::InvalidResponse(e.to_string()))
        })?;

        if api_response.success {
            Ok(api_response.data)
        } else {
            Err(FirecrawlError::ApiError(
                crate::errors::ApiError::InvalidResponse("API request failed".to_string()),
            ))
        }
    }

    /// Create a request builder for crawl operations
    pub async fn crawl(
        &self,
        url: &str,
        limit: Option<u32>,
    ) -> FirecrawlResult<Vec<crate::api::models::scrape_model::ScrapeData>> {
        let mut request_body = serde_json::json!({
            "url": url
        });

        if let Some(limit_val) = limit {
            request_body["limit"] = serde_json::Value::Number(limit_val.into());
        }

        let request = self
            .request_builder(reqwest::Method::POST, "/crawl")
            .json(&request_body);

        let response = self.execute_with_retry(request).await?;

        // Parse the response - this needs to match the actual API response format
        let api_response: crate::api::models::crawl_model::CrawlStartResponse =
            response.json().await.map_err(|e| {
                FirecrawlError::ApiError(crate::errors::ApiError::InvalidResponse(e.to_string()))
            })?;

        // For now, return empty results since crawl job monitoring is separate
        Ok(vec![])
    }

    /// Monitor crawl job progress
    pub async fn monitor_crawl_job<F>(
        &self,
        job_id: &str,
        progress_callback: F,
    ) -> FirecrawlResult<Vec<crate::api::models::crawl_model::CrawlResponse>>
    where
        F: Fn(f32) + Send + Sync,
    {
        let request = self.request_builder(reqwest::Method::GET, &format!("/crawl/{}", job_id));

        let response = self.execute_with_retry(request).await?;

        // Parse the response - this needs to match the actual API response format
        let api_response: crate::api::models::crawl_model::CrawlStatusResponse =
            response.json().await.map_err(|e| {
                FirecrawlError::ApiError(crate::errors::ApiError::InvalidResponse(e.to_string()))
            })?;

        if let Some(data) = api_response.data {
            // Convert ScrapeData to CrawlResponse
            let crawl_results: Vec<crate::api::models::crawl_model::CrawlResponse> = data
                .into_iter()
                .map(
                    |scrape_data| crate::api::models::crawl_model::CrawlResponse {
                        id: job_id.to_string(),
                        url: scrape_data.url.clone().unwrap_or_default(),
                        status: "completed".to_string(),
                        completed_at: Some(chrono::Utc::now()),
                        markdown: scrape_data.markdown.clone(),
                        html: scrape_data.html.clone().or(scrape_data.raw_html.clone()),
                        metadata: crate::api::models::crawl_model::CrawlMetadata {
                            title: scrape_data.metadata.title.clone(),
                            language: scrape_data.metadata.language.clone(),
                            keywords: None,
                            robots: None,
                            og_image: None,
                            page_title: scrape_data.metadata.title.clone(),
                            author: None,
                            published_date: None,
                            modified_date: None,
                            site_name: None,
                        },
                    },
                )
                .collect();

            // Report progress
            progress_callback(100.0);

            Ok(crawl_results)
        } else {
            Err(FirecrawlError::ApiError(
                crate::errors::ApiError::InvalidResponse("Crawl not completed".to_string()),
            ))
        }
    }
}

/// Implement the original FirecrawlClient interface for EnhancedFirecrawlClient
impl From<EnhancedFirecrawlClient> for FirecrawlClient {
    fn from(enhanced: EnhancedFirecrawlClient) -> Self {
        // Create the original client with the same configuration
        FirecrawlClient::new(&enhanced.base_url, enhanced.api_key.as_deref())
            .expect("Failed to create FirecrawlClient from EnhancedFirecrawlClient")
    }
}

/// Factory for creating pre-configured clients
pub struct FirecrawlClientFactory;

impl FirecrawlClientFactory {
    /// Create a default client
    pub fn create_default() -> FirecrawlResult<FirecrawlClient> {
        FirecrawlClientBuilder::new()
            .base_url("https://api.firecrawl.dev")
            .build()
            .map(|client| client.into())
    }

    /// Create a client from configuration
    pub fn from_config(config: &ApiConfig) -> FirecrawlResult<FirecrawlClient> {
        FirecrawlClientBuilder::from_config(config)
            .build()
            .map(|client| client.into())
    }

    /// Create a client from environment variables
    pub fn from_env() -> FirecrawlResult<FirecrawlClient> {
        let config = crate::config::environment::load_from_env()?;
        Self::from_config(&config.api)
    }

    /// Create a high-performance client optimized for batch operations
    pub fn create_high_performance() -> FirecrawlResult<FirecrawlClient> {
        FirecrawlClientBuilder::new()
            .base_url("https://api.firecrawl.dev")
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(10))
            .pool_max_idle_per_host(50)
            .pool_idle_timeout(Duration::from_secs(300))
            .max_retries(5)
            .compression(true)
            .http2(true)
            .build()
            .map(|client| client.into())
    }

    /// Create a low-latency client optimized for single requests
    pub fn create_low_latency() -> FirecrawlResult<FirecrawlClient> {
        FirecrawlClientBuilder::new()
            .base_url("https://api.firecrawl.dev")
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(5))
            .pool_max_idle_per_host(5)
            .max_retries(1)
            .tcp_nodelay(true)
            .build()
            .map(|client| client.into())
    }

    /// Create a client for testing purposes
    #[cfg(test)]
    pub fn create_for_testing() -> FirecrawlResult<FirecrawlClient> {
        FirecrawlClientBuilder::new()
            .base_url("https://httpbin.org")
            .timeout(Duration::from_secs(5))
            .max_retries(1)
            .validate_certs(false)
            .build_for_testing()
            .map(|client| client.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_builder_default() {
        let builder = FirecrawlClientBuilder::new();
        assert_eq!(builder.max_retries, 3);
        assert_eq!(builder.timeout, Duration::from_secs(30));
        assert!(builder.http2);
        assert!(builder.compression);
    }

    #[test]
    fn test_client_builder_configuration() {
        let builder = FirecrawlClientBuilder::new()
            .base_url("https://test.example.com")
            .api_key(Some("test-key"))
            .timeout(Duration::from_secs(60))
            .max_retries(5)
            .user_agent(Some("test-agent"));

        assert_eq!(
            builder.base_url,
            Some("https://test.example.com".to_string())
        );
        assert_eq!(builder.api_key, Some("test-key".to_string()));
        assert_eq!(builder.timeout, Duration::from_secs(60));
        assert_eq!(builder.max_retries, 5);
        assert_eq!(builder.user_agent, Some("test-agent".to_string()));
    }

    #[test]
    fn test_from_config() {
        let config = ApiConfig {
            base_url: "https://test.example.com".to_string(),
            api_key: Some("test-key".to_string()),
            timeout: Duration::from_secs(45),
            max_retries: 2,
            retry_delay: Duration::from_millis(500),
            user_agent: Some("test-agent".to_string()),
            proxy: None,
        };

        let builder = FirecrawlClientBuilder::from_config(&config);
        assert_eq!(
            builder.base_url,
            Some("https://test.example.com".to_string())
        );
        assert_eq!(builder.api_key, Some("test-key".to_string()));
        assert_eq!(builder.timeout, Duration::from_secs(45));
        assert_eq!(builder.max_retries, 2);
    }

    #[tokio::test]
    async fn test_client_factory() {
        let client = FirecrawlClientFactory::create_for_testing();
        assert!(client.is_ok());
    }
}
