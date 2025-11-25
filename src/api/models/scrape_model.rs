use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

// Re-export the CLI OutputFormat to maintain consistency
pub use crate::cli::OutputFormat;

// Available parser types for special content handling
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ParserType {
    Pdf, // PDF document parser
}

// Proxy configuration options for requests
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ProxyType {
    Basic,   // Standard proxy
    Stealth, // Stealth/proxy that avoids detection
    Auto,    // Automatic proxy selection
}

// Geographic location configuration for requests
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub country: String, // Country code or name
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub languages: Vec<String>, // Preferred language codes
}

// Wait action configuration for controlling timing
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WaitAction {
    pub milliseconds: u64, // Duration to wait in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>, // CSS selector to wait for (optional)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClickAction {
    pub selector: String, // CSS selector to click
    #[serde(default)]
    pub all: bool, // Whether to click all matching elements
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PressAKeyAction {
    pub key: String, // Key to press
}

fn down() -> String {
    "down".to_string()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScrollAction {
    #[serde(default = "down")]
    pub direction: String, // Direction to scroll (e.g., "down", "up")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>, // CSS selector to scroll within (optional)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteJavaScriptAction {
    pub script: String, // JavaScript code to execute
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Format {
    A0,
    A1,
    A2,
    A3,
    A4,
    A5,
    A6,
    Letter,
    Legal,
    Tabloid,
    Ledger,
}

fn default_scale() -> f64 {
    1.0
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GeneratePdfAction {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<Format>, // PDF format (e.g., "A4", "Letter")
    #[serde(default)]
    pub landscape: bool, // Landscape orientation
    #[serde(default = "default_scale")]
    pub scale: f64, // Scale factor (e.g., 1.0 for 100%)
}

// Available actions to perform during scraping
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Action {
    Wait(WaitAction), // Wait for specified time or element
                      // Additional actions like Click, Scroll can be added here in future
                      // Click(ClickAction),
}

// Main scrape request structure containing all configuration options
#[derive(Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScrapeRequest {
    pub url: String, // Target URL to scrape

    // --- Core configuration ---
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub formats: Vec<OutputFormat>, // Requested output formats

    #[serde(skip_serializing_if = "Option::is_none")]
    pub only_main_content: Option<bool>, // Extract only main content (skip headers/footers)

    // --- Content filtering ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_tags: Option<Vec<String>>, // HTML tags to include in output

    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_tags: Option<Vec<String>>, // HTML tags to exclude from output

    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_base64_images: Option<bool>, // Remove base64-encoded images from content

    // --- Request configuration ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>, // Custom HTTP headers

    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>, // Request timeout in seconds

    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_for: Option<u64>, // Wait time after page load (ms)

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mobile: Option<bool>, // Use mobile user agent

    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_ads: Option<bool>, // Block advertisements

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_tls_verification: Option<bool>, // Skip SSL/TLS verification

    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<ProxyType>, // Proxy configuration

    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>, // Geographic location

    // --- Advanced configuration ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parsers: Option<Vec<ParserType>>, // Special content parsers

    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<Vec<Action>>, // Actions to perform during scraping

    // --- Cache and data management ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_age: Option<u64>, // Maximum age for cached results (seconds)

    #[serde(skip_serializing_if = "Option::is_none")]
    pub store_in_cache: Option<bool>, // Whether to store results in cache

    #[serde(skip_serializing_if = "Option::is_none")]
    pub zero_data_retention: Option<bool>, // Enable zero data retention mode
}

impl ScrapeRequest {
    pub fn builder() -> ScrapeRequestBuilder {
        ScrapeRequestBuilder::new()
    }
}

// Builder pattern for ScrapeRequest
pub struct ScrapeRequestBuilder {
    request: ScrapeRequest,
}

impl ScrapeRequestBuilder {
    pub fn new() -> Self {
        Self {
            request: ScrapeRequest::default(),
        }
    }

    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.request.url = url.into();
        self
    }

    pub fn formats(mut self, formats: Vec<OutputFormat>) -> Self {
        self.request.formats = formats;
        self
    }

    pub fn only_main_content(mut self, only_main_content: bool) -> Self {
        self.request.only_main_content = Some(only_main_content);
        self
    }

    pub fn include_tags(mut self, tags: Vec<String>) -> Self {
        self.request.include_tags = Some(tags);
        self
    }

    pub fn exclude_tags(mut self, tags: Vec<String>) -> Self {
        self.request.exclude_tags = Some(tags);
        self
    }

    pub fn remove_base64_images(mut self, remove: bool) -> Self {
        self.request.remove_base64_images = Some(remove);
        self
    }

    pub fn headers(mut self, headers: HashMap<String, String>) -> Self {
        self.request.headers = Some(headers);
        self
    }

    pub fn timeout(mut self, timeout: u64) -> Self {
        self.request.timeout = Some(timeout);
        self
    }

    pub fn wait_for(mut self, wait_for: u64) -> Self {
        self.request.wait_for = Some(wait_for);
        self
    }

    pub fn mobile(mut self, mobile: bool) -> Self {
        self.request.mobile = Some(mobile);
        self
    }

    pub fn block_ads(mut self, block: bool) -> Self {
        self.request.block_ads = Some(block);
        self
    }

    pub fn skip_tls_verification(mut self, skip: bool) -> Self {
        self.request.skip_tls_verification = Some(skip);
        self
    }

    pub fn proxy(mut self, proxy: ProxyType) -> Self {
        self.request.proxy = Some(proxy);
        self
    }

    pub fn location(mut self, location: Location) -> Self {
        self.request.location = Some(location);
        self
    }

    pub fn parsers(mut self, parsers: Vec<ParserType>) -> Self {
        self.request.parsers = Some(parsers);
        self
    }

    pub fn actions(mut self, actions: Vec<Action>) -> Self {
        self.request.actions = Some(actions);
        self
    }

    pub fn max_age(mut self, max_age: u64) -> Self {
        self.request.max_age = Some(max_age);
        self
    }

    pub fn store_in_cache(mut self, store: bool) -> Self {
        self.request.store_in_cache = Some(store);
        self
    }

    pub fn zero_data_retention(mut self, zero: bool) -> Self {
        self.request.zero_data_retention = Some(zero);
        self
    }

    pub fn build(self) -> ScrapeRequest {
        self.request
    }
}

// --- Response Structures ---

// Generic API response wrapper for self-hosted Firecrawl instances
// Format: {"success": true, "data": {...}}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    pub success: bool, // Whether the request was successful
    pub data: T,       // The response data payload
}

// Detailed error information structure
#[derive(Deserialize, Debug)]
pub struct ApiError {
    pub success: bool,   // Always false for errors
    pub code: String,    // Error code identifier
    pub message: String, // Human-readable error message
}

// Re-export the CLI ScrapeOptions to maintain consistency
pub use crate::cli::ScrapeOptions;

// Scrape response wrapper
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScrapeResponse {
    pub success: bool,
    pub data: Option<ScrapeData>,
    pub error: Option<String>,
}

// Main scrape response data structure containing all extracted content
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScrapeData {
    // Basic information
    pub url: Option<String>, // The URL that was scraped

    // Content in various formats
    pub markdown: Option<String>,    // Markdown content
    pub html: Option<String>,        // Processed HTML content
    pub raw_html: Option<String>,    // Raw HTML content as returned
    pub images: Option<Vec<String>>, // List of image URLs
    pub screenshot: Option<String>,  // Base64-encoded screenshot

    // Links and navigation
    pub links: Option<Vec<String>>, // List of found links

    // Actions and interactions
    pub actions: Option<Actions>, // Actions performed during scraping
    pub warning: Option<String>,  // Any warnings generated

    // Change tracking
    pub change_tracking: Option<ChangeTracking>, // Change tracking information
    pub branding: Option<HashMap<String, Value>>, // Branding information

    // Metadata extracted from the page
    #[serde(default)]
    pub metadata: Metadata, // Structured metadata
}

// Display implementation for ScrapeData to provide human-readable summary
impl fmt::Display for ScrapeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "📄 Scrape Data:")?;
        if let Some(url) = &self.url {
            writeln!(f, "  URL: {}", url)?;
        }

        // Show available content formats
        let content_types = [
            ("Markdown", self.markdown.is_some()),
            ("HTML", self.html.is_some()),
            ("Raw HTML", self.raw_html.is_some()),
            ("Screenshot", self.screenshot.is_some()),
            ("Links", self.links.is_some()),
            ("Images", self.images.is_some()),
        ];

        let available_formats: Vec<&str> = content_types
            .iter()
            .filter_map(|(name, available)| if *available { Some(*name) } else { None })
            .collect();

        if !available_formats.is_empty() {
            writeln!(f, "  Available formats: {}", available_formats.join(", "))?;
        }

        // Show link count
        if let Some(links) = &self.links {
            writeln!(f, "  Links found: {}", links.len())?;
        }

        // Show action summary
        if let Some(actions) = &self.actions {
            let action_count = [
                (
                    "screenshots",
                    actions.screenshots.as_ref().map_or(0, |s| s.len()),
                ),
                ("scrapes", actions.scrapes.as_ref().map_or(0, |s| s.len())),
                (
                    "javascriptReturns",
                    actions.javascript_returns.as_ref().map_or(0, |s| s.len()),
                ),
                ("pdfs", actions.pdfs.as_ref().map_or(0, |s| s.len())),
            ];
            let action_summary: Vec<String> = action_count
                .iter()
                .filter_map(|(name, count)| {
                    if *count > 0 {
                        Some(format!("{}: {}", name, count))
                    } else {
                        None
                    }
                })
                .collect();
            if !action_summary.is_empty() {
                writeln!(f, "  Actions: {}", action_summary.join(", "))?;
            }
        }

        // Show any warnings
        if let Some(warning) = &self.warning {
            writeln!(f, "  Warning: {}", warning)?;
        }

        // Show extra metadata count
        if !self.metadata.extra.is_empty() {
            writeln!(f, "Extra metadata fields: {}", self.metadata.extra.len())?;
        }

        Ok(())
    }
}

