use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use crate::tui::{app::App, events::{Event, EventHandler}, ui};

pub async fn run_tui(mut app: App) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create event handler with 250ms tick rate
    let events = EventHandler::new(250);

    loop {
        // Render UI
        terminal.draw(|f| ui::ui(f, &mut app))?;

        // Handle events
        match events.next()? {
            Event::Key(key_event) => {
                if let KeyCode::Char('q') = key_event.code {
                    break;
                }

                match key_event.code {
                    KeyCode::Char('a') => {
                        app.mode = crate::tui::app::Mode::Input;
                    }
                    KeyCode::Char('c') => {
                        if app.mode == crate::tui::app::Mode::Input {
                            if !app.input.trim().is_empty() {
                                app.add_crawl_task(app.input.trim().to_string());
                                app.input.clear();
                            }
                        }
                        app.mode = crate::tui::app::Mode::Normal;
                    }
                    KeyCode::Char('p') => {
                        let client_clone = app.get_client().clone();
                        let selected_task = app.selected_task;
                        let mut app_clone = App::new(client_clone);
                        app_clone.selected_task = selected_task;
                        app_clone.tasks = app.tasks.clone();
                        
                        tokio::spawn(async move {
                            if let Err(e) = app_clone.process_next_task().await {
                                eprintln!("Error processing task: {}", e);
                            }
                        });
                    }
                    KeyCode::Up => {
                        app.select_previous_task();
                    }
                    KeyCode::Down => {
                        app.select_next_task();
                    }
                    KeyCode::Enter => {
                        if app.mode == crate::tui::app::Mode::Input && !app.input.trim().is_empty() {
                            app.add_scrape_task(app.input.trim().to_string());
                            app.input.clear();
                            app.mode = crate::tui::app::Mode::Normal;
                        }
                    }
                    KeyCode::Esc => {
                        app.mode = crate::tui::app::Mode::Normal;
                        app.input.clear();
                    }
                    KeyCode::Char(c) => {
                        if app.mode == crate::tui::app::Mode::Input {
                            app.input.push(c);
                        }
                    }
                    KeyCode::Backspace => {
                        if app.mode == crate::tui::app::Mode::Input {
                            app.input.pop();
                        }
                    }
                    _ => {}
                }
            }
            Event::Tick => {
                // Update any ongoing tasks or animations
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
