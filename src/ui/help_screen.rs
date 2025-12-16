use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

#[derive(Default)]
pub struct HelpState {
    pub scroll: u16,
    pub max_scroll: u16,
}


impl HelpState {
    pub fn scroll_down(&mut self) {
        if self.scroll < self.max_scroll {
            self.scroll += 1;
        }
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }
}

pub fn draw(frame: &mut Frame, state: &mut HelpState) {
    let area = centered_rect(80, 80, frame.area());

    // Clear the area behind the popup
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Help - GRIMOIRE ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(inner);

    // Help content
    let help_text = get_help_content();
    state.max_scroll = help_text.len().saturating_sub(chunks[0].height as usize) as u16;

    let paragraph = Paragraph::new(help_text)
        .scroll((state.scroll, 0));

    frame.render_widget(paragraph, chunks[0]);

    // Scrollbar
    if state.max_scroll > 0 {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        let mut scrollbar_state = ScrollbarState::new(state.max_scroll as usize)
            .position(state.scroll as usize);

        frame.render_stateful_widget(
            scrollbar,
            chunks[0],
            &mut scrollbar_state,
        );
    }

    // Status bar
    let status = Paragraph::new(Line::from(vec![
        Span::styled("j/k ", Style::default().fg(Color::Yellow)),
        Span::styled("scroll  ", Style::default().fg(Color::DarkGray)),
        Span::styled("ESC/? ", Style::default().fg(Color::Yellow)),
        Span::styled("close", Style::default().fg(Color::DarkGray)),
    ]));
    frame.render_widget(status, chunks[1]);
}

fn get_help_content() -> Vec<Line<'static>> {
    let sections = vec![
        ("NAVIGATION", vec![
            ("j / ↓", "Move down"),
            ("k / ↑", "Move up"),
            ("h / ←", "Focus sidebar"),
            ("l / →", "Focus item list"),
            ("gg", "Go to top"),
            ("G", "Go to bottom"),
            ("Ctrl+d", "Page down"),
            ("Ctrl+u", "Page up"),
        ]),
        ("ACTIONS", vec![
            ("Enter", "View selected item"),
            ("e", "Edit selected item"),
            ("n", "Create new item"),
            ("c / yy", "Copy content to clipboard"),
            ("dd", "Delete item (with confirmation)"),
            ("x", "Export to .claude/ directory"),
            ("/", "Open search"),
            ("s", "Open settings"),
            ("?", "Show this help"),
            ("q / ESC", "Quit / Back"),
        ]),
        ("QUICK FILTERS", vec![
            ("1", "Show Prompts"),
            ("2", "Show Agents"),
            ("3", "Show Skills"),
            ("4", "Show Commands"),
            ("0", "Show all (recent)"),
        ]),
        ("EDIT MODE", vec![
            ("Tab", "Next field"),
            ("Shift+Tab", "Previous field"),
            ("Ctrl+S", "Save"),
            ("a", "AI assistant (in content field)"),
            ("ESC", "Cancel"),
        ]),
        ("SEARCH", vec![
            ("j / k", "Navigate results"),
            ("Enter", "Select result"),
            ("c", "Copy selected item"),
            ("ESC", "Close search"),
        ]),
        ("VIEW MODE", vec![
            ("j / k", "Scroll content"),
            ("e", "Edit item"),
            ("c / yy", "Copy content"),
            ("x", "Export item"),
            ("a", "AI assistant"),
            ("ESC / q", "Back to list"),
        ]),
    ];

    let mut lines = Vec::new();

    lines.push(Line::from(Span::styled(
        "GRIMOIRE - Manage your Claude Code configurations",
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    for (section_title, shortcuts) in sections {
        lines.push(Line::from(Span::styled(
            section_title,
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        for (key, desc) in shortcuts {
            lines.push(Line::from(vec![
                Span::styled(format!("  {:12}", key), Style::default().fg(Color::Green)),
                Span::raw(desc),
            ]));
        }
        lines.push(Line::from(""));
    }

    lines.push(Line::from(Span::styled(
        "ITEM CATEGORIES",
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  Prompts   ", Style::default().fg(Color::Green)),
        Span::raw("Reusable prompt templates (copy-only, no export)"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Agents    ", Style::default().fg(Color::Green)),
        Span::raw("Sub-agents with custom tools and permissions"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Skills    ", Style::default().fg(Color::Green)),
        Span::raw("Auto-invoked capabilities with instructions"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Commands  ", Style::default().fg(Color::Green)),
        Span::raw("Custom slash commands for quick actions"),
    ]));

    lines
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
