use crate::app::{App, Focus};
use crate::models::Category;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
    Frame,
};

const SELECTED_STYLE: Style = Style::new().bg(Color::DarkGray).add_modifier(Modifier::BOLD);
const HEADER_STYLE: Style = Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD);

pub fn draw(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Title bar
            Constraint::Min(0),     // Main content
            Constraint::Length(1),  // Status bar
        ])
        .split(frame.area());

    draw_title_bar(frame, chunks[0]);
    draw_main_content(frame, chunks[1], app);
    draw_status_bar(frame, chunks[2], app);
}

fn draw_title_bar(frame: &mut Frame, area: Rect) {
    let title = Paragraph::new(Line::from(vec![
        Span::styled(" GRIMOIRE ", Style::default().fg(Color::Cyan).bold()),
        Span::raw("                                                        "),
        Span::styled("[?] Help", Style::default().fg(Color::DarkGray)),
    ]));
    frame.render_widget(title, area);
}

fn draw_main_content(frame: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Min(0)])
        .split(area);

    draw_sidebar(frame, chunks[0], app);
    draw_item_list(frame, chunks[1], app);
}

fn draw_sidebar(frame: &mut Frame, area: Rect, app: &App) {
    let is_focused = app.focus == Focus::Sidebar;
    let border_color = if is_focused { Color::Cyan } else { Color::DarkGray };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(" Categories ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Build sidebar content
    let mut lines = Vec::new();

    // Recent Items (index 0)
    let is_recent_selected = app.sidebar_index == 0 && is_focused;
    let is_recent_active = app.selected_category.is_none() && app.selected_tag.is_none();
    let recent_prefix = if is_recent_active { "> " } else { "  " };
    let recent_style = if is_recent_selected {
        SELECTED_STYLE
    } else if is_recent_active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };
    lines.push(Line::styled(format!("{}Recent", recent_prefix), recent_style));

    // Categories section (indices 1-4)
    for (i, category) in Category::all().iter().enumerate() {
        let count = app.get_category_count(*category);
        let sidebar_index = i + 1; // Offset by 1 for Recent
        let is_selected = app.sidebar_index == sidebar_index && is_focused;
        let is_active = app.selected_category == Some(*category);

        let prefix = if is_active { "> " } else { "  " };
        let text = format!("{}{} ({})", prefix, category.display_name(), count);

        let style = if is_selected {
            SELECTED_STYLE
        } else if is_active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };

        lines.push(Line::styled(text, style));
    }

    // Tags header
    lines.push(Line::raw(""));
    lines.push(Line::styled(" Tags", Style::default().fg(Color::Yellow)));

    // Tags list (indices 5+)
    for (i, (tag, count)) in app.tags.iter().enumerate() {
        let sidebar_index = 5 + i; // After Recent + 4 categories
        let is_selected = app.sidebar_index == sidebar_index && is_focused;
        let is_active = app.selected_tag.as_ref() == Some(tag);

        let prefix = if is_active { "> " } else { "  " };
        let text = format!("{}#{} ({})", prefix, tag, count);

        let style = if is_selected {
            SELECTED_STYLE
        } else if is_active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        lines.push(Line::styled(text, style));
    }

    // Calculate scroll to keep selected item visible
    let selected_line = if app.sidebar_index <= 4 {
        app.sidebar_index
    } else {
        // Account for empty line and "Tags" header between categories and tags
        app.sidebar_index + 2
    };

    let visible_height = inner.height as usize;
    let scroll = if selected_line >= visible_height {
        (selected_line - visible_height + 1) as u16
    } else {
        0
    };

    let paragraph = Paragraph::new(lines).scroll((scroll, 0));
    frame.render_widget(paragraph, inner);
}

fn draw_item_list(frame: &mut Frame, area: Rect, app: &mut App) {
    let is_focused = app.focus == Focus::ItemList;
    let border_color = if is_focused { Color::Cyan } else { Color::DarkGray };

    let title = match (&app.selected_category, &app.selected_tag) {
        (Some(cat), _) => format!(" {} ", cat.display_name()),
        (None, Some(tag)) => format!(" #{} ", tag),
        (None, None) => " Recent Items ".to_string(),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(title);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.items.is_empty() {
        let msg = Paragraph::new("No items found. Press 'n' to create one.")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(msg, inner);
        return;
    }

    // Create header
    let header = Row::new(vec![
        Cell::from("NAME").style(HEADER_STYLE),
        Cell::from("CATEGORY").style(HEADER_STYLE),
        Cell::from("TAGS").style(HEADER_STYLE),
        Cell::from("UPDATED").style(HEADER_STYLE),
    ])
    .height(1);

    // Create rows
    let rows: Vec<Row> = app
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_selected = i == app.selected_item_index && is_focused;

            let (row_style, dim_style) = if is_selected {
                (SELECTED_STYLE, Style::default().fg(Color::Gray).bg(Color::DarkGray))
            } else {
                (Style::default(), Style::default().fg(Color::DarkGray))
            };

            let tags = item.tags.clone().unwrap_or_default();
            let tags_short = if tags.len() > 15 {
                format!("{}...", &tags[..12])
            } else {
                tags
            };

            Row::new(vec![
                Cell::from(item.name.clone()),
                Cell::from(item.category.display_name()),
                Cell::from(tags_short).style(dim_style),
                Cell::from(item.updated_ago()).style(dim_style),
            ])
            .style(row_style)
        })
        .collect();

    let widths = [
        Constraint::Min(15),
        Constraint::Length(10),
        Constraint::Length(15),
        Constraint::Length(12),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .row_highlight_style(SELECTED_STYLE);

    let mut state = TableState::default();
    state.select(Some(app.selected_item_index));

    frame.render_stateful_widget(table, inner, &mut state);
}

fn draw_status_bar(frame: &mut Frame, area: Rect, app: &App) {
    // If there's a status message, show it instead of shortcuts
    if let Some(ref msg) = app.status_message {
        let style = if msg.contains("failed") || msg.contains("Error") {
            Style::default().fg(Color::Red).bg(Color::Black)
        } else {
            Style::default().fg(Color::Green).bg(Color::Black)
        };
        let status = Paragraph::new(format!(" {} ", msg)).style(style);
        frame.render_widget(status, area);
        return;
    }

    let shortcuts = vec![
        ("/ ", "search"),
        ("n ", "new"),
        ("e ", "edit"),
        ("c ", "copy"),
        ("dd ", "delete"),
        ("x ", "export"),
        ("Enter ", "view"),
        ("? ", "help"),
        ("q ", "quit"),
    ];

    let spans: Vec<Span> = shortcuts
        .iter()
        .flat_map(|(key, action)| {
            vec![
                Span::styled(*key, Style::default().fg(Color::Yellow)),
                Span::styled(format!("{}  ", action), Style::default().fg(Color::DarkGray)),
            ]
        })
        .collect();

    let status = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(Color::Black));

    frame.render_widget(status, area);
}
