use common::{RoomEvent, RoomName, ServerEvent, Username};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, List, ListDirection, ListItem, Paragraph, Widget},
};

#[derive(Debug, Clone, Default)]
pub struct MessageList {
    pub events: Vec<ServerEvent>,
    pub room_name: RoomName,
    pub username: Username,
}

impl Widget for &mut MessageList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // TODO: Implement rendering a List: <https://docs.rs/ratatui/latest/ratatui/widgets/struct.List.html>
        // Use the helper functions below to render the list items

        let text = vec![Line::from(vec![
            Span::raw(format!("Room: {} ", self.room_name)),
            Span::styled(
                format!("User: {}", self.username),
                Style::new().green().italic(),
            ),
        ])];

        Paragraph::new(text).render(area, buf);
    }
}

impl MessageList {
    fn server_event_line<'a>(&self, event: &'a ServerEvent) -> Option<Line<'a>> {
        match event {
            ServerEvent::CommandHelp(_, contents) => todo!("return help line"),
            ServerEvent::RoomEvent {
                room_name: _,
                username,
                date,
                event,
            } => self.room_event_line(username.clone(), date, event),
            ServerEvent::Error(error) => todo!("return error line"),
            _ => None,
        }
    }

    fn room_event_line<'a>(
        &self,
        username: Username,
        date: &'a str,
        event: &'a RoomEvent,
    ) -> Option<Line<'a>> {
        match event {
            RoomEvent::Message(message) => {
                todo!("return message line")
            }
            RoomEvent::Joined(room) => todo!("user joined line"),
            RoomEvent::Left(room) => todo!("user left line"),
            RoomEvent::NameChange(name) => todo!("user changed name line"),
            _ => None,
        }
    }
}
