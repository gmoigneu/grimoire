use crate::models::Item;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState},
    Frame,
};

#[derive(Default)]
pub struct SearchState {
    pub query: String,
    pub cursor_pos: usize,
    pub results: Vec<Item>,
    pub selected_index: usize,
}


impl SearchState {
    pub fn insert_char(&mut self, c: char) {
        self.query.insert(self.cursor_pos, c);
        self.cursor_pos += 1;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_pos > 0 {
            self.query.remove(self.cursor_pos - 1);
            self.cursor_pos -= 1;
        }
    }

    pub fn move_cursor_left(&mut self) {
        self.cursor_pos = self.cursor_pos.saturating_sub(1);
    }

    pub fn move_cursor_right(&mut self) {
        self.cursor_pos = (self.cursor_pos + 1).min(self.query.len());
    }

    pub fn clear(&mut self) {
        self.query.clear();
        self.cursor_pos = 0;
        self.results.clear();
        self.selected_index = 0;
    }

    pub fn select_next(&mut self) {
        if !self.results.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.results.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.results.is_empty() {
            self.selected_index = self.selected_index.checked_sub(1).unwrap_or(self.results.len() - 1);
        }
    }

    pub fn selected_item(&self) -> Option<&Item> {
        self.results.get(self.selected_index)
    }
}

pub fn draw(frame: &mut Frame, state: &SearchState) {
    let area = centered_rect(70, 60, frame.area());

    // Clear the area behind the popup
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Search ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Search input
            Constraint::Min(0),     // Results
            Constraint::Length(1),  // Status bar
        ])
        .split(inner);

    // Search input
    draw_search_input(frame, chunks[0], state);

    // Results
    draw_results(frame, chunks[1], state);

    // Status bar
    draw_status_bar(frame, chunks[2]);
}

fn draw_search_input(frame: &mut Frame, area: Rect, state: &SearchState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Build query with cursor
    let chars: Vec<char> = state.query.chars().collect();
    let cursor = state.cursor_pos.min(chars.len());
    let before: String = chars.iter().take(cursor).collect();
    let cursor_char = chars.get(cursor).copied().unwrap_or(' ');
    let after: String = chars.iter().skip(cursor + 1).collect();

    let line = Line::from(vec![
        Span::styled("/ ", Style::default().fg(Color::Yellow)),
        Span::raw(before),
        Span::styled(cursor_char.to_string(), Style::default().bg(Color::White).fg(Color::Black)),
        Span::raw(after),
    ]);

    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, inner);
}

fn draw_results(frame: &mut Frame, area: Rect, state: &SearchState) {
    if state.results.is_empty() {
        let msg = if state.query.is_empty() {
            "Type to search..."
        } else {
            "No results found"
        };
        let paragraph = Paragraph::new(msg)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(paragraph, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from("NAME").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("CATEGORY").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("TAGS").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ]);

    let rows: Vec<Row> = state
        .results
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == state.selected_index {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            Row::new(vec![
                Cell::from(item.name.clone()),
                Cell::from(item.category.display_name()),
                Cell::from(item.tags.clone().unwrap_or_default())
                    .style(Style::default().fg(Color::DarkGray)),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Min(20),
        Constraint::Length(10),
        Constraint::Min(15),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .row_highlight_style(Style::default().bg(Color::DarkGray));

    let mut table_state = TableState::default();
    table_state.select(Some(state.selected_index));

    frame.render_stateful_widget(table, area, &mut table_state);
}

fn draw_status_bar(frame: &mut Frame, area: Rect) {
    let shortcuts = [("j/k ", "navigate"),
        ("Enter ", "select"),
        ("c ", "copy"),
        ("ESC ", "close")];

    let spans: Vec<Span> = shortcuts
        .iter()
        .flat_map(|(key, action)| {
            vec![
                Span::styled(*key, Style::default().fg(Color::Yellow)),
                Span::styled(format!("{}  ", action), Style::default().fg(Color::DarkGray)),
            ]
        })
        .collect();

    let paragraph = Paragraph::new(Line::from(spans));
    frame.render_widget(paragraph, area);
}

/// Helper function to create a centered rect
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
