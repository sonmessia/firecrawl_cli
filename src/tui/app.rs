use crate::api::{FirecrawlClient, ScrapeData};
use anyhow::Result;
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    Normal,
    Input,
    Processing,
}

#[derive(Debug, Clone)]
pub enum Operation {
    Scrape,
    Crawl,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: usize,
    pub operation: Operation,
    pub url: String,
    pub status: TaskStatus,
    pub progress: String,
    pub result: Option<TaskResult>,
}

#[derive(Debug, Clone)]
pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone)]
pub enum TaskResult {
    Scrape(ScrapeData),
    Crawl(Vec<ScrapeData>),
}

pub struct App {
    pub mode: Mode,
    pub tasks: VecDeque<Task>,
    pub current_task_id: usize,
    pub input: String,
    pub selected_task: usize,
    pub scroll_offset: usize,
    pub client: FirecrawlClient,
}

impl App {
    pub fn new(client: FirecrawlClient) -> Self {
        Self {
            mode: Mode::Normal,
            tasks: VecDeque::new(),
            current_task_id: 0,
            input: String::new(),
            selected_task: 0,
            scroll_offset: 0,
            client,
        }
    }

    pub fn add_scrape_task(&mut self, url: String) {
        let task = Task {
            id: self.current_task_id,
            operation: Operation::Scrape,
            url: url.clone(),
            status: TaskStatus::Pending,
            progress: "Pending".to_string(),
            result: None,
        };
        self.tasks.push_back(task);
        self.current_task_id += 1;
    }

    pub fn add_crawl_task(&mut self, url: String) {
        let task = Task {
            id: self.current_task_id,
            operation: Operation::Crawl,
            url: url.clone(),
            status: TaskStatus::Pending,
            progress: "Pending".to_string(),
            result: None,
        };
        self.tasks.push_back(task);
        self.current_task_id += 1;
    }

    pub async fn process_next_task(&mut self) -> Result<()> {
        if let Some(task) = self.tasks.get_mut(self.selected_task) {
            if matches!(task.status, TaskStatus::Pending) {
                task.status = TaskStatus::Processing;
                task.progress = "Processing...".to_string();

                let result: Result<TaskResult, anyhow::Error> = match task.operation {
                    Operation::Scrape => {
                        let scrape_result = self.client.scrape(&task.url).await?;
                        Ok(TaskResult::Scrape(scrape_result))
                    }
                    Operation::Crawl => {
                        let crawl_result = self.client.crawl(&task.url, Some(10)).await?;
                        Ok(TaskResult::Crawl(crawl_result))
                    }
                };

                match result {
                    Ok(task_result) => {
                        task.status = TaskStatus::Completed;
                        task.progress = "Completed".to_string();
                        task.result = Some(task_result);
                    }
                    Err(e) => {
                        task.status = TaskStatus::Failed(e.to_string());
                        task.progress = "Failed".to_string();
                    }
                }
            }
        }
        Ok(())
    }

    pub fn select_next_task(&mut self) {
        if !self.tasks.is_empty() {
            self.selected_task = (self.selected_task + 1) % self.tasks.len();
        }
    }

    pub fn select_previous_task(&mut self) {
        if !self.tasks.is_empty() {
            self.selected_task = if self.selected_task == 0 {
                self.tasks.len() - 1
            } else {
                self.selected_task - 1
            };
        }
    }

    pub fn handle_input_char(&mut self, c: char) {
        match c {
            '\n' => {
                if !self.input.trim().is_empty() {
                    self.add_scrape_task(self.input.trim().to_string());
                    self.input.clear();
                    self.mode = Mode::Normal;
                }
            }
            '\x08' | '\x7f' => {
                self.input.pop();
            }
            _ => {
                self.input.push(c);
            }
        }
    }

    pub fn get_client(&self) -> &FirecrawlClient {
        &self.client
    }
}
