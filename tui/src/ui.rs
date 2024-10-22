use ratatui::layout::{Constraint, Layout};
use ratatui::widgets::Clear;
use ratatui::Frame;
use ratatui_image::StatefulImage;

use crate::app::App;
use crate::popup::Popup;

impl App {
    pub fn draw_ui(&mut self, frame: &mut Frame) {
        let [message_area, text_area] =
            Layout::vertical([Constraint::Percentage(100), Constraint::Min(3)]).areas(frame.area());

        frame.render_widget(&self.text_area, text_area);

        let [message_area, room_area] =
            Layout::horizontal([Constraint::Percentage(80), Constraint::Percentage(20)])
                .areas(message_area);

        frame.render_widget(&mut self.message_list, message_area);
        frame.render_widget(&mut self.room_list, room_area);

        if self.popup == Popup::FileExplorer {
            let popup_area = Popup::area(frame.area(), 80, 80);
            frame.render_widget(Clear, popup_area);
            frame.render_widget(&self.file_explorer.widget(), popup_area);
        } else if let Popup::ImagePreview(protocol) = &mut self.popup {
            let popup_area = Popup::area(frame.area(), 80, 80);
            frame.render_widget(Clear, popup_area);
            let image = StatefulImage::new(None);
            frame.render_stateful_widget(image, popup_area, protocol);
        }
    }
}
