use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::commands::CommandResult;
use crate::errors::{FirecrawlError, FirecrawlResult};

/// Trait for progress monitoring and notifications
#[async_trait]
pub trait ProgressService {
    /// Notify that a task has started
    async fn notify_task_started(&self, url: &str, task_type: &str);

    /// Notify task progress (0.0 to 1.0)
    async fn notify_task_progress(&self, url: &str, task_type: &str, progress: f32);

    /// Notify that a task has completed
    async fn notify_task_completed(&self, url: &str, task_type: &str);

    /// Notify that a task has failed
    async fn notify_task_failed(&self, url: &str, task_type: &str, error: &FirecrawlError);

    /// Get current statistics
    async fn get_statistics(&self) -> crate::services::task_service::TaskStatistics;

    /// Register a progress observer
    async fn register_observer(&self, observer: Arc<dyn ProgressObserver + Send + Sync>);

    /// Unregister a progress observer
    async fn unregister_observer(&self, observer_id: &str);
}

/// Trait for observing progress events
#[async_trait]
pub trait ProgressObserver {
    /// Called when a task starts
    async fn on_task_started(&self, url: &str, task_type: &str);

    /// Called when task progress updates
    async fn on_task_progress(&self, url: &str, task_type: &str, progress: f32);

    /// Called when a task completes
    async fn on_task_completed(&self, url: &str, task_type: &str);

    /// Called when a task fails
    async fn on_task_failed(&self, url: &str, task_type: &str, error: &FirecrawlError);

    /// Get a unique identifier for this observer
    fn observer_id(&self) -> &str;
}

/// Default implementation of ProgressService
pub struct DefaultProgressService {
    statistics: Arc<RwLock<crate::services::task_service::TaskStatistics>>,
    observers: Arc<RwLock<HashMap<String, Arc<dyn ProgressObserver + Send + Sync>>>>,
}

