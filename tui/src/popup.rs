use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui_image::protocol::StatefulProtocol;

pub enum Popup {
    None,
    FileExplorer,
    ImagePreview(Box<dyn StatefulProtocol>),
}

impl PartialEq for Popup {
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Popup::None, Popup::None)
                | (Popup::FileExplorer, Popup::FileExplorer)
                | (Popup::ImagePreview(_), Popup::ImagePreview(_))
        )
    }
}

impl Popup {
    pub fn area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
        let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
    }
}
