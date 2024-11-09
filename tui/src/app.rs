use std::time::Duration;

use crossterm::event::{Event, EventStream, KeyCode};
use futures::StreamExt;
use ratatui::{widgets::Paragraph, DefaultTerminal};
use tokio::time::sleep;

pub struct App {
    term_stream: EventStream,
    is_running: bool,
}

impl App {
    pub fn new() -> Self {
        let term_stream = EventStream::new();
        Self {
            term_stream,
            is_running: true,
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> anyhow::Result<()> {
        // Initialize the main event loop:
        // - Render content to the terminal
        // - Listen for key events (e.g., Esc key to exit the loop)

        terminal.draw(|frame| {
            let area = frame.area();
            frame.render_widget("Hello world", area);
        })?;

        let event = self.term_stream.next().await.unwrap()?;

        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char('q') => {
                    self.is_running = false;
                }
                _ => {}
            }
        }

        Ok(())
    }
}