impl DefaultProgressService {
    /// Create a new DefaultProgressService
    pub fn new() -> Self {
        Self {
            statistics: Arc::new(RwLock::new(
                crate::services::task_service::TaskStatistics::default(),
            )),
            observers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create an Arc-wrapped instance
    pub fn new_arc() -> Arc<Self> {
        Arc::new(Self::new())
    }
}

impl Default for DefaultProgressService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProgressService for DefaultProgressService {
    async fn notify_task_started(&self, url: &str, task_type: &str) {
        // Update statistics
        {
            let mut stats = self.statistics.write().await;
            stats.total_tasks += 1;
            match task_type {
                "scrape" => stats.scrape_tasks += 1,
                "crawl" => stats.crawl_tasks += 1,
                _ => {}
            }
        }

        // Notify observers
        let observers = self.observers.read().await;
        for observer in observers.values() {
            let url = url.to_string();
            let task_type = task_type.to_string();
            let observer_clone = Arc::clone(observer);
            tokio::spawn(async move {
                observer_clone.on_task_started(&url, &task_type).await;
            });
        }
    }

    async fn notify_task_progress(&self, url: &str, task_type: &str, progress: f32) {
        // Notify observers
        let observers = self.observers.read().await;
        for observer in observers.values() {
            let url = url.to_string();
            let task_type = task_type.to_string();
            let observer_clone = Arc::clone(observer);
            tokio::spawn(async move {
                observer_clone
                    .on_task_progress(&url, &task_type, progress)
                    .await;
            });
        }
    }

    async fn notify_task_completed(&self, url: &str, task_type: &str) {
        // Update statistics
        {
            let mut stats = self.statistics.write().await;
            stats.completed_tasks += 1;
        }

        // Notify observers
        let observers = self.observers.read().await;
        for observer in observers.values() {
            let url = url.to_string();
            let task_type = task_type.to_string();
            let observer_clone = Arc::clone(observer);
            tokio::spawn(async move {
                observer_clone.on_task_completed(&url, &task_type).await;
            });
        }
    }

    async fn notify_task_failed(&self, url: &str, task_type: &str, error: &FirecrawlError) {
        // Update statistics
        {
            let mut stats = self.statistics.write().await;
            stats.failed_tasks += 1;
        }

        // Notify observers
        let observers = self.observers.read().await;
        for observer in observers.values() {
            let url = url.to_string();
            let task_type = task_type.to_string();
            let error_clone = error.clone();
            let observer_clone = Arc::clone(observer);
            tokio::spawn(async move {
                observer_clone
                    .on_task_failed(&url, &task_type, &error_clone)
                    .await;
            });
        }
    }

    async fn get_statistics(&self) -> crate::services::task_service::TaskStatistics {
        self.statistics.read().await.clone()
    }

    async fn register_observer(&self, observer: Arc<dyn ProgressObserver + Send + Sync>) {
        let mut observers = self.observers.write().await;
        observers.insert(observer.observer_id().to_string(), observer);
    }

    async fn unregister_observer(&self, observer_id: &str) {
        let mut observers = self.observers.write().await;
        observers.remove(observer_id);
    }
}

/// Console progress observer for CLI output
pub struct ConsoleProgressObserver {
    id: String,
}

impl ConsoleProgressObserver {
    pub fn new() -> Self {
        Self {
            id: format!(
                "console-{}",
                chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
            ),
        }
    }
}

impl Default for ConsoleProgressObserver {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProgressObserver for ConsoleProgressObserver {
    async fn on_task_started(&self, url: &str, task_type: &str) {
        println!("🚀 Started {} task for: {}", task_type, url);
    }

    async fn on_task_progress(&self, url: &str, _task_type: &str, progress: f32) {
        let percentage = (progress * 100.0) as u32;
        println!("⏳ {}: {}% complete", url, percentage);
    }

    async fn on_task_completed(&self, url: &str, task_type: &str) {
        println!("✅ Completed {} task for: {}", task_type, url);
    }

    async fn on_task_failed(&self, url: &str, task_type: &str, error: &FirecrawlError) {
        println!("❌ Failed {} task for: {} - {}", task_type, url, error);
    }

    fn observer_id(&self) -> &str {
        &self.id
    }
}

/// Logging progress observer
pub struct LoggingProgressObserver {
    id: String,
}

impl LoggingProgressObserver {
    pub fn new() -> Self {
        Self {
            id: format!(
                "logging-{}",
                chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
            ),
        }
    }
}

impl Default for LoggingProgressObserver {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProgressObserver for LoggingProgressObserver {
    async fn on_task_started(&self, url: &str, task_type: &str) {
        log::info!("Started {} task for URL: {}", task_type, url);
    }

    async fn on_task_progress(&self, url: &str, task_type: &str, progress: f32) {
        log::debug!(
            "Progress for {} {}: {:.1}%",
            task_type,
            url,
            progress * 100.0
        );
    }

    async fn on_task_completed(&self, url: &str, task_type: &str) {
        log::info!("Completed {} task for URL: {}", task_type, url);
    }

    async fn on_task_failed(&self, url: &str, task_type: &str, error: &FirecrawlError) {
        log::error!(
            "Failed {} task for URL: {} - Error: {}",
            task_type,
            url,
            error
        );
    }

    fn observer_id(&self) -> &str {
        &self.id
    }
}

/// Factory for creating progress services
pub struct ProgressServiceFactory;

impl ProgressServiceFactory {
    /// Create a default progress service with console observer
    pub fn create_console_service() -> Arc<dyn ProgressService + Send + Sync> {
        let service = DefaultProgressService::new_arc();
        let console_observer = Arc::new(ConsoleProgressObserver::new());

        // Register the observer (fire and forget since it's in the constructor)
        let service_clone = Arc::clone(&service);
        let _ = console_observer.observer_id().to_string();
        tokio::spawn(async move {
            service_clone.register_observer(console_observer).await;
        });

        service
    }

    /// Create a default progress service with logging observer
    pub fn create_logging_service() -> Arc<dyn ProgressService + Send + Sync> {
        let service = DefaultProgressService::new_arc();
        let logging_observer = Arc::new(LoggingProgressObserver::new());

        let service_clone = Arc::clone(&service);
        let _ = logging_observer.observer_id().to_string();
        tokio::spawn(async move {
            service_clone.register_observer(logging_observer).await;
        });

        service
    }

    /// Create a silent progress service (no observers)
    pub fn create_silent_service() -> Arc<dyn ProgressService + Send + Sync> {
        Arc::new(DefaultProgressService::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestProgressObserver {
        id: String,
        events: Arc<RwLock<Vec<String>>>,
    }

    impl TestProgressObserver {
        fn new() -> Self {
            Self {
                id: "test-observer".to_string(),
                events: Arc::new(RwLock::new(Vec::new())),
            }
        }

        async fn get_events(&self) -> Vec<String> {
            self.events.read().await.clone()
        }
    }

    #[async_trait]
    impl ProgressObserver for TestProgressObserver {
        async fn on_task_started(&self, url: &str, task_type: &str) {
            let mut events = self.events.write().await;
            events.push(format!("started:{}:{}", task_type, url));
        }

        async fn on_task_progress(&self, url: &str, task_type: &str, progress: f32) {
            let mut events = self.events.write().await;
            events.push(format!("progress:{}:{}:{}", task_type, url, progress));
        }

        async fn on_task_completed(&self, url: &str, task_type: &str) {
            let mut events = self.events.write().await;
            events.push(format!("completed:{}:{}", task_type, url));
        }

        async fn on_task_failed(&self, url: &str, task_type: &str, error: &FirecrawlError) {
            let mut events = self.events.write().await;
            events.push(format!("failed:{}:{}:{}", task_type, url, error));
        }

        fn observer_id(&self) -> &str {
            &self.id
        }
    }

    #[tokio::test]
    async fn test_progress_service() {
        let service = DefaultProgressService::new();
        let observer = Arc::new(TestProgressObserver::new());

        service.register_observer(Arc::clone(&observer)).await;

        service
            .notify_task_started("https://example.com", "scrape")
            .await;
        service
            .notify_task_progress("https://example.com", "scrape", 0.5)
            .await;
        service
            .notify_task_completed("https://example.com", "scrape")
            .await;

        // Give time for async tasks to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let events = observer.get_events().await;
        assert_eq!(events.len(), 3);
        assert_eq!(events[0], "started:scrape:https://example.com");
        assert_eq!(events[1], "progress:scrape:https://example.com:0.5");
        assert_eq!(events[2], "completed:scrape:https://example.com");
    }

    #[tokio::test]
    async fn test_statistics() {
        let service = DefaultProgressService::new();

        service
            .notify_task_started("https://example1.com", "scrape")
            .await;
        service
            .notify_task_started("https://example2.com", "crawl")
            .await;
        service
            .notify_task_completed("https://example1.com", "scrape")
            .await;
        service
            .notify_task_failed(
                "https://example2.com",
                "crawl",
                &FirecrawlError::ValidationError("test error".to_string()),
            )
            .await;

        let stats = service.get_statistics().await;
        assert_eq!(stats.total_tasks, 2);
        assert_eq!(stats.completed_tasks, 1);
        assert_eq!(stats.failed_tasks, 1);
        assert_eq!(stats.scrape_tasks, 1);
        assert_eq!(stats.crawl_tasks, 1);
        assert_eq!(stats.success_rate(), 50.0);
    }
}
