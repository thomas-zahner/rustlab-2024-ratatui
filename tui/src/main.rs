mod args;

use args::Args;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use common::{RoomEvent, ServerCommand, ServerEvent};
use crossterm::event::Event as CrosstermEvent;
use futures::{SinkExt, StreamExt};
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, List, ListDirection, ListItem, ListState,
};
use ratatui::Frame;
use ratatui_explorer::{FileExplorer, Theme};
use ratatui_image::picker::{Picker, ProtocolType};
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::StatefulImage;
use tokio::net::tcp::WriteHalf;
use tokio::net::TcpStream;
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};
use tui_textarea::{Input, Key, TextArea};
use tui_tree_widget::{Tree, TreeItem, TreeState};

struct App {
    is_running: bool,
    messages: Vec<ServerEvent>,
    rooms: Vec<String>,
    users: Vec<String>,
    current_room: String,
    username: String,
    text_area: TextArea<'static>,
    list_state: ListState,
    file_explorer: FileExplorer,
    popup: Popup,
}

enum Popup {
    None,
    FileExplorer,
    ImagePreview(Box<dyn StatefulProtocol>),
}

impl PartialEq for Popup {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Popup::None, Popup::None) => true,
            (Popup::FileExplorer, Popup::FileExplorer) => true,
            (Popup::ImagePreview(_), Popup::ImagePreview(_)) => true,
            _ => false,
        }
    }
}

#[derive(Clone)]
enum Event {
    Terminal(CrosstermEvent),
    FileSelected,
}

fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(Style::default());
        textarea.set_placeholder_text("Start typing...");
        textarea.set_block(Block::default().borders(Borders::ALL).title("Send message"));
        let theme = Theme::default()
            .add_default_title()
            .with_title_bottom(|fe| format!("[{} files]", fe.files().len()).into())
            .with_block(Block::bordered().border_type(BorderType::Rounded));
        let file_explorer = FileExplorer::with_theme(theme)?;
        Ok(Self {
            is_running: true,
            messages: Vec::new(),
            rooms: Vec::new(),
            users: Vec::new(),
            current_room: "lobby".to_owned(),
            username: "anonymous".to_owned(),
            text_area: textarea,
            list_state: ListState::default(),
            file_explorer,
            popup: Popup::None,
        })
    }

    pub async fn handle_event(
        &mut self,
        event: Event,
        tcp_writer: &mut FramedWrite<WriteHalf<'_>, LinesCodec>,
        client_writer: &mut tokio::sync::mpsc::UnboundedSender<Event>,
    ) -> anyhow::Result<()> {
        match event {
            Event::Terminal(event) => {
                if self.popup == Popup::FileExplorer {
                    if let Input { key: Key::Esc, .. } = event.clone().into() {
                        self.popup = Popup::None;
                    } else if let Input {
                        key: Key::Enter, ..
                    } = event.clone().into()
                    {
                        self.popup = Popup::None;
                        client_writer.send(Event::FileSelected)?;
                    } else {
                        self.file_explorer.handle(&event)?;
                    }
                    return Ok(());
                } else if let Popup::ImagePreview(_) = self.popup {
                    if let Input { key: Key::Esc, .. } = event.clone().into() {
                        self.popup = Popup::None;
                    }
                    return Ok(());
                }
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
                        if !self.text_area.is_empty() {
                            for line in self.text_area.clone().into_lines() {
                                tcp_writer.send(line).await?;
                            }
                            self.text_area.select_all();
                            self.text_area.delete_line_by_end();
                        }
                    }
                    // Down
                    Input { key: Key::Down, .. } => {
                        self.list_state.select_previous();
                    }
                    // Up
                    Input { key: Key::Up, .. } => {
                        self.list_state.select_next();
                    }
                    // Show explorer
                    Input {
                        key: Key::Char('e'),
                        ctrl: true,
                        ..
                    } => {
                        self.popup = Popup::FileExplorer;
                    }
                    // Preview file
                    Input {
                        key: Key::Char('p'),
                        ctrl: true,
                        ..
                    } => {
                        let selected_event = self
                            .list_state
                            .selected()
                            .map(|v| self.messages[self.messages.len() - (v + 1)].clone());

                        if let Some(ServerEvent::RoomEvent(_, RoomEvent::File(_, contents))) =
                            selected_event
                        {
                            let data = BASE64_STANDARD.decode(contents.as_bytes())?;
                            let img = image::load_from_memory(&data)?;
                            let user_fontsize = (7, 14);
                            let user_protocol = ProtocolType::Halfblocks;
                            let mut picker = Picker::new(user_fontsize);
                            picker.protocol_type = user_protocol;
                            let image = picker.new_resize_protocol(img);
                            self.popup = Popup::ImagePreview(image);
                        }
                    }
                    input => {
                        self.text_area.input_without_shortcuts(input);
                    }
                }
            }
            Event::FileSelected => {
                let file = self.file_explorer.current();
                if file.is_dir() {
                    return Ok(());
                }
                let contents = tokio::fs::read(file.path()).await?;
                let base64 = BASE64_STANDARD.encode(contents);
                tcp_writer
                    .send(ServerCommand::File(file.name().to_string(), base64).to_string())
                    .await?;
            }
        }

        Ok(())
    }

    #[allow(unused_variables)]
    pub async fn handle_tcp_event(
        &mut self,
        event: String,
        tcp_writer: &mut FramedWrite<WriteHalf<'_>, LinesCodec>,
    ) -> anyhow::Result<()> {
        let event = ServerEvent::from_json_str(&event)?;
        self.messages.push(event.clone());
        match event {
            ServerEvent::Help(help) => {}
            ServerEvent::RoomEvent(username, RoomEvent::Message(message)) => {}
            ServerEvent::RoomEvent(username, RoomEvent::Joined(room))
            | ServerEvent::RoomEvent(username, RoomEvent::Left(room)) => {
                self.current_room = room;
                tcp_writer.send(ServerCommand::Users.to_string()).await?;
                tcp_writer.send(ServerCommand::Rooms.to_string()).await?;
            }
            ServerEvent::RoomEvent(username, RoomEvent::NameChange(new_username)) => {
                if username == self.username {
                    self.username = new_username;
                }
            }
            ServerEvent::RoomEvent(username, RoomEvent::File(name, contents)) => {}
            ServerEvent::Error(error) => {}
            ServerEvent::Rooms(rooms) => {
                self.rooms = rooms;
            }
            ServerEvent::Users(users) => {
                self.users = users;
            }
        }
        Ok(())
    }

    pub fn draw_ui(&mut self, frame: &mut Frame) {
        let [message_area, text_area] =
            Layout::vertical([Constraint::Percentage(100), Constraint::Min(3)]).areas(frame.area());

        frame.render_widget(&self.text_area, text_area);

        let [message_area, room_area] =
            Layout::horizontal([Constraint::Percentage(80), Constraint::Percentage(20)])
                .areas(message_area);

        let title = format!("Room - {} [{}]", self.current_room, self.username);
        let list = List::new(
            self.messages
                .iter()
                .rev()
                .map(|event| ListItem::new(event.as_json_str())),
        )
        .block(Block::bordered().title(title))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol(">>")
        .repeat_highlight_symbol(true)
        .direction(ListDirection::BottomToTop);
        frame.render_stateful_widget(list, message_area, &mut self.list_state);

        let leaves = self
            .rooms
            .iter()
            .map(|room| {
                if room == &self.current_room {
                    TreeItem::new(
                        self.current_room.as_str(),
                        room.as_str(),
                        self.users
                            .iter()
                            .map(|user| TreeItem::new_leaf(user.as_str(), user.as_str()))
                            .collect(),
                    )
                    .unwrap()
                } else {
                    TreeItem::new(room.as_str(), room.as_str(), vec![]).unwrap()
                }
            })
            .collect::<Vec<TreeItem<&str>>>();
        let mut tree_state = TreeState::default();
        tree_state.open(vec![self.current_room.as_str()]);
        frame.render_stateful_widget(
            Tree::new(&leaves)
                .unwrap()
                .block(Block::default().borders(Borders::ALL).title("Rooms"))
                .style(Style::default().fg(Color::White)),
            room_area,
            &mut tree_state,
        );

        if self.popup == Popup::FileExplorer {
            let popup_area = popup_area(frame.area(), 80, 80);
            frame.render_widget(Clear, popup_area);
            frame.render_widget(&self.file_explorer.widget(), popup_area);
        } else if let Popup::ImagePreview(protocol) = &mut self.popup {
            let popup_area = popup_area(frame.area(), 80, 80);
            frame.render_widget(Clear, popup_area);
            let image = StatefulImage::new(None);
            frame.render_stateful_widget(image, popup_area, protocol);
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = Args::parse_socket_addr();
    let mut connection = TcpStream::connect(addr).await?;
    let (reader, writer) = connection.split();
    let mut tcp_writer = FramedWrite::new(writer, LinesCodec::new());
    let mut tcp_reader = FramedRead::new(reader, LinesCodec::new());

    let (mut client_writer, mut client_reader) = tokio::sync::mpsc::unbounded_channel::<Event>();

    tcp_writer
        .send(ServerCommand::Name("orhun".to_string()).to_string())
        .await?;

    let mut app = App::new()?;
    let mut terminal = ratatui::init();
    let mut term_stream = crossterm::event::EventStream::new();

    while app.is_running {
        terminal.draw(|f| {
            app.draw_ui(f);
        })?;
        tokio::select! {
            term_event = term_stream.next() => {
                if let Some(event) = term_event {
                    let event = Event::Terminal(event?);
                    app.handle_event(event, &mut tcp_writer, &mut client_writer).await?;
                }
            },
            client_event = client_reader.recv() => {
                if let Some(event) = client_event {
                    app.handle_event(event, &mut tcp_writer, &mut client_writer).await?;
                }
            }
            tcp_event = tcp_reader.next() => {
                if let Some(tcp_event) = tcp_event {
                    app.handle_tcp_event(tcp_event?, &mut tcp_writer).await?;
                }
            },
        }
    }

    ratatui::restore();
    Ok(())
}
