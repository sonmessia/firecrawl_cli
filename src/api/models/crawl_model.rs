use super::scrape_model::{OutputFormat, ScrapeData};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct CrawlRequest {
    pub url: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,

    // Cho phép tùy chỉnh toàn bộ quá trình scrape cho mỗi trang crawl được
    #[serde(flatten)]
    pub crawl_options: CrawlOptions,
}

#[derive(Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct CrawlOptions {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub formats: Vec<OutputFormat>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub only_main_content: Option<bool>,
    // Thêm các tùy chọn khác nếu cần
    // pub include_tags: Option<Vec<String>>,
    // pub exclude_tags: Option<Vec<String>>,
}

// Model để nhận Job ID khi bắt đầu crawl
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CrawlStartResponse {
    pub job_id: String,
}

// Enum để biểu diễn trạng thái của job một cách tường minh
#[derive(Debug, Clone)]
pub enum CrawlState {
    Started {
        job_id: String,
    },
    InProgress {
        job_id: String,
        status: String,
        completed: u32,
        total: u32,
    },
    Completed {
        job_id: String,
        data: Vec<ScrapeData>,
    },
    Failed {
        job_id: String,
        error: String,
    },
}

// Model để deserialize response khi kiểm tra trạng thái
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CrawlStatusResponse {
    pub status: String,
    pub completed: Option<u32>,
    pub total: Option<u32>,
    pub data: Option<Vec<ScrapeData>>,
    pub error: Option<String>,
}
