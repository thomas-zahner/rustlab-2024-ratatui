use common::{RoomEvent, ServerEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style, Stylize},
    text::Line,
    widgets::{Block, List, ListDirection, ListItem, ListState, StatefulWidget, Widget},
};

#[derive(Debug, Clone, Default)]
pub struct MessageList {
    pub state: ListState,
    pub events: Vec<ServerEvent>,
    pub room: String,
    pub username: String,
}

impl MessageList {
    pub fn selected_event(&self) -> Option<ServerEvent> {
        self.state
            .selected()
            .map(|v| self.events[self.events.len() - (v + 1)].clone())
    }
}

impl Widget for &mut MessageList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut items = Vec::new();
        for event in self.events.iter().rev() {
            match event {
                ServerEvent::Help(_, contents) => {
                    items.push(ListItem::new(contents.clone().blue()));
                }
                ServerEvent::RoomEvent(username, room_event) => match room_event {
                    RoomEvent::Message(message) => {
                        items.push(ListItem::new(Line::from(vec![
                            if username == &self.username {
                                username.clone().yellow().bold()
                            } else {
                                username.clone().cyan().bold()
                            },
                            ": ".white(),
                            message.into(),
                        ])));
                    }
                    RoomEvent::Joined(_) => {
                        items.push(ListItem::new(
                            format!("{} joined the room", username).italic(),
                        ));
                    }
                    RoomEvent::Left(_) => {
                        items.push(ListItem::new(
                            format!("{} left the room", username).italic(),
                        ));
                    }
                    RoomEvent::NameChange(name) => {
                        items.push(ListItem::new(Line::from(vec![
                            username.clone().cyan().bold(),
                            " is now known as ".into(),
                            name.clone().green().italic(),
                        ])));
                    }
                    RoomEvent::File(file, _) => {
                        items.push(ListItem::new(Line::from(vec![
                            username.clone().cyan().bold(),
                            " sent a file: ".into(),
                            file.clone().red().magenta(),
                        ])));
                    }
                },
                ServerEvent::Error(error) => {
                    items.push(ListItem::new(format!("Error: {error}").red()));
                }
                _ => {}
            }
        }

        let list = List::new(items)
            .block(Block::bordered().title("[ Messages ]"))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::BottomToTop);

        StatefulWidget::render(list, area, buf, &mut self.state);
    }
}
