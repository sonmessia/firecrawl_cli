use async_trait::async_trait;
use std::path::PathBuf;

use super::errors::{FirecrawlError, FirecrawlResult};
use crate::cli::OutputFormat;
use crate::storage::ContentRepository;

pub mod crawl_command;
pub mod scrape_command;
pub mod task_queue;

pub use crawl_command::*;
pub use scrape_command::*;
pub use task_queue::*;

/// Command pattern trait for executable tasks
#[async_trait]
pub trait Command {
    type Result;

    /// Execute the command and return the result
    async fn execute(
        &self,
        repository: &dyn ContentRepository,
        output_dir: &PathBuf,
    ) -> FirecrawlResult<Self::Result>;

    /// Get a description of what this command does
    fn description(&self) -> String;

    /// Get the URL this command operates on
    fn url(&self) -> &str;

    /// Get the output format
    fn output_format(&self) -> OutputFormat;
}

/// Result type for command execution
#[derive(Debug, Clone)]
pub enum CommandResult {
    Scrape {
        url: String,
        file_path: PathBuf,
    },
    Crawl {
        url: String,
        file_paths: Vec<PathBuf>,
    },
}

/// Trait for command progress monitoring
pub trait CommandObserver {
    fn on_command_started(&self, command: &dyn Command<Result = CommandResult>);
    fn on_command_progress(&self, command: &dyn Command<Result = CommandResult>, progress: f32);
    fn on_command_completed(
        &self,
        command: &dyn Command<Result = CommandResult>,
        result: &CommandResult,
    );
    fn on_command_failed(
        &self,
        command: &dyn Command<Result = CommandResult>,
        error: &FirecrawlError,
    );
}

/// No-op observer implementation
pub struct NoOpObserver;

impl CommandObserver for NoOpObserver {
    fn on_command_started(&self, _command: &dyn Command<Result = CommandResult>) {}
    fn on_command_progress(&self, _command: &dyn Command<Result = CommandResult>, _progress: f32) {}
    fn on_command_completed(
        &self,
        _command: &dyn Command<Result = CommandResult>,
        _result: &CommandResult,
    ) {
    }
    fn on_command_failed(
        &self,
        _command: &dyn Command<Result = CommandResult>,
        _error: &FirecrawlError,
    ) {
    }
}

