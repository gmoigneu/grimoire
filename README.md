# GRIMOIRE

**G**lobal **R**epository and **I**ndex for **M**odel **O**perations, **I**nstructions and **R**esponse **E**ngineering

A terminal user interface (TUI) application for managing prompts, agents, and skills configuration.

![Rust](https://img.shields.io/badge/rust-2024-orange.svg)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

![grimoire](./grimoire.gif)

## Features

- Manage and organize your LLM prompts in a searchable database
- Configure and switch between multiple LLM providers (Anthropic, OpenAI)
- AI-powered prompt improvement suggestions
- Fast, keyboard-driven terminal interface
- Local SQLite storage for your data

## Installation

### Homebrew (macOS/Linux)

```bash
brew tap gmoigneu/grimoire
brew install grimoire
```

### Download Binary

Pre-built binaries are available on the [Releases](https://github.com/gmoigneu/grimoire/releases) page for:
- Linux (x86_64, aarch64)
- macOS (Intel, Apple Silicon)
- Windows (x86_64)

### From Source

```bash
git clone https://github.com/gmoigneu/grimoire.git
cd grimoire
cargo build --release
```

The binary will be available at `target/release/grimoire`.

### Requirements

- Rust 2024 edition (1.85+)
- A terminal with Unicode support

## Usage

```bash
# Run the application
cargo run

# Or run the release binary
./target/release/grimoire
```

### Keyboard Shortcuts

#### Main Screen

| Key | Action |
|-----|--------|
| `q` | Quit |
| `j` / `Down` | Move down |
| `k` / `Up` | Move up |
| `h` / `Left` | Focus categories |
| `l` / `Right` | Focus item list |
| `Enter` | View selected item |
| `n` | New item |
| `e` | Edit item |
| `c` / `yy` | Copy item to clipboard |
| `dd` | Delete item |
| `x` | Export item |
| `/` | Search |
| `s` | Settings |
| `?` | Help |
| `gg` | Go to top |
| `G` | Go to bottom |
| `Ctrl+d` | Page down |
| `Ctrl+u` | Page up |
| `0` | Show all categories |
| `1` | Filter: Prompts |
| `2` | Filter: Agents |
| `3` | Filter: Skills |
| `4` | Filter: Commands |

#### View Screen

| Key | Action |
|-----|--------|
| `q` / `Esc` | Back to list |
| `j` / `Down` | Scroll down |
| `k` / `Up` | Scroll up |
| `e` | Edit item |
| `c` / `yy` | Copy to clipboard |
| `dd` | Delete item |
| `x` | Export item |
| `h` | View history |
| `L` | Go to latest version |
| `Ctrl+a` | AI improve prompt |

#### Edit Screen

| Key | Action |
|-----|--------|
| `Esc` | Cancel and go back |
| `Tab` | Next field |
| `Shift+Tab` | Previous field |
| `Ctrl+s` | Save |
| `Ctrl+a` | AI improve (content field) |
| `Enter` | Newline (in content) / Toggle (dropdowns) |
| `Space` | Toggle dropdown options |

#### Search

| Key | Action |
|-----|--------|
| `Esc` | Close search |
| `Enter` | Select result |
| `j` / `Down` | Next result |
| `k` / `Up` | Previous result |
| `c` | Copy selected to clipboard |

#### Settings

| Key | Action |
|-----|--------|
| `q` / `Esc` | Close settings |
| `Tab` | Next field |
| `Shift+Tab` | Previous field |
| `Ctrl+s` | Save settings |
| `Enter` / `Space` | Toggle dropdown |
| `Left` / `Right` | Change dropdown selection |

#### History Popup

| Key | Action |
|-----|--------|
| `q` / `Esc` | Close |
| `j` / `Down` | Next version |
| `k` / `Up` | Previous version |
| `Enter` | View version |
| `r` | Restore version |

## Configuration

GRIMOIRE stores its configuration and database in `~/.config/grimoire/`:

- `grimoire.db` - SQLite database containing your prompts and settings
- Settings for LLM providers can be configured within the application

### LLM Providers

Configure your API keys for the supported providers:

- **Anthropic** - Claude models
- **OpenAI** - GPT models

## Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Lint
cargo clippy

# Format
cargo fmt
```

## Development

### Setup

Enable git hooks for format checking on commit:

```bash
git config core.hooksPath .githooks
```

### Generate Fixture Data

To populate the database with sample data for testing:

```bash
./scripts/generate_fixtures.sh
```

This creates 200 sample items (prompts, agents, skills, and commands) with varied content and tags. The application must be run at least once before generating fixtures to create the database.

## Project Structure

```
src/
├── main.rs          # Application entry point
├── app.rs           # Main application state and logic
├── db/              # Database schema and operations
├── models/          # Data models
├── screens/         # TUI screens (main, edit, view, settings)
├── ui/              # UI components and widgets
└── llm/             # LLM client integrations
```

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.
