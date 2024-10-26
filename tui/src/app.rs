use crossterm::event::{Event, EventStream, KeyCode};
use futures::StreamExt;
use ratatui::DefaultTerminal;

pub struct App {
    term_stream: EventStream,
    is_running: bool,
}

impl App {
    pub fn new() -> Self {
        let term_stream = EventStream::new();
        Self {
            term_stream,
            is_running: false,
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> anyhow::Result<()> {
        // Initialize the main event loop:
        // - Render content to the terminal
        // - Listen for key events (e.g., Esc key to exit the loop)
        todo!();

        Ok(())
    }
}
