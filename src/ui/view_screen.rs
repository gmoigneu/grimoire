use crate::models::{Category, Item};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};

#[derive(Default)]
pub struct ViewState {
    pub scroll: u16,
    pub max_scroll: u16,
    pub viewing_version: Option<i64>, // None means latest/current
    pub max_version: i64,             // Current/latest version number
}

pub fn draw(frame: &mut Frame, item: Option<&Item>, view_state: &mut ViewState) {
    let item = match item {
        Some(item) => item,
        None => {
            let msg =
                Paragraph::new("No item selected").style(Style::default().fg(Color::DarkGray));
            frame.render_widget(msg, frame.area());
            return;
        }
    };

    let is_viewing_old = view_state.viewing_version.is_some()
        && view_state.viewing_version != Some(view_state.max_version);

    let constraints = if is_viewing_old {
        vec![
            Constraint::Length(1), // Title bar
            Constraint::Length(1), // Version warning banner
            Constraint::Length(9), // Metadata section
            Constraint::Length(5), // Description section
            Constraint::Min(0),    // Content section
            Constraint::Length(1), // Status bar
        ]
    } else {
        vec![
            Constraint::Length(1), // Title bar
            Constraint::Length(9), // Metadata section
            Constraint::Length(5), // Description section
            Constraint::Min(0),    // Content section
            Constraint::Length(1), // Status bar
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(frame.area());

    // Title bar
    let title = format!(" {}: {} ", item.category.display_name(), item.name);
    let title_bar = Paragraph::new(Line::from(vec![
        Span::styled(
            title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("                                                        "),
        Span::styled("[ESC] Back", Style::default().fg(Color::DarkGray)),
    ]));
    frame.render_widget(title_bar, chunks[0]);

    let mut idx = 1;

    // Version warning banner (only when viewing old version)
    if is_viewing_old {
        let viewing_v = view_state.viewing_version.unwrap_or(1);
        let banner = Paragraph::new(Line::from(vec![
            Span::styled(" ⚠ ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!(
                    "Viewing version {} of {}  ",
                    viewing_v, view_state.max_version
                ),
                Style::default().fg(Color::Yellow),
            ),
            Span::styled("[L] Go to latest", Style::default().fg(Color::Cyan)),
        ]))
        .style(Style::default().bg(Color::DarkGray));
        frame.render_widget(banner, chunks[idx]);
        idx += 1;
    }

    // Metadata section
    draw_metadata(frame, chunks[idx], item, view_state);
    idx += 1;

    // Description section
    draw_description(frame, chunks[idx], item);
    idx += 1;

    // Content section
    draw_content(frame, chunks[idx], item, view_state);
    idx += 1;

    // Status bar
    draw_status_bar(frame, chunks[idx], is_viewing_old);
}

fn draw_metadata(frame: &mut Frame, area: Rect, item: &Item, view_state: &ViewState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Version display
    let version_display = if view_state.viewing_version.is_some() {
        view_state.viewing_version.unwrap_or(item.version)
    } else {
        item.version
    };

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Name:        ", Style::default().fg(Color::Yellow)),
            Span::raw(&item.name),
        ]),
        Line::from(vec![
            Span::styled("Category:    ", Style::default().fg(Color::Yellow)),
            Span::raw(item.category.display_name()),
        ]),
        Line::from(vec![
            Span::styled("Version:     ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("v{}", version_display),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::styled("Tags:        ", Style::default().fg(Color::Yellow)),
            Span::styled(
                item.tags.clone().unwrap_or_else(|| "none".to_string()),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    ];

    // Category-specific fields
    match item.category {
        Category::Agent => {
            lines.push(Line::from(vec![
                Span::styled("Model:       ", Style::default().fg(Color::Yellow)),
                Span::raw(item.model.clone().unwrap_or_else(|| "default".to_string())),
            ]));
            lines.push(Line::from(vec![
                Span::styled("Tools:       ", Style::default().fg(Color::Yellow)),
                Span::raw(item.tools.clone().unwrap_or_else(|| "all".to_string())),
            ]));
            if let Some(ref perm) = item.permission_mode {
                lines.push(Line::from(vec![
                    Span::styled("Permissions: ", Style::default().fg(Color::Yellow)),
                    Span::raw(perm),
                ]));
            }
        }
        Category::Command => {
            if let Some(ref hint) = item.argument_hint {
                lines.push(Line::from(vec![
                    Span::styled("Arguments:   ", Style::default().fg(Color::Yellow)),
                    Span::raw(hint),
                ]));
            }
            if let Some(ref tools) = item.allowed_tools {
                lines.push(Line::from(vec![
                    Span::styled("Tools:       ", Style::default().fg(Color::Yellow)),
                    Span::raw(tools),
                ]));
            }
        }
        Category::Skill => {
            if let Some(ref tools) = item.allowed_tools {
                lines.push(Line::from(vec![
                    Span::styled("Tools:       ", Style::default().fg(Color::Yellow)),
                    Span::raw(tools),
                ]));
            }
        }
        Category::Prompt => {}
    }

    // Timestamps
    lines.push(Line::from(vec![
        Span::styled("Updated:     ", Style::default().fg(Color::Yellow)),
        Span::styled(item.updated_ago(), Style::default().fg(Color::DarkGray)),
    ]));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn draw_description(frame: &mut Frame, area: Rect, item: &Item) {
    let block = Block::default()
        .title(" Description ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let desc = item
        .description
        .clone()
        .unwrap_or_else(|| "No description".to_string());
    let paragraph = Paragraph::new(desc)
        .block(block)
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(if item.description.is_some() {
            Color::White
        } else {
            Color::DarkGray
        }));

    frame.render_widget(paragraph, area);
}

fn draw_content(frame: &mut Frame, area: Rect, item: &Item, view_state: &mut ViewState) {
    let block = Block::default()
        .title(" Content ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);

    // Calculate max scroll based on content height
    let content_lines = item.content.lines().count() as u16;
    view_state.max_scroll = content_lines.saturating_sub(inner.height);

    let paragraph = Paragraph::new(item.content.clone())
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((view_state.scroll, 0));

    frame.render_widget(paragraph, area);

    // Scrollbar
    if view_state.max_scroll > 0 {
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));

        let mut scrollbar_state = ScrollbarState::new(view_state.max_scroll as usize)
            .position(view_state.scroll as usize);

        frame.render_stateful_widget(
            scrollbar,
            area.inner(ratatui::layout::Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut scrollbar_state,
        );
    }
}

fn draw_status_bar(frame: &mut Frame, area: Rect, is_viewing_old: bool) {
    let mut shortcuts = vec![
        ("e ", "edit"),
        ("c ", "copy"),
        ("C-a ", "ai-assist"),
        ("h ", "history"),
    ];

    if is_viewing_old {
        shortcuts.push(("L ", "latest"));
    }

    shortcuts.extend([("x ", "export"), ("dd ", "delete"), ("ESC ", "back")]);

    let spans: Vec<Span> = shortcuts
        .iter()
        .flat_map(|(key, action)| {
            vec![
                Span::styled(*key, Style::default().fg(Color::Yellow)),
                Span::styled(
                    format!("{}  ", action),
                    Style::default().fg(Color::DarkGray),
                ),
            ]
        })
        .collect();

    let status = Paragraph::new(Line::from(spans)).style(Style::default().bg(Color::Black));

    frame.render_widget(status, area);
}
