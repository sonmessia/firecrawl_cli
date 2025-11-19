pub mod models;
pub mod services;

// Re-export all types for easier access from other modules
pub use models::{crawl_model::*, scrape_model::*};
pub use services::client::*;
