use crate::db::ItemVersion;
use chrono::{NaiveDateTime, Utc};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

pub struct HistoryState {
    pub versions: Vec<ItemVersion>,
    pub list_state: ListState,
    pub item_name: String,
}

impl HistoryState {
    pub fn new(versions: Vec<ItemVersion>, item_name: String) -> Self {
        let mut list_state = ListState::default();
        if !versions.is_empty() {
            list_state.select(Some(0));
        }
        Self {
            versions,
            list_state,
            item_name,
        }
    }

    pub fn selected_version(&self) -> Option<&ItemVersion> {
        self.list_state
            .selected()
            .and_then(|i| self.versions.get(i))
    }

    pub fn select_next(&mut self) {
        if self.versions.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.versions.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn select_previous(&mut self) {
        if self.versions.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.versions.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }
}

pub fn draw(frame: &mut Frame, state: &mut HistoryState) {
    let popup_height = (state.versions.len() as u16 + 5).clamp(7, 15);
    let area = centered_rect_fixed(50, popup_height, frame.area());

    // Clear the area behind the popup
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" History: {} ", state.item_name))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // Version list
            Constraint::Length(1), // Footer with keybindings
        ])
        .split(inner);

    // Version list
    let items: Vec<ListItem> = state
        .versions
        .iter()
        .map(|v| {
            let formatted_date = format_datetime(&v.created_at);
            let label = if v.is_current {
                format!("v{}  {}  (latest)", v.version, formatted_date)
            } else {
                format!("v{}  {}", v.version, formatted_date)
            };
            ListItem::new(Line::from(label))
        })
        .collect();

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, chunks[0], &mut state.list_state);

    // Footer
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(" view  "),
        Span::styled("r", Style::default().fg(Color::Yellow)),
        Span::raw(" restore  "),
        Span::styled("ESC", Style::default().fg(Color::Yellow)),
        Span::raw(" close"),
    ]))
    .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(footer, chunks[1]);
}

fn format_datetime(s: &str) -> String {
    // Parse SQLite datetime format and format nicely
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        let now = Utc::now().naive_utc();
        let duration = now.signed_duration_since(dt);

        let relative = if duration.num_days() > 7 {
            format!("{} weeks ago", duration.num_weeks())
        } else if duration.num_days() > 0 {
            let days = duration.num_days();
            if days == 1 {
                "1 day ago".to_string()
            } else {
                format!("{} days ago", days)
            }
        } else if duration.num_hours() > 0 {
            let hours = duration.num_hours();
            if hours == 1 {
                "1 hour ago".to_string()
            } else {
                format!("{} hours ago", hours)
            }
        } else if duration.num_minutes() > 0 {
            let mins = duration.num_minutes();
            if mins == 1 {
                "1 min ago".to_string()
            } else {
                format!("{} mins ago", mins)
            }
        } else {
            "just now".to_string()
        };

        relative
    } else {
        s.to_string()
    }
}

fn centered_rect_fixed(percent_x: u16, height: u16, r: Rect) -> Rect {
    // Center vertically with fixed height
    let vertical_padding = r.height.saturating_sub(height) / 2;
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(vertical_padding),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(r);

    // Center horizontally with percentage width
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
