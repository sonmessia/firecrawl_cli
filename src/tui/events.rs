use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEventKind};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub enum Event {
    Key(crossterm::event::KeyEvent),
    Tick,
}

pub struct EventHandler {
    sender: mpsc::Sender<Event>,
    receiver: mpsc::Receiver<Event>,
    stop_flag: Arc<Mutex<bool>>,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let (sender, receiver) = mpsc::channel();
        let stop_flag = Arc::new(Mutex::new(false));
        let stop_flag_clone = Arc::clone(&stop_flag);
        let sender_clone = sender.clone();
        
        thread::spawn(move || {
            let mut last_tick = std::time::Instant::now();
            loop {
                // Check if we should stop
                if *stop_flag_clone.lock().unwrap() {
                    break;
                }
                
                let timeout = Duration::from_millis(tick_rate)
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_millis(1));

                if event::poll(timeout).expect("no events available") {
                    match event::read().expect("unable to read event") {
                        CrosstermEvent::Key(key) if key.kind == KeyEventKind::Press => {
                            if sender_clone.send(Event::Key(key)).is_err() {
                                break;
                            }
                        }
                        CrosstermEvent::Resize(_, _) => {}
                        _ => {}
                    }
                }

                if last_tick.elapsed() >= Duration::from_millis(tick_rate) {
                    if sender_clone.send(Event::Tick).is_err() {
                        break;
                    }
                    last_tick = std::time::Instant::now();
                }
            }
        });

        Self {
            sender,
            receiver,
            stop_flag,
        }
    }

    pub fn next(&self) -> Result<Event> {
        Ok(self.receiver.recv()?)
    }

    pub fn send(&self, event: Event) -> Result<()> {
        self.sender.send(event)?;
        Ok(())
    }
}

impl Drop for EventHandler {
    fn drop(&mut self) {
        *self.stop_flag.lock().unwrap() = true;
        // Signal the thread to stop by sending a dummy event
        let _ = self.sender.send(Event::Tick);
    }
}
