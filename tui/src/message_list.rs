use common::ServerEvent;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
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
        let title = format!("Room - {} [{}]", self.room, self.username);
        let list = List::new(
            self.events
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

        StatefulWidget::render(list, area, buf, &mut self.state);
    }
}
