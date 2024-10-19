mod args;

use args::Args;
use futures::{SinkExt, StreamExt};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListDirection, ListItem};
use tokio::net::TcpStream;
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};
use tui_textarea::{Input, Key, TextArea};

fn textarea_new() -> TextArea<'static> {
    let mut textarea = TextArea::default();
    textarea.set_cursor_line_style(Style::default());
    textarea.set_placeholder_text("Start typing...");
    textarea.set_block(Block::default().borders(Borders::ALL).title("Send message"));
    textarea
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = Args::parse_socket_addr();
    let mut connection = TcpStream::connect(addr).await?;

    let (reader, writer) = connection.split();
    let mut tcp_sink = FramedWrite::new(writer, LinesCodec::new());
    let mut tcp_stream = FramedRead::new(reader, LinesCodec::new());

    let mut terminal = ratatui::init();

    let mut textarea = textarea_new();
    let layout = Layout::default().constraints([Constraint::Percentage(100), Constraint::Min(3)]);

    let mut messages: Vec<String> = Vec::new();
    let mut current_room = "lobby".to_owned();
    let mut term_stream = crossterm::event::EventStream::new();

    loop {
        terminal.draw(|f| {
            let chunks = layout.split(f.area());

            let title = format!("Room - {current_room}");

            let list = List::new(messages.iter().rev().map(|msg| ListItem::new(msg.as_str())))
                .block(Block::bordered().title(title))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
                .highlight_symbol(">>")
                .repeat_highlight_symbol(true)
                .direction(ListDirection::BottomToTop);

            f.render_widget(list, chunks[0]);
            f.render_widget(&textarea, chunks[1]);
        })?;

        tokio::select! {
            term_event = term_stream.next() => {
                if let Some(event) = term_event {
                    let event = event?;
                    match event.into() {
                        // escape
                        Input { key: Key::Esc, .. } |
                        // ctrl+c
                        Input { key: Key::Char('c'), ctrl: true, .. } |
                        // ctrl+d
                        Input { key: Key::Char('d'), ctrl: true, .. }  => break,
                        // enter
                        Input { key: Key::Enter, .. } => {
                            if textarea.is_empty() {
                                continue;
                            }
                            //messages.extend(textarea.into_lines());
                            for line in textarea.into_lines() {
                                // tracing::info!("SENT {line}");
                                match tcp_sink.send(line).await {
                                    Ok(_) => (),
                                    Err(_) => break,
                                };
                            }
                            textarea = textarea_new();
                        }
                        // forward input to textarea
                        input => {
                            // messages.push(format!("{:?}", input));
                            // TextArea::input returns if the input modified its text
                            textarea.input_without_shortcuts(input);
                        }
                    }
                } else {
                    break;
                }
            },
            tcp_event = tcp_stream.next() => match tcp_event {
                Some(event) => {
                    let server_msg = event?;
                    if server_msg.starts_with("You joined ") {
                        let room_name = server_msg
                            .split_ascii_whitespace()
                            .nth(2)
                            .unwrap();
                        current_room = room_name.to_owned();
                    } else if server_msg.starts_with("You are ") {
                        let name = server_msg
                            .split_ascii_whitespace()
                            .nth(2)
                            .unwrap();
                    }
                    messages.push(server_msg);
                },
                None => break,
            },
        }
    }

    ratatui::restore();
    Ok(())
}
