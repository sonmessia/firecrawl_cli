use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

// Available output formats for scrape operations
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OutputFormat {
    Markdown,   // Markdown text content
    Html,       // Processed HTML content
    RawHtml,    // Raw HTML content as-is
    Screenshot, // Screenshot of the page
    Summary,    // AI-generated summary
    Links,      // List of links found on the page
    Images,     // Images extracted from the page
    Branding,   // Branding information
}

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
    pub code: String,    // Error code identifier
    pub message: String, // Human-readable error message
}

// Main scrape response data structure containing all extracted content
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScrapeData {
    // Basic information
    pub url: Option<String>, // The URL that was scraped

    // Content in various formats
    pub markdown: Option<String>,   // Markdown content
    pub html: Option<String>,       // Processed HTML content
    pub raw_html: Option<String>,   // Raw HTML content as returned
    pub screenshot: Option<String>, // Base64-encoded screenshot
    pub summary: Option<String>,    // AI-generated summary

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
        if let Some(title) = &self.metadata.title {
            writeln!(f, "  Title: {}", title)?;
        }
        if let Some(description) = &self.metadata.description {
            writeln!(f, "  Description: {}", description)?;
        }
        if let Some(language) = &self.metadata.language {
            writeln!(f, "  Language: {}", language)?;
        }

        // Show available content formats
        let content_types = [
            ("Markdown", self.markdown.is_some()),
            ("HTML", self.html.is_some()),
            ("Raw HTML", self.raw_html.is_some()),
            ("Screenshot", self.screenshot.is_some()),
            ("Summary", self.summary.is_some()),
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
            writeln!(f, "  Extra metadata fields: {}", self.metadata.extra.len())?;
        }

        Ok(())
    }
}

// Actions performed during the scraping process
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Actions {
    pub screenshots: Option<Vec<String>>, // List of screenshot paths/URLs
    pub scrapes: Option<Vec<ScrapeResult>>, // Results of sub-scrapes
    pub javascript_returns: Option<Vec<JavaScriptReturn>>, // JS execution results
    pub pdfs: Option<Vec<String>>,        // List of PDF paths/URLs
}

// Result of a sub-scrape operation
#[derive(Deserialize, Debug, Clone)]
pub struct ScrapeResult {
    pub url: String,          // URL that was sub-scraped
    pub html: Option<String>, // HTML content from sub-scrape
}

// Result of JavaScript execution
#[derive(Deserialize, Debug, Clone)]
pub struct JavaScriptReturn {
    #[serde(rename = "type")]
    pub return_type: String, // Type of returned value
    pub value: Value, // The actual returned value
}

// Change tracking information for comparing with previous scrapes
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChangeTracking {
    pub previous_scrape_at: Option<String>, // Timestamp of previous scrape
    pub change_status: Option<String>,      // Status of changes detected
    pub visibility: Option<String>,         // Visibility status
    pub diff: Option<String>,               // Text difference/diff
    pub json: Option<HashMap<String, Value>>, // JSON difference data
}

// Comprehensive metadata extracted from the scraped page
#[derive(Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub title: Option<String>,       // Page title
    pub description: Option<String>, // Meta description
    pub language: Option<String>,    // Content language
    pub source_url: Option<String>,  // Source URL if different from requested URL

    // Additional metadata fields are captured here
    #[serde(flatten)]
    pub extra: HashMap<String, Value>, // Extra/unknown metadata fields
}
