use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub struct ConfirmDialog {
    pub title: String,
    pub message: String,
    pub confirm_label: String,
    pub cancel_label: String,
    pub selected: bool, // true = confirm, false = cancel
}

impl ConfirmDialog {
    pub fn delete(item_name: &str) -> Self {
        Self {
            title: " Confirm Delete ".to_string(),
            message: format!("Are you sure you want to delete '{}'?", item_name),
            confirm_label: "Delete".to_string(),
            cancel_label: "Cancel".to_string(),
            selected: false, // Default to cancel
        }
    }

    pub fn discard_changes() -> Self {
        Self {
            title: " Unsaved Changes ".to_string(),
            message: "You have unsaved changes. Discard them?".to_string(),
            confirm_label: "Discard".to_string(),
            cancel_label: "Keep Editing".to_string(),
            selected: false,
        }
    }

    pub fn toggle_selection(&mut self) {
        self.selected = !self.selected;
    }
}

pub fn draw(frame: &mut Frame, dialog: &ConfirmDialog) {
    let area = centered_rect_fixed(50, 7, frame.area());

    // Clear the area behind the popup
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(dialog.title.as_str())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Message
            Constraint::Length(1), // Buttons
        ])
        .split(inner);

    // Message
    let message = Paragraph::new(dialog.message.as_str()).style(Style::default().fg(Color::White));
    frame.render_widget(message, chunks[0]);

    // Buttons
    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    let cancel_style = if !dialog.selected {
        Style::default()
            .bg(Color::DarkGray)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let confirm_style = if dialog.selected {
        Style::default()
            .bg(Color::Red)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Red)
    };

    let cancel_btn = Paragraph::new(Line::from(vec![Span::styled(
        format!(" [{}] ", dialog.cancel_label),
        cancel_style,
    )]));

    let confirm_btn = Paragraph::new(Line::from(vec![Span::styled(
        format!(" [{}] ", dialog.confirm_label),
        confirm_style,
    )]));

    frame.render_widget(cancel_btn, button_chunks[0]);
    frame.render_widget(confirm_btn, button_chunks[1]);
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
