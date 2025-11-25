use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::OutputFormat;
use crate::commands::CommandResult;
use crate::config::{AppConfig, CacheConfig};
use crate::errors::{FirecrawlError, FirecrawlResult};

/// Trait for caching operations
#[async_trait]
pub trait CacheService {
    /// Store scrape result
    async fn store_scrape_result(
        &self,
        url: &str,
        format: &OutputFormat,
        result: &CommandResult,
    ) -> FirecrawlResult<()>;

    /// Retrieve scrape result
    async fn get_scrape_result(
        &self,
        url: &str,
        format: &OutputFormat,
    ) -> FirecrawlResult<Option<CommandResult>>;

    /// Store crawl result
    async fn store_crawl_result(
        &self,
        url: &str,
        format: &OutputFormat,
        result: &CommandResult,
    ) -> FirecrawlResult<()>;

    /// Retrieve crawl result
    async fn get_crawl_result(
        &self,
        url: &str,
        format: &OutputFormat,
    ) -> FirecrawlResult<Option<CommandResult>>;

    /// Check if a result exists in cache
    async fn exists(&self, url: &str, format: &OutputFormat) -> FirecrawlResult<bool>;

    /// Clear cache
    async fn clear(&self) -> FirecrawlResult<()>;

    /// Clean expired entries
    async fn clean_expired(&self) -> FirecrawlResult<usize>;

