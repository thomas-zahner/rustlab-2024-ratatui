use std::io;

use base64::{prelude::BASE64_STANDARD, Engine};
use crossterm::event::Event as CrosstermEvent;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Offset, Rect},
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, BorderType, Clear, Paragraph, StatefulWidget, Widget, Wrap},
};
use ratatui_explorer::{FileExplorer, Theme};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol, StatefulImage};
use tachyonfx::{
    fx::{self, Direction as FxDirection},
    Duration, Effect, EffectRenderer, EffectTimer, Interpolation, Shader,
};
use tokio::sync::mpsc::UnboundedSender;
use tui_textarea::{Input, Key};

use crate::app::Event;

pub enum Popup {
    Help(String, UnboundedSender<Event>),
    FileExplorer(FileExplorer, UnboundedSender<Event>),
    ImagePreview(Box<dyn StatefulProtocol>, UnboundedSender<Event>),
    MarkdownPreview(String, UnboundedSender<Event>),
    Effect(Effect, UnboundedSender<Event>),
}

impl Popup {
    pub fn help(key_bindings: String, event_sender: UnboundedSender<Event>) -> Self {
        Self::Help(key_bindings, event_sender)
    }

    pub fn file_explorer(event_sender: UnboundedSender<Event>) -> io::Result<Self> {
        let theme = Theme::default()
            .add_default_title()
            .with_title_bottom(|fe| format!("[ {} files ]", fe.files().len()).into())
            .with_style(Color::Yellow)
            .with_highlight_item_style(Modifier::BOLD)
            .with_highlight_dir_style(Style::new().blue().bold())
            .with_highlight_symbol("> ")
            .with_block(Block::bordered().border_type(BorderType::Rounded));
        let file_explorer = FileExplorer::with_theme(theme)?;
        Ok(Self::FileExplorer(file_explorer, event_sender))
    }

    pub fn image_preview(
        contents: String,
        event_sender: UnboundedSender<Event>,
    ) -> anyhow::Result<Popup> {
        let data = BASE64_STANDARD.decode(contents.as_bytes())?;
        let img = image::load_from_memory(&data)?;
        let user_fontsize = (7, 14);
        let mut picker = Picker::new(user_fontsize);
        picker.guess_protocol();
        let image = picker.new_resize_protocol(img);
        Ok(Popup::ImagePreview(image, event_sender))
    }

    pub fn markdown_preview(
        contents: String,
        event_sender: UnboundedSender<Event>,
    ) -> anyhow::Result<Popup> {
        let contents = BASE64_STANDARD.decode(contents.as_bytes())?;
        Ok(Popup::MarkdownPreview(
            String::from_utf8(contents)?,
            event_sender,
        ))
    }

    pub fn effect(event_sender: UnboundedSender<Event>) -> Self {
        let effect = fx::sequence(&[
            fx::ping_pong(fx::sweep_in(
                FxDirection::DownToUp,
                10,
                0,
                Color::DarkGray,
                EffectTimer::from_ms(3000, Interpolation::QuadIn),
            )),
            fx::hsl_shift_fg([360.0, 0.0, 0.0], 750),
            fx::hsl_shift_fg([0.0, -100.0, 0.0], 750),
            fx::hsl_shift_fg([0.0, -100.0, 0.0], 750).reversed(),
            fx::hsl_shift_fg([0.0, 100.0, 0.0], 750),
            fx::hsl_shift_fg([0.0, 100.0, 0.0], 750).reversed(),
            fx::hsl_shift_fg([0.0, 0.0, -100.0], 750),
            fx::hsl_shift_fg([0.0, 0.0, -100.0], 750).reversed(),
            fx::hsl_shift_fg([0.0, 0.0, 100.0], 750),
            fx::hsl_shift_fg([0.0, 0.0, 100.0], 750).reversed(),
            fx::dissolve((800, Interpolation::SineOut)),
            fx::coalesce((800, Interpolation::SineOut)),
        ]);

        Popup::Effect(effect, event_sender)
    }

    pub async fn handle_input(
        &mut self,
        input: Input,
        raw_event: CrosstermEvent,
    ) -> anyhow::Result<()> {
        match self {
            Popup::Help(_, ref event_sender) if input.key == Key::Esc => {
                let _ = event_sender.send(Event::PopupClosed);
            }
            Popup::FileExplorer(ref mut explorer, ref mut event_sender) => match input.key {
                Key::Esc => {
                    let _ = event_sender.send(Event::PopupClosed);
                }
                Key::Enter => {
                    let file = explorer.current().clone();
                    if file.is_dir() {
                        return Ok(());
                    }
                    let event = Event::FileSelected(file);
                    let _ = event_sender.send(event);
                    let _ = event_sender.send(Event::PopupClosed);
                }
                _ => explorer.handle(&raw_event)?,
            },
            Popup::ImagePreview(_, ref event_sender) if input.key == Key::Esc => {
                let _ = event_sender.send(Event::PopupClosed);
            }
            Popup::MarkdownPreview(_, ref event_sender) if input.key == Key::Esc => {
                let _ = event_sender.send(Event::PopupClosed);
            }
            _ => {}
        }
        Ok(())
    }
}

impl Widget for &mut Popup {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self {
            Popup::Help(ref key_bindings, ..) => render_help(key_bindings, area, buf),
            Popup::FileExplorer(explorer, _) => render_explorer(area, buf, explorer),
            Popup::ImagePreview(ref mut protocol, _) => render_image_preview(area, buf, protocol),
            Popup::MarkdownPreview(contents, _) => {
                render_markdown_preview(area, buf, contents);
            }
            Popup::Effect(effect, event_sender) => {
                if effect.running() {
                    render_effect(area, buf, effect);
                    let _ = event_sender.send(Event::EffectRendered);
                } else {
                    let _ = event_sender.send(Event::PopupClosed);
                }
            }
        }
    }
}

fn render_help(key_bindings: &str, area: Rect, buf: &mut Buffer) {
    let popup_area = popup_area(area, 30, 30);
    Clear.render(popup_area, buf);
    Paragraph::new(key_bindings.trim())
        .wrap(Wrap { trim: false })
        .block(
            Block::bordered()
                .title("Help")
                .title_style(Style::new().bold()),
        )
        .render(popup_area, buf);
}

fn render_explorer(area: Rect, buf: &mut Buffer, explorer: &mut FileExplorer) {
    let popup_area = popup_area(area, 50, 50);
    Clear.render(popup_area, buf);
    explorer.widget().render(popup_area, buf);
}

fn render_image_preview(area: Rect, buf: &mut Buffer, protocol: &mut Box<dyn StatefulProtocol>) {
    let popup_area = popup_area(area, 80, 80);
    let image = StatefulImage::new(None);
    image.render(popup_area, buf, protocol);
}

fn render_markdown_preview(area: Rect, buf: &mut Buffer, contents: &str) {
    let text = tui_markdown::from_str(contents);
    let mut popup_area = popup_area(area, 80, 80);
    if let (Ok(width), Ok(height)) = (u16::try_from(text.width()), u16::try_from(text.height())) {
        popup_area = popup_area.clamp(Rect::new(popup_area.x, popup_area.y, width + 2, height + 2));
    }
    Clear.render(popup_area, buf);
    Block::bordered().render(popup_area, buf);
    text.render(popup_area.offset(Offset { x: 1, y: 1 }), buf);
}

fn render_effect(area: Rect, buf: &mut Buffer, effect: &mut Effect) {
    let popup_area = popup_area(area, 100, 100);
    buf.render_effect(effect, popup_area, Duration::from_millis(10));
}

fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}