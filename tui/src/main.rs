mod args;

use args::Args;
use common::{RoomEvent, ServerCommand, ServerEvent};
use crossterm::event::Event;
use futures::{SinkExt, StreamExt};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListDirection, ListItem, Widget};
use tokio::net::tcp::WriteHalf;
use tokio::net::TcpStream;
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};
use tui_textarea::{Input, Key, TextArea};

struct App {
    is_running: bool,
    messages: Vec<String>,
    current_room: String,
    username: String,
    textarea: TextArea<'static>,
}

impl App {
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(Style::default());
        textarea.set_placeholder_text("Start typing...");
        textarea.set_block(Block::default().borders(Borders::ALL).title("Send message"));
        Self {
            is_running: true,
            messages: Vec::new(),
            current_room: "lobby".to_owned(),
            username: "anonymous".to_owned(),
            textarea,
        }
    }

    pub async fn handle_terminal_event(
        &mut self,
        event: Event,
        tcp_writer: &mut FramedWrite<WriteHalf<'_>, LinesCodec>,
    ) -> anyhow::Result<()> {
        match event.into() {
            // Ctrl+C, Ctrl+D, Esc
            Input { key: Key::Esc, .. }
            | Input {
                key: Key::Char('c'),
                ctrl: true,
                ..
            }
            | Input {
                key: Key::Char('d'),
                ctrl: true,
                ..
            } => self.is_running = false,
            // Enter
            Input {
                key: Key::Enter, ..
            } => {
                if !self.textarea.is_empty() {
                    for line in self.textarea.clone().into_lines() {
                        tcp_writer.send(line).await?;
                    }
                    self.textarea.select_all();
                    self.textarea.delete_line_by_end();
                }
            }
            input => {
                self.textarea.input_without_shortcuts(input);
            }
        }
        Ok(())
    }

    pub async fn handle_tcp_event(&mut self, event: String) -> anyhow::Result<()> {
        self.messages.push(event.to_string());
        let event = ServerEvent::from_json_str(&event)?;
        // match event {}
        // if event.starts_with("You joined ") {
        //     let room_name = event.split_ascii_whitespace().nth(2).unwrap();
        //     self.current_room = room_name.to_owned();
        // } else if event.starts_with("You are ") {
        //     let username = event.split_ascii_whitespace().nth(2).unwrap();
        //     self.username = username.to_owned();
        // }
        match event {
            ServerEvent::Help(help) => {}
            ServerEvent::RoomEvent(username, RoomEvent::Message(message)) => {}
            ServerEvent::RoomEvent(username, RoomEvent::Joined(room)) => {}
            ServerEvent::RoomEvent(username, RoomEvent::Left(room)) => {}
            ServerEvent::RoomEvent(username, RoomEvent::NameChange(message)) => {}
            ServerEvent::Error(error) => {}
            ServerEvent::Rooms(rooms) => {}
            ServerEvent::Users(users) => {}
        }
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let layout =
            Layout::default().constraints([Constraint::Percentage(100), Constraint::Min(3)]);
        let chunks = layout.split(area);
        let title = format!("Room - {} [{}]", self.current_room, self.username);
        let list = List::new(
            self.messages
                .iter()
                .rev()
                .map(|msg| ListItem::new(msg.as_str())),
        )
        .block(Block::bordered().title(title))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol(">>")
        .repeat_highlight_symbol(true)
        .direction(ListDirection::BottomToTop);
        list.render(chunks[0], buf);
        self.textarea.render(chunks[1], buf);
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = Args::parse_socket_addr();
    let mut connection = TcpStream::connect(addr).await?;
    let (reader, writer) = connection.split();
    let mut tcp_writer = FramedWrite::new(writer, LinesCodec::new());
    let mut tcp_reader = FramedRead::new(reader, LinesCodec::new());

    tcp_writer
        .send(ServerCommand::Name("orhun".to_string()).to_string())
        .await?;

    let mut app = App::new();
    let mut terminal = ratatui::init();
    let mut term_stream = crossterm::event::EventStream::new();

    while app.is_running {
        terminal.draw(|f| {
            f.render_widget(&app, f.area());
        })?;
        tokio::select! {
            term_event = term_stream.next() => {
                if let Some(event) = term_event {
                    let event = event?;
                    app.handle_terminal_event(event,&mut tcp_writer).await?;
                }
            },
            tcp_event = tcp_reader.next() => {
                if let Some(tcp_event) = tcp_event {
                    app.handle_tcp_event(tcp_event?).await?;

                }
            },
        }
    }

    ratatui::restore();
    Ok(())
}
