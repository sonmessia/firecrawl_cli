use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OutputFormat {
    Markdown,
    Html,
    RawHtml,
    Screenshot,
    Summary,
    // Raw data is handled by rawHtml format
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ParserType {
    Pdf,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ProxyType {
    Basic,
    Stealth,
    Auto,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub country: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub languages: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WaitAction {
    pub milliseconds: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Action {
    Wait(WaitAction),
    // Có thể thêm các actions khác như Click, Scroll ở đây trong tương lai
    // Click(ClickAction),
}

// --- Struct Request chính ---

#[derive(Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScrapeRequest {
    pub url: String,

    // --- Các tham số chính ---
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub formats: Vec<OutputFormat>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub only_main_content: Option<bool>,

    // --- Cấu hình nội dung ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_tags: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_tags: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_base64_images: Option<bool>,

    // --- Cấu hình request ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_for: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mobile: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_ads: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_tls_verification: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<ProxyType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,

    // --- Cấu hình nâng cao ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parsers: Option<Vec<ParserType>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<Vec<Action>>,

    // --- Quản lý Cache & Dữ liệu ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_age: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub store_in_cache: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub zero_data_retention: Option<bool>,
}

// --- Response ---

// Self-hosted response format: {"success": true, "data": {...}}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
}

// Struct lỗi chi tiết hơn
#[derive(Deserialize, Debug)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScrapeData {
    pub url: Option<String>,
    pub markdown: Option<String>,
    pub html: Option<String>,
    pub raw_html: Option<String>,
    pub screenshot: Option<String>,
    pub summary: Option<String>,
    pub links: Option<Vec<String>>,
    pub actions: Option<Actions>,
    pub warning: Option<String>,
    pub change_tracking: Option<ChangeTracking>,
    pub branding: Option<HashMap<String, Value>>,

    // Metadata được định nghĩa rõ ràng
    #[serde(default)]
    pub metadata: Metadata,
}

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

        if let Some(links) = &self.links {
            writeln!(f, "  Links found: {}", links.len())?;
        }

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

        if let Some(warning) = &self.warning {
            writeln!(f, "  Warning: {}", warning)?;
        }

        if !self.metadata.extra.is_empty() {
            writeln!(f, "  Extra metadata fields: {}", self.metadata.extra.len())?;
        }

        Ok(())
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Actions {
    pub screenshots: Option<Vec<String>>,
    pub scrapes: Option<Vec<ScrapeResult>>,
    pub javascript_returns: Option<Vec<JavaScriptReturn>>,
    pub pdfs: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ScrapeResult {
    pub url: String,
    pub html: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct JavaScriptReturn {
    #[serde(rename = "type")]
    pub return_type: String,
    pub value: Value,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChangeTracking {
    pub previous_scrape_at: Option<String>,
    pub change_status: Option<String>,
    pub visibility: Option<String>,
    pub diff: Option<String>,
    pub json: Option<HashMap<String, Value>>,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub source_url: Option<String>,

    // Gom tất cả các trường không xác định khác vào đây
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}
