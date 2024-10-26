## Ratatui quickstart guide

First, add `ratatui` to your `Cargo.toml`:

```sh
cargo add ratatui
```

Then you can create a simple “Hello World” application:

```rust
use ratatui::{text::Text, Frame};
use ratatui::crossterm::event::{self, Event};

fn main() {
    let mut terminal = ratatui::init();
    loop {
        terminal.draw(draw).expect("failed to draw frame");
        if matches!(event::read().expect("failed to read event"), Event::Key(_)) {
            break;
        }
    }
    ratatui::restore();
}

fn draw(frame: &mut Frame) {
    let text = Text::raw("Hello World!");
    frame.render_widget(text, frame.area());
}
```
