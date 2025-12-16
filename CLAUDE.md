# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

GRIMOIRE (Global Repository and Index for Model Operations, Instructions and Response Engineering) is a terminal user interface (TUI) application built in Rust using ratatui. It helps users manage their prompts, agents, and skills configuration.

## Build Commands

```bash
cargo build              # Build debug version
cargo build --release    # Build release version
cargo run                # Run the application
cargo test               # Run all tests
cargo test <test_name>   # Run a single test
cargo clippy             # Lint the code
cargo fmt                # Format the code
```

## Dependencies

- **ratatui**: TUI framework for building the terminal interface
- **crossterm**: Terminal handling (re-exported by ratatui)
- **color-eyre**: Error handling

## Architecture Notes

This project uses Rust 2024 edition. The application should follow the typical ratatui application pattern:

```rust
use color_eyre::Result;
use ratatui::{DefaultTerminal, crossterm::event::{self, Event, KeyCode}};

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = App::default().run(terminal);
    ratatui::restore();
    app_result
}

struct App {
    should_exit: bool,
    // state fields
}

impl App {
    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            }
        }
        Ok(())
    }
}
```

Key components:
- `ratatui::init()` / `ratatui::restore()` for terminal setup/cleanup
- Event loop polling with `event::read()`
- `terminal.draw(|frame| ...)` for rendering
- `Layout::vertical/horizontal` with constraints for arranging widgets
- Common widgets: `List`, `Paragraph`, `Block`, `Table`

## Documentation

Use the context7 MCP to fetch up-to-date ratatui documentation:
- Library ID: `/websites/rs_ratatui`
- Example: `mcp__context7__get-library-docs` with topic "widgets" or "layout"