// Actions performed during the scraping process
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Actions {
    pub screenshots: Option<Vec<String>>, // List of screenshot paths/URLs
    pub scrapes: Option<Vec<ScrapeResult>>, // Results of sub-scrapes
    pub javascript_returns: Option<Vec<JavaScriptReturn>>, // JS execution results
    pub pdfs: Option<Vec<String>>,        // List of PDF paths/URLs
}

// Result of a sub-scrape operation
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ScrapeResult {
    pub url: String,          // URL that was sub-scraped
    pub html: Option<String>, // HTML content from sub-scrape
}

// Result of JavaScript execution
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct JavaScriptReturn {
    #[serde(rename = "type")]
    pub return_type: String, // Type of returned value
    pub value: Value, // The actual returned value
}

// Change tracking information for comparing with previous scrapes
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChangeTracking {
    pub previous_scrape_at: Option<String>, // Timestamp of previous scrape
    pub change_status: Option<String>,      // Status of changes detected
    pub visibility: Option<String>,         // Visibility status
    pub diff: Option<String>,               // Text difference/diff
    pub json: Option<HashMap<String, Value>>, // JSON difference data
}

// Comprehensive metadata extracted from the scraped page
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    // Additional metadata fields are captured here
    #[serde(flatten)]
    pub extra: HashMap<String, Value>, // Extra/unknown metadata fields
}
