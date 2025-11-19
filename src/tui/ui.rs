use crate::tui::app::{App, Mode, Operation, TaskStatus};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{
        Block, Borders, List, ListItem, ListState, Paragraph, Wrap,
    },
    Frame,
};

pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    render_title_bar(f, chunks[0]);
    render_main_content(f, app, chunks[1]);
    render_status_bar(f, app, chunks[2]);
}

fn render_title_bar(f: &mut Frame, area: Rect) {
    let title = Paragraph::new("Firecrawl CLI - Terminal UI")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    f.render_widget(title, area);
}

fn render_main_content(f: &mut Frame, app: &mut App, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    render_task_list(f, app, main_chunks[0]);
    render_task_details(f, app, main_chunks[1]);
}

fn render_task_list(f: &mut Frame, app: &mut App, area: Rect) {
    let tasks: Vec<ListItem> = app
        .tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let (color, symbol) = match task.status {
                TaskStatus::Pending => (Color::Yellow, "⏸"),
                TaskStatus::Processing => (Color::Blue, "⏳"),
                TaskStatus::Completed => (Color::Green, "✓"),
                TaskStatus::Failed(_) => (Color::Red, "✗"),
            };

            let operation_symbol = match task.operation {
                Operation::Scrape => "🔥",
                Operation::Crawl => "🕷️",
            };

            let content = format!(
                "{} {} [{}] {}",
                symbol, operation_symbol, task.id, task.url
            );

            let style = if i == app.selected_task {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(color)
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let tasks_list = List::new(tasks)
        .block(Block::default().borders(Borders::ALL).title("Tasks"));
    
    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_task));
    
    f.render_stateful_widget(tasks_list, area, &mut list_state);
}

fn render_task_details(f: &mut Frame, app: &mut App, area: Rect) {
    let details = if let Some(task) = app.tasks.get(app.selected_task) {
        let status_text = match &task.status {
            TaskStatus::Pending => "Status: Pending",
            TaskStatus::Processing => "Status: Processing",
            TaskStatus::Completed => "Status: Completed",
            TaskStatus::Failed(msg) => &format!("Status: Failed - {}", msg),
        };

        let operation_text = match task.operation {
            Operation::Scrape => "Operation: Scrape",
            Operation::Crawl => "Operation: Crawl",
        };

        let mut content = vec![
            Line::from(format!("Task ID: {}", task.id)),
            Line::from(operation_text.to_string()),
            Line::from(format!("URL: {}", task.url)),
            Line::from(status_text.to_string()),
            Line::from(""),
        ];

        if let Some(result) = &task.result {
            content.push(Line::from("Result:"));
            match result {
                crate::tui::app::TaskResult::Scrape(scrape_result) => {
                    content.push(Line::from(format!("  URL: {}", scrape_result.url.as_deref().unwrap_or("N/A"))));
                    content.push(Line::from(format!("  HTML length: {} chars", 
                        scrape_result.html.as_ref().map_or(0, |h| h.len()))));
                }
                crate::tui::app::TaskResult::Crawl(crawl_results) => {
                    content.push(Line::from(format!("  Pages crawled: {}", crawl_results.len())));
                    for (i, result) in crawl_results.iter().take(5).enumerate() {
                        content.push(Line::from(format!("  {}. {}", i + 1, result.url.as_deref().unwrap_or("N/A"))));
                    }
                    if crawl_results.len() > 5 {
                        content.push(Line::from(format!("  ... and {} more", crawl_results.len() - 5)));
                    }
                }
            }
        }

        content
    } else {
        vec![Line::from("No task selected")]
    };

    let details_paragraph = Paragraph::new(details)
        .block(Block::default().borders(Borders::ALL).title("Task Details"))
        .wrap(Wrap { trim: true });

    f.render_widget(details_paragraph, area);
}

fn render_status_bar(f: &mut Frame, app: &mut App, area: Rect) {
    let status_text: String = match app.mode {
        Mode::Normal => {
            "Commands: [q]uit [a]dd scrape [c]rawl [p]rocess [↑↓]navigate".to_string()
        }
        Mode::Input => {
            format!("Enter URL: {} [Enter] to submit [Esc] to cancel", app.input)
        }
        Mode::Processing => "Processing task...".to_string(),
    };

    let status_paragraph = Paragraph::new(status_text)
        .style(Style::default().fg(Color::Black).bg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status_paragraph, area);
}
