mod app;
mod db;
mod export;
mod llm;
mod models;
mod ui;

use app::App;
use color_eyre::eyre::Result;
use crossterm::event::{DisableBracketedPaste, EnableBracketedPaste};
use crossterm::execute;
use std::io::stdout;

fn main() -> Result<()> {
    color_eyre::install()?;

    // Enable bracketed paste mode so pasted text comes as a single event
    execute!(stdout(), EnableBracketedPaste)?;

    let terminal = ratatui::init();
    let app_result = App::new()?.run(terminal);
    ratatui::restore();

    // Disable bracketed paste mode
    let _ = execute!(stdout(), DisableBracketedPaste);

    app_result
}
