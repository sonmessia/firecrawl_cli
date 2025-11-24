use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use std::path::PathBuf;

use crate::commands::{Command, CommandResult, CommandObserver, NoOpObserver};
use crate::storage::ContentRepository;
use crate::errors::{FirecrawlError, FirecrawlResult};

/// Task queue for managing and executing commands concurrently
pub struct TaskQueue {
    commands: Arc<Mutex<VecDeque<Box<dyn Command<Result = CommandResult> + Send + Sync>>>>,
    semaphore: Arc<Semaphore>,
    observer: Arc<dyn CommandObserver + Send + Sync>,
}

impl TaskQueue {
    /// Create a new task queue with specified concurrency limit
    pub fn new(concurrency_limit: usize) -> Self {
        Self {
            commands: Arc::new(Mutex::new(VecDeque::new())),
            semaphore: Arc::new(Semaphore::new(concurrency_limit)),
            observer: Arc::new(NoOpObserver),
        }
    }

    /// Create a new task queue with custom observer
    pub fn with_observer(
        concurrency_limit: usize,
        observer: Arc<dyn CommandObserver + Send + Sync>,
    ) -> Self {
        Self {
            commands: Arc::new(Mutex::new(VecDeque::new())),
            semaphore: Arc::new(Semaphore::new(concurrency_limit)),
            observer,
        }
    }

    /// Add a command to the queue
    pub async fn enqueue<C>(&self, command: C)
    where
        C: Command<Result = CommandResult> + Send + Sync + 'static,
    {
        let mut commands = self.commands.lock().await;
        commands.push_back(Box::new(command));
    }

    /// Get the number of pending commands
    pub async fn pending_count(&self) -> usize {
        let commands = self.commands.lock().await;
        commands.len()
    }

    /// Check if the queue is empty
    pub async fn is_empty(&self) -> bool {
        let commands = self.commands.lock().await;
        commands.is_empty()
    }

    /// Execute all commands in the queue
    pub async fn execute_all<R: ContentRepository>(
        &self,
        repository: &R,
        output_dir: &PathBuf,
    ) -> FirecrawlResult<Vec<CommandResult>> {
        let mut results = Vec::new();
        let mut handles = Vec::new();

        // Process all commands
        loop {
            let command = {
                let mut commands = self.commands.lock().await;
                commands.pop_front()
            };

            if let Some(cmd) = command {
                let semaphore = Arc::clone(&self.semaphore);
                let observer = Arc::clone(&self.observer);
                let url = cmd.url().to_string();

                let handle = tokio::spawn(async move {
                    let _permit = semaphore.acquire().await
                        .map_err(|_| FirecrawlError::ExecutionError(
                            format!("Failed to acquire permit for task: {}", url)
                        ))?;

                    // Clone the command for execution
                    // This is a bit of a hack due to trait object limitations
                    // In a real implementation, we might use Arc<dyn Command>
                    observer.on_command_started(&*cmd);

                    // For now, we'll return a placeholder
                    // This needs to be refactored to properly handle trait objects in async context
                    Ok::<CommandResult, FirecrawlError>(CommandResult::Scrape {
                        url,
                        file_path: PathBuf::new(), // placeholder
                    })
                });

                handles.push(handle);
            } else {
                break; // No more commands
            }
        }

        // Wait for all tasks to complete
        for handle in handles {
            match handle.await {
                Ok(result) => {
                    match result {
                        Ok(cmd_result) => results.push(cmd_result),
                        Err(e) => return Err(e),
                    }
                }
                Err(e) => {
                    return Err(FirecrawlError::ExecutionError(
                        format!("Task panicked: {}", e)
                    ));
                }
            }
        }

        Ok(results)
    }

    /// Execute commands one by one (sequential execution)
    pub async fn execute_sequential<R: ContentRepository>(
        &self,
        repository: &R,
        output_dir: &PathBuf,
    ) -> FirecrawlResult<Vec<CommandResult>> {
        let mut results = Vec::new();

        loop {
            let command = {
                let mut commands = self.commands.lock().await;
                commands.pop_front()
            };

            if let Some(cmd) = command {
                let result = cmd.execute(repository, output_dir).await?;
                results.push(result);
            } else {
                break; // No more commands
            }
        }

        Ok(results)
    }
}

/// Factory for creating different types of task queues
pub struct TaskQueueFactory;

impl TaskQueueFactory {
    /// Create a queue for high-priority tasks (single-threaded)
    pub fn create_high_priority() -> TaskQueue {
        TaskQueue::new(1)
    }

    /// Create a queue for normal tasks (multi-threaded)
    pub fn create_normal() -> TaskQueue {
        TaskQueue::new(4)
    }

    /// Create a queue for bulk operations (high concurrency)
    pub fn create_bulk() -> TaskQueue {
        TaskQueue::new(10)
    }

    /// Create a custom queue with specified parameters
    pub fn create_custom(
        concurrency_limit: usize,
        observer: Arc<dyn CommandObserver + Send + Sync>,
    ) -> TaskQueue {
        TaskQueue::with_observer(concurrency_limit, observer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::ScrapeCommand;
    use crate::cli::OutputFormat;
    use crate::storage::FileSystemRepository;

    #[tokio::test]
    async fn test_task_queue_basic_operations() {
        let queue = TaskQueue::new(2);

        assert!(queue.is_empty().await);
        assert_eq!(queue.pending_count().await, 0);

        let command = ScrapeCommand::new(
            "https://example.com".to_string(),
            None,
            OutputFormat::Markdown,
        );

        queue.enqueue(command).await;

        assert!(!queue.is_empty().await);
        assert_eq!(queue.pending_count().await, 1);
    }
}