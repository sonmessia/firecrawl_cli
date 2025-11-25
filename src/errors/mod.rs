use crate::storage::StorageError;
use thiserror::Error;

/// Domain-specific error types for the Firecrawl CLI
#[derive(Error, Debug, Clone)]
pub enum FirecrawlError {
    /// API-related errors
    #[error("API error: {0}")]
    ApiError(#[from] ApiError),

    /// Storage-related errors
    #[error("Storage error: {0}")]
    StorageError(#[from] StorageError),

    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Validation errors
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Execution errors
    #[error("Execution error: {0}")]
    ExecutionError(String),

    /// Network-related errors
    #[error("Network error: {0}")]
    NetworkError(#[from] NetworkError),

    /// CLI/TUI-related errors
    #[error("UI error: {0}")]
    UiError(String),

    /// Authentication errors
    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    /// Rate limiting errors
    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),

    /// Timeout errors
    #[error("Operation timed out: {0}")]
    TimeoutError(String),
}

/// API-specific error types
#[derive(Error, Debug, Clone)]
pub enum ApiError {
    #[error("HTTP request failed: {0}")]
    RequestError(String),

    #[error("Invalid response format: {0}")]
    InvalidResponse(String),

    #[error("API request failed with status {status}: {message}")]
    ApiFailure { status: u16, message: String },

    #[error("API rate limit exceeded")]
    RateLimitExceeded,

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("API endpoint not found: {0}")]
    EndpointNotFound(String),

    #[error("Invalid API key format")]
    InvalidApiKey,

    #[error("Request timeout: {0}")]
    Timeout(String),

    #[error("Other API error: {0}")]
    Other(String),
}

impl From<reqwest::Error> for ApiError {
    fn from(err: reqwest::Error) -> Self {
        ApiError::RequestError(err.to_string())
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::Other(err.to_string())
    }
}

/// Network-specific error types
#[derive(Error, Debug, Clone)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("DNS resolution failed: {0}")]
    DnsError(String),

    #[error("SSL/TLS error: {0}")]
    SslError(String),

    #[error("Request timeout: {0}")]
    Timeout(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Proxy error: {0}")]
    ProxyError(String),
}

/// Result type for Firecrawl operations
pub type FirecrawlResult<T> = Result<T, FirecrawlError>;

impl FirecrawlError {
    /// Get error code for programmatic handling
    pub fn error_code(&self) -> &'static str {
        match self {
            FirecrawlError::ApiError(_) => "API_ERROR",
            FirecrawlError::StorageError(_) => "STORAGE_ERROR",
            FirecrawlError::ConfigurationError(_) => "CONFIG_ERROR",
            FirecrawlError::ValidationError(_) => "VALIDATION_ERROR",
            FirecrawlError::ExecutionError(_) => "EXECUTION_ERROR",
            FirecrawlError::NetworkError(_) => "NETWORK_ERROR",
            FirecrawlError::UiError(_) => "UI_ERROR",
            FirecrawlError::AuthenticationError(_) => "AUTH_ERROR",
            FirecrawlError::RateLimitError(_) => "RATE_LIMIT_ERROR",
            FirecrawlError::TimeoutError(_) => "TIMEOUT_ERROR",
        }
    }

    /// Check if this is a retryable error
    pub fn is_retryable(&self) -> bool {
        match self {
            FirecrawlError::NetworkError(_) => true,
            FirecrawlError::ApiError(api_error) => match api_error {
                ApiError::RequestError(_) => true,
                ApiError::Timeout(_) => true,
                ApiError::RateLimitExceeded => true,
                _ => false,
            },
            FirecrawlError::TimeoutError(_) => true,
            FirecrawlError::StorageError(_) => false, // Usually not retryable
            _ => false,
        }
    }

    /// Get user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            FirecrawlError::ApiError(ApiError::AuthenticationFailed(_)) => {
                "Please check your API key. You can set it using the FIRECRAWL_API_KEY environment variable.".to_string()
            }
            FirecrawlError::ApiError(ApiError::RateLimitExceeded) => {
                "You've exceeded the rate limit. Please wait and try again later.".to_string()
            }
            FirecrawlError::ValidationError(msg) => {
                format!("Invalid input: {}", msg)
            }
            FirecrawlError::NetworkError(NetworkError::ConnectionFailed(_)) => {
                "Failed to connect to the server. Please check your internet connection.".to_string()
            }
            FirecrawlError::ConfigurationError(_) => {
                "There's a problem with your configuration. Please check your settings.".to_string()
            }
            _ => self.to_string(),
        }
    }
}

/// Error context for better debugging
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub operation: String,
    pub url: Option<String>,
    pub component: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub additional_info: std::collections::HashMap<String, String>,
}

impl ErrorContext {
    pub fn new(operation: &str, component: &str) -> Self {
        Self {
            operation: operation.to_string(),
            url: None,
            component: component.to_string(),
            timestamp: chrono::Utc::now(),
            additional_info: std::collections::HashMap::new(),
        }
    }

    pub fn with_url(mut self, url: &str) -> Self {
        self.url = Some(url.to_string());
        self
    }

    pub fn with_info(mut self, key: &str, value: &str) -> Self {
        self.additional_info
            .insert(key.to_string(), value.to_string());
        self
    }
}

/// Enhanced error with context
#[derive(Error, Debug)]
#[error("{error}")]
pub struct ContextualError {
    #[source]
    pub error: FirecrawlError,
    pub context: ErrorContext,
}

impl ContextualError {
    pub fn new(error: FirecrawlError, context: ErrorContext) -> Self {
        Self { error, context }
    }

    pub fn with_context(error: FirecrawlError, operation: &str, component: &str) -> Self {
        Self::new(error, ErrorContext::new(operation, component))
    }
}