    /// Get cache statistics
    async fn get_statistics(&self) -> CacheStatistics;
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStatistics {
    pub total_entries: usize,
    pub scrape_entries: usize,
    pub crawl_entries: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub total_size_bytes: u64,
    pub hit_rate: f64,
}

impl CacheStatistics {
    /// Calculate hit rate as percentage
    pub fn calculate_hit_rate(&self) -> f64 {
        let total_requests = self.cache_hits + self.cache_misses;
        if total_requests == 0 {
            0.0
        } else {
            (self.cache_hits as f64 / total_requests as f64) * 100.0
        }
    }
}

/// In-memory cache service implementation
pub struct MemoryCacheService {
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    statistics: Arc<RwLock<CacheStatistics>>,
    config: CacheConfig,
}

/// Cache entry with metadata
#[derive(Debug, Clone)]
struct CacheEntry {
    data: CacheData,
    created_at: chrono::DateTime<chrono::Utc>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    access_count: u64,
    last_accessed: chrono::DateTime<chrono::Utc>,
}

/// Cached data types
#[derive(Debug, Clone)]
enum CacheData {
    ScrapeResult(CommandResult),
    CrawlResult(CommandResult),
}

impl MemoryCacheService {
    /// Create a new memory cache service
    pub fn new(config: CacheConfig) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            statistics: Arc::new(RwLock::new(CacheStatistics::default())),
            config,
        }
    }

    /// Create from AppConfig
    pub fn from_app_config(app_config: &AppConfig) -> Self {
        Self::new(app_config.execution.cache.clone())
    }

    /// Generate cache key
    fn generate_key(url: &str, format: &OutputFormat, data_type: &str) -> String {
        format!("{}:{}:{}", data_type, url, format)
    }

    /// Check if an entry has expired
    fn is_expired(&self, entry: &CacheEntry) -> bool {
        if let Some(expires_at) = entry.expires_at {
            chrono::Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Update access statistics for an entry
    async fn update_access(&self, entry: &mut CacheEntry) {
        entry.access_count += 1;
        entry.last_accessed = chrono::Utc::now();
    }

    /// Remove expired entries
    async fn remove_expired_entries(&self) -> usize {
        let mut cache = self.cache.write().await;
        let initial_size = cache.len();

        cache.retain(|_, entry| !self.is_expired(entry));

        initial_size - cache.len()
    }
}

#[async_trait]
impl CacheService for MemoryCacheService {
    async fn store_scrape_result(
        &self,
        url: &str,
        format: &OutputFormat,
        result: &CommandResult,
    ) -> FirecrawlResult<()> {
        let key = Self::generate_key(url, format, "scrape");
        let now = chrono::Utc::now();
        let expires_at = if self.config.ttl.as_secs() > 0 {
            Some(now + self.config.ttl)
        } else {
            None
        };

        let entry = CacheEntry {
            data: CacheData::ScrapeResult(result.clone()),
            created_at: now,
            expires_at,
            access_count: 1,
            last_accessed: now,
        };

        let mut cache = self.cache.write().await;
        cache.insert(key, entry);

        // Update statistics
        let mut stats = self.statistics.write().await;
        stats.total_entries = cache.len();
        stats.scrape_entries += 1;

        Ok(())
    }

    async fn get_scrape_result(
        &self,
        url: &str,
        format: &OutputFormat,
    ) -> FirecrawlResult<Option<CommandResult>> {
        let key = Self::generate_key(url, format, "scrape");

        {
            let mut cache = self.cache.write().await;
            if let Some(entry) = cache.get_mut(&key) {
                if self.is_expired(entry) {
                    cache.remove(&key);
                    let cache_len = cache.len();
                    let scrape_count = cache
                        .values()
                        .filter(|e| matches!(e.data, CacheData::ScrapeResult(_)))
                        .count();
                    drop(cache);

                    // Update statistics
                    let mut stats = self.statistics.write().await;
                    stats.cache_misses += 1;
                    stats.total_entries = cache_len;
                    stats.scrape_entries = scrape_count;

                    return Ok(None);
                }

                self.update_access(entry).await;
                let result = match &entry.data {
                    CacheData::ScrapeResult(result) => Some(result.clone()),
                    _ => None,
                };

                drop(cache);

                // Update statistics
                let mut stats = self.statistics.write().await;
                stats.cache_hits += 1;
                stats.hit_rate = stats.calculate_hit_rate();

                return Ok(result);
            }
        }

        // Update statistics for miss
        let mut stats = self.statistics.write().await;
        stats.cache_misses += 1;
        stats.hit_rate = stats.calculate_hit_rate();

        Ok(None)
    }

    async fn store_crawl_result(
        &self,
        url: &str,
        format: &OutputFormat,
        result: &CommandResult,
    ) -> FirecrawlResult<()> {
        let key = Self::generate_key(url, format, "crawl");
        let now = chrono::Utc::now();
        let expires_at = if self.config.ttl.as_secs() > 0 {
            Some(now + self.config.ttl)
        } else {
            None
        };

        let entry = CacheEntry {
            data: CacheData::CrawlResult(result.clone()),
            created_at: now,
            expires_at,
            access_count: 1,
            last_accessed: now,
        };

        let mut cache = self.cache.write().await;
        cache.insert(key, entry);

        // Update statistics
        let mut stats = self.statistics.write().await;
        stats.total_entries = cache.len();
        stats.crawl_entries += 1;

        Ok(())
    }

    async fn get_crawl_result(
        &self,
        url: &str,
        format: &OutputFormat,
    ) -> FirecrawlResult<Option<CommandResult>> {
        let key = Self::generate_key(url, format, "crawl");

        {
            let mut cache = self.cache.write().await;
            if let Some(entry) = cache.get_mut(&key) {
                if self.is_expired(entry) {
                    cache.remove(&key);
                    let cache_len = cache.len();
                    let crawl_count = cache
                        .values()
                        .filter(|e| matches!(e.data, CacheData::CrawlResult(_)))
                        .count();
                    drop(cache);

                    // Update statistics
                    let mut stats = self.statistics.write().await;
                    stats.cache_misses += 1;
                    stats.total_entries = cache_len;
                    stats.crawl_entries = crawl_count;

                    return Ok(None);
                }

                self.update_access(entry).await;
                let result = match &entry.data {
                    CacheData::CrawlResult(result) => Some(result.clone()),
                    _ => None,
                };

                drop(cache);

                // Update statistics
                let mut stats = self.statistics.write().await;
                stats.cache_hits += 1;
                stats.hit_rate = stats.calculate_hit_rate();

                return Ok(result);
            }
        }

        // Update statistics for miss
        let mut stats = self.statistics.write().await;
        stats.cache_misses += 1;
        stats.hit_rate = stats.calculate_hit_rate();

        Ok(None)
    }

    async fn exists(&self, url: &str, format: &OutputFormat) -> FirecrawlResult<bool> {
        let scrape_key = Self::generate_key(url, format, "scrape");
        let crawl_key = Self::generate_key(url, format, "crawl");

        let cache = self.cache.read().await;
        let scrape_exists = cache
            .get(&scrape_key)
            .map_or(false, |entry| !self.is_expired(entry));
        let crawl_exists = cache
            .get(&crawl_key)
            .map_or(false, |entry| !self.is_expired(entry));

        Ok(scrape_exists || crawl_exists)
    }

    async fn clear(&self) -> FirecrawlResult<()> {
        let mut cache = self.cache.write().await;
        cache.clear();

        // Reset statistics
        let mut stats = self.statistics.write().await;
        *stats = CacheStatistics::default();

        Ok(())
    }

    async fn clean_expired(&self) -> FirecrawlResult<usize> {
        let removed_count = self.remove_expired_entries().await;

        // Update statistics
        {
            let cache = self.cache.read().await;
            let mut stats = self.statistics.write().await;
            stats.total_entries = cache.len();
            stats.scrape_entries = cache
                .values()
                .filter(|e| matches!(e.data, CacheData::ScrapeResult(_)))
                .count();
            stats.crawl_entries = cache
                .values()
                .filter(|e| matches!(e.data, CacheData::CrawlResult(_)))
                .count();
        }

        Ok(removed_count)
    }

    async fn get_statistics(&self) -> CacheStatistics {
        let cache = self.cache.read().await;
        let stats = self.statistics.read().await;

        let mut result = stats.clone();
        result.total_entries = cache.len();
        result.scrape_entries = cache
            .values()
            .filter(|e| matches!(e.data, CacheData::ScrapeResult(_)))
            .count();
        result.crawl_entries = cache
            .values()
            .filter(|e| matches!(e.data, CacheData::CrawlResult(_)))
            .count();

        result
    }
}

/// No-op cache service (caching disabled)
pub struct NoOpCacheService;

#[async_trait]
impl CacheService for NoOpCacheService {
    async fn store_scrape_result(
        &self,
        _url: &str,
        _format: &OutputFormat,
        _result: &CommandResult,
    ) -> FirecrawlResult<()> {
        Ok(())
    }

    async fn get_scrape_result(
        &self,
        _url: &str,
        _format: &OutputFormat,
    ) -> FirecrawlResult<Option<CommandResult>> {
        Ok(None)
    }

    async fn store_crawl_result(
        &self,
        _url: &str,
        _format: &OutputFormat,
        _result: &CommandResult,
    ) -> FirecrawlResult<()> {
        Ok(())
    }

    async fn get_crawl_result(
        &self,
        _url: &str,
        _format: &OutputFormat,
    ) -> FirecrawlResult<Option<CommandResult>> {
        Ok(None)
    }

    async fn exists(&self, _url: &str, _format: &OutputFormat) -> FirecrawlResult<bool> {
        Ok(false)
    }

    async fn clear(&self) -> FirecrawlResult<()> {
        Ok(())
    }

    async fn clean_expired(&self) -> FirecrawlResult<usize> {
        Ok(0)
    }

    async fn get_statistics(&self) -> CacheStatistics {
        CacheStatistics::default()
    }
}

/// Factory for creating cache services
pub struct CacheServiceFactory;

impl CacheServiceFactory {
    /// Create cache service based on configuration
    pub fn create_from_config(config: &AppConfig) -> Arc<dyn CacheService + Send + Sync> {
        if config.execution.cache.enabled {
            Arc::new(MemoryCacheService::from_app_config(config))
        } else {
            Arc::new(NoOpCacheService)
        }
    }

    /// Create memory cache service
    pub fn create_memory_cache(cache_config: CacheConfig) -> Arc<dyn CacheService + Send + Sync> {
        Arc::new(MemoryCacheService::new(cache_config))
    }

    /// Create no-op cache service (caching disabled)
    pub fn create_no_op_cache() -> Arc<dyn CacheService + Send + Sync> {
        Arc::new(NoOpCacheService)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::CommandResult;

    fn create_test_scrape_result() -> CommandResult {
        CommandResult::Scrape {
            url: "https://example.com".to_string(),
            file_path: PathBuf::from("/test/example.md"),
        }
    }

    #[tokio::test]
    async fn test_memory_cache_basic_operations() {
        let cache_config = CacheConfig {
            enabled: true,
            directory: PathBuf::from("/tmp/cache"),
            ttl: std::time::Duration::from_secs(3600),
            max_size_mb: 100,
        };

        let cache = MemoryCacheService::new(cache_config);

        // Test storing and retrieving scrape result
        let result = create_test_scrape_result();
        cache
            .store_scrape_result("https://example.com", &OutputFormat::Markdown, &result)
            .await
            .unwrap();

        let retrieved = cache
            .get_scrape_result("https://example.com", &OutputFormat::Markdown)
            .await
            .unwrap();
        assert!(retrieved.is_some());

        // Test cache exists
        assert!(
            cache
                .exists("https://example.com", &OutputFormat::Markdown)
                .await
                .unwrap()
        );

        // Test cache miss
        let miss = cache
            .get_scrape_result("https://nonexistent.com", &OutputFormat::Markdown)
            .await
            .unwrap();
        assert!(miss.is_none());
    }

    #[tokio::test]
    async fn test_cache_statistics() {
        let cache_config = CacheConfig {
            enabled: true,
            directory: PathBuf::from("/tmp/cache"),
            ttl: std::time::Duration::from_secs(3600),
            max_size_mb: 100,
        };

        let cache = MemoryCacheService::new(cache_config);

        let result = create_test_scrape_result();

        // Store and retrieve to generate statistics
        cache
            .store_scrape_result("https://example.com", &OutputFormat::Markdown, &result)
            .await
            .unwrap();

        cache
            .get_scrape_result("https://example.com", &OutputFormat::Markdown)
            .await
            .unwrap();

        cache
            .get_scrape_result("https://nonexistent.com", &OutputFormat::Markdown)
            .await
            .unwrap();

        let stats = cache.get_statistics().await;
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.scrape_entries, 1);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.hit_rate, 50.0);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache_config = CacheConfig {
            enabled: true,
            directory: PathBuf::from("/tmp/cache"),
            ttl: std::time::Duration::from_millis(1), // Very short TTL
            max_size_mb: 100,
        };

        let cache = MemoryCacheService::new(cache_config);

        let result = create_test_scrape_result();
        cache
            .store_scrape_result("https://example.com", &OutputFormat::Markdown, &result)
            .await
            .unwrap();

        // Wait for expiration
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Should be expired now
        let retrieved = cache
            .get_scrape_result("https://example.com", &OutputFormat::Markdown)
            .await
            .unwrap();
        assert!(retrieved.is_none());
    }
}
