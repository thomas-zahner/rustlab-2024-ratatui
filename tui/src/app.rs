use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use common::{RoomEvent, ServerCommand, ServerEvent};
use crossterm::event::Event as CrosstermEvent;
use futures::SinkExt;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType};
use ratatui_explorer::{FileExplorer, Theme};
use ratatui_image::picker::{Picker, ProtocolType};
use tokio::net::tcp::WriteHalf;
use tokio_util::codec::{FramedWrite, LinesCodec};
use tui_textarea::{Input, Key, TextArea};

use crate::message_list::MessageList;
use crate::popup::Popup;
use crate::room_list::RoomList;

pub struct App {
    pub is_running: bool,
    pub message_list: MessageList,
    pub room_list: RoomList,
    pub text_area: TextArea<'static>,
    pub file_explorer: FileExplorer,
    pub popup: Popup,
}

#[derive(Clone)]
pub enum Event {
    Terminal(CrosstermEvent),
    FileSelected,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        // Message list.
        let message_list = MessageList::default();

        // Room list.
        let room_list = RoomList::default();

        // Create text input
        let mut text_area = TextArea::default();
        text_area.set_cursor_line_style(Style::default());
        text_area.set_placeholder_text("Start typing...");

        // Create a file explorer
        let file_explorer = FileExplorer::with_theme(
            Theme::default()
                .add_default_title()
                .with_title_bottom(|fe| format!("[ {} files ]", fe.files().len()).into())
                .with_style(Style::default().fg(Color::Yellow))
                .with_highlight_item_style(Style::default().add_modifier(Modifier::BOLD))
                .with_highlight_dir_style(
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                )
                .with_highlight_symbol("> ")
                .with_block(Block::bordered().border_type(BorderType::Rounded)),
        )?;

        Ok(Self {
            is_running: true,
            message_list,
            room_list,
            text_area,
            file_explorer,
            popup: Popup::None,
        })
    }

    pub async fn handle_event(
        &mut self,
        event: Event,
        tcp_writer: &mut FramedWrite<WriteHalf<'_>, LinesCodec>,
        event_sender: &mut tokio::sync::mpsc::UnboundedSender<Event>,
    ) -> anyhow::Result<()> {
        match event {
            Event::Terminal(raw_event) => {
                let event = raw_event.clone().into();
                // Handle popup
                match self.popup {
                    Popup::FileExplorer => {
                        if let Input { key: Key::Esc, .. } = event {
                            self.popup = Popup::None;
                        } else if let Input {
                            key: Key::Enter, ..
                        } = event
                        {
                            self.popup = Popup::None;
                            event_sender.send(Event::FileSelected)?;
                        } else {
                            self.file_explorer.handle(&raw_event)?;
                        }
                        return Ok(());
                    }
                    Popup::ImagePreview(_) => {
                        if let Input { key: Key::Esc, .. } = event {
                            self.popup = Popup::None;
                        }
                        return Ok(());
                    }
                    _ => {}
                }

                // Handle key input
                match event {
                    // Esc
                    Input { key: Key::Esc, .. } => self.is_running = false,
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
                        self.message_list.state.select_previous();
                    }
                    // Up
                    Input { key: Key::Up, .. } => {
                        self.message_list.state.select_next();
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
                        let selected_event = self.message_list.selected_event();
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
                    // Other key presses
                    input => {
                        self.text_area.input_without_shortcuts(input);
                    }
                }
            }
            // Send file to server
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
        self.message_list.events.push(event.clone());
        match event {
            ServerEvent::Help(username, help) => {
                self.message_list.username = username;
            }
            ServerEvent::RoomEvent(username, RoomEvent::Message(message)) => {}
            ServerEvent::RoomEvent(username, RoomEvent::Joined(room))
            | ServerEvent::RoomEvent(username, RoomEvent::Left(room)) => {
                self.message_list.room = room.clone();
                self.room_list.room = room;
                tcp_writer.send(ServerCommand::Users.to_string()).await?;
                tcp_writer.send(ServerCommand::Rooms.to_string()).await?;
            }
            ServerEvent::RoomEvent(username, RoomEvent::NameChange(new_username)) => {
                if username == self.message_list.username {
                    self.message_list.username = new_username;
                }
            }
            ServerEvent::RoomEvent(username, RoomEvent::File(name, contents)) => {}
            ServerEvent::Error(error) => {}
            ServerEvent::Rooms(rooms) => {
                self.room_list.rooms = rooms;
            }
            ServerEvent::Users(users) => {
                self.room_list.users = users;
            }
        }
        Ok(())
    }
}
