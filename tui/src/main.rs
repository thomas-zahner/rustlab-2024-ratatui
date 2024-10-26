#![allow(unused_imports, dead_code, unused_variables)]
mod app;

use app::App;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = App::new();
    let terminal = ratatui::init();
    let result = app.run(terminal).await;
    ratatui::restore();
    result
}
