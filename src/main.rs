mod app;
mod db;
mod export;
mod llm;
mod models;
mod ui;

use app::App;
use color_eyre::eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;

    let terminal = ratatui::init();
    let app_result = App::new()?.run(terminal);
    ratatui::restore();

    app_result
}
