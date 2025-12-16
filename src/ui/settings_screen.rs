use crate::db::Database;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LlmProvider {
    #[default]
    Anthropic,
    OpenAI,
}

impl LlmProvider {
    pub fn all() -> &'static [LlmProvider] {
        &[LlmProvider::Anthropic, LlmProvider::OpenAI]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            LlmProvider::Anthropic => "Anthropic",
            LlmProvider::OpenAI => "OpenAI",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "openai" => LlmProvider::OpenAI,
            _ => LlmProvider::Anthropic,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsField {
    Provider,
    ApiKey,
    Model,
    ExportPath,
}

impl SettingsField {
    pub fn next(&self) -> Self {
        match self {
            SettingsField::Provider => SettingsField::ApiKey,
            SettingsField::ApiKey => SettingsField::Model,
            SettingsField::Model => SettingsField::ExportPath,
            SettingsField::ExportPath => SettingsField::Provider,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            SettingsField::Provider => SettingsField::ExportPath,
            SettingsField::ApiKey => SettingsField::Provider,
            SettingsField::Model => SettingsField::ApiKey,
            SettingsField::ExportPath => SettingsField::Model,
        }
    }
}

pub struct SettingsState {
    pub provider: LlmProvider,
    pub api_key: String,
    pub llm_model: String,
    pub export_path: String,
    pub focused_field: SettingsField,
    pub cursor_pos: usize,
    pub has_changes: bool,
    pub show_provider_dropdown: bool,
    pub provider_dropdown_index: usize,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            provider: LlmProvider::Anthropic,
            api_key: String::new(),
            llm_model: "claude-sonnet-4-20250514".to_string(),
            export_path: "~/.claude".to_string(),
            focused_field: SettingsField::Provider,
            cursor_pos: 0,
            has_changes: false,
            show_provider_dropdown: false,
            provider_dropdown_index: 0,
        }
    }
}

impl SettingsState {
    pub fn current_field_value(&self) -> &str {
        match self.focused_field {
            SettingsField::Provider => self.provider.display_name(),
            SettingsField::ApiKey => &self.api_key,
            SettingsField::Model => &self.llm_model,
            SettingsField::ExportPath => &self.export_path,
        }
    }

    fn set_current_field(&mut self, value: String) {
        self.has_changes = true;
        match self.focused_field {
            SettingsField::Provider => {} // Handled by dropdown
            SettingsField::ApiKey => self.api_key = value,
            SettingsField::Model => self.llm_model = value,
            SettingsField::ExportPath => self.export_path = value,
        }
    }

    pub fn insert_char(&mut self, c: char) {
        if self.focused_field == SettingsField::Provider {
            return;
        }
        let field_value = self.current_field_value().to_string();
        let mut chars: Vec<char> = field_value.chars().collect();
        chars.insert(self.cursor_pos.min(chars.len()), c);
        self.cursor_pos += 1;
        self.set_current_field(chars.into_iter().collect());
    }

    pub fn insert_str(&mut self, s: &str) {
        if self.focused_field == SettingsField::Provider {
            return;
        }
        // Filter out newlines and other control characters
        let clean: String = s.chars().filter(|c| !c.is_control()).collect();
        let field_value = self.current_field_value().to_string();
        let mut chars: Vec<char> = field_value.chars().collect();
        let insert_pos = self.cursor_pos.min(chars.len());
        for (i, c) in clean.chars().enumerate() {
            chars.insert(insert_pos + i, c);
        }
        self.cursor_pos += clean.chars().count();
        self.set_current_field(chars.into_iter().collect());
    }

    pub fn delete_char(&mut self) {
        if self.focused_field == SettingsField::Provider {
            return;
        }
        if self.cursor_pos > 0 {
            let field_value = self.current_field_value().to_string();
            let mut chars: Vec<char> = field_value.chars().collect();
            chars.remove(self.cursor_pos - 1);
            self.cursor_pos -= 1;
            self.set_current_field(chars.into_iter().collect());
        }
    }

    pub fn next_field(&mut self) {
        self.focused_field = self.focused_field.next();
        self.cursor_pos = self.current_field_value().chars().count();
    }

    pub fn prev_field(&mut self) {
        self.focused_field = self.focused_field.prev();
        self.cursor_pos = self.current_field_value().chars().count();
    }

    pub fn open_provider_dropdown(&mut self) {
        self.show_provider_dropdown = true;
        self.provider_dropdown_index = LlmProvider::all()
            .iter()
            .position(|p| *p == self.provider)
            .unwrap_or(0);
    }

    pub fn select_provider_from_dropdown(&mut self) {
        if let Some(provider) = LlmProvider::all().get(self.provider_dropdown_index) {
            self.provider = *provider;
            self.has_changes = true;
        }
        self.show_provider_dropdown = false;
    }

    pub fn dropdown_next(&mut self) {
        let max = LlmProvider::all().len();
        self.provider_dropdown_index = (self.provider_dropdown_index + 1) % max;
    }

    pub fn dropdown_prev(&mut self) {
        let max = LlmProvider::all().len();
        self.provider_dropdown_index = (self.provider_dropdown_index + max - 1) % max;
    }

    /// Mask the API key for display
    pub fn mask_key(key: &str) -> String {
        if key.is_empty() {
            String::new()
        } else if key.len() <= 8 {
            "*".repeat(key.len())
        } else {
            format!("{}...{}", &key[..4], &key[key.len() - 4..])
        }
    }
}

pub fn draw(frame: &mut Frame, state: &SettingsState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title bar
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Status bar
        ])
        .split(frame.area());

    // Title bar
    let title_bar = Paragraph::new(Line::from(vec![
        Span::styled(
            " Settings ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("                                                        "),
        Span::styled("[ESC] Back", Style::default().fg(Color::DarkGray)),
    ]));
    frame.render_widget(title_bar, chunks[0]);

    // Content
    let content_area = draw_content(frame, chunks[1], state);

    // Status bar
    draw_status_bar(frame, chunks[2], state);

    // Draw dropdown overlay last (on top)
    if state.show_provider_dropdown {
        draw_provider_dropdown(frame, content_area, state);
    }
}

fn draw_content(frame: &mut Frame, area: Rect, state: &SettingsState) -> Rect {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7), // LLM section
            Constraint::Length(4), // Export section
            Constraint::Length(4), // Data section
            Constraint::Min(0),    // Spacer
        ])
        .split(inner);

    // LLM Configuration section
    draw_llm_section(frame, chunks[0], state);

    // Export section
    draw_section(
        frame,
        chunks[1],
        " Export Settings ",
        &[(
            "Path:     ",
            &state.export_path,
            state.focused_field == SettingsField::ExportPath,
            state.cursor_pos,
        )],
    );

    // Data section (read-only info)
    let data_block = Block::default()
        .title(" Data ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let data_inner = data_block.inner(chunks[2]);
    frame.render_widget(data_block, chunks[2]);

    let db_path = Database::db_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    let data_info = Paragraph::new(vec![Line::from(vec![
        Span::styled("Database: ", Style::default().fg(Color::Yellow)),
        Span::styled(db_path, Style::default().fg(Color::DarkGray)),
    ])]);
    frame.render_widget(data_info, data_inner);

    // Return the LLM section area for dropdown positioning
    chunks[0]
}

fn draw_llm_section(frame: &mut Frame, area: Rect, state: &SettingsState) {
    let block = Block::default()
        .title(" LLM Configuration ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = Vec::new();

    // Provider field
    let provider_focused = state.focused_field == SettingsField::Provider;
    let provider_style = if provider_focused {
        Style::default().bg(Color::DarkGray)
    } else {
        Style::default()
    };
    lines.push(Line::from(vec![
        Span::styled("Provider: ", Style::default().fg(Color::Yellow)),
        Span::styled(
            format!("[{}]", state.provider.display_name()),
            provider_style,
        ),
        Span::styled(" ▼", Style::default().fg(Color::DarkGray)),
    ]));

    // API Key field
    let api_key_focused = state.focused_field == SettingsField::ApiKey;
    let masked_key = SettingsState::mask_key(&state.api_key);
    if api_key_focused {
        let chars: Vec<char> = state.api_key.chars().collect();
        let cursor_pos = state.cursor_pos.min(chars.len());
        let before: String = chars.iter().take(cursor_pos).collect();
        let cursor_char = chars.get(cursor_pos).copied().unwrap_or(' ');
        let after: String = chars.iter().skip(cursor_pos + 1).collect();

        lines.push(Line::from(vec![
            Span::styled("API Key:  ", Style::default().fg(Color::Yellow)),
            Span::raw(before),
            Span::styled(
                cursor_char.to_string(),
                Style::default().bg(Color::White).fg(Color::Black),
            ),
            Span::raw(after),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::styled("API Key:  ", Style::default().fg(Color::Yellow)),
            Span::raw(masked_key),
        ]));
    }

    // Model field (only show for Anthropic)
    if state.provider == LlmProvider::Anthropic {
        let model_focused = state.focused_field == SettingsField::Model;
        if model_focused {
            let chars: Vec<char> = state.llm_model.chars().collect();
            let cursor_pos = state.cursor_pos.min(chars.len());
            let before: String = chars.iter().take(cursor_pos).collect();
            let cursor_char = chars.get(cursor_pos).copied().unwrap_or(' ');
            let after: String = chars.iter().skip(cursor_pos + 1).collect();

            lines.push(Line::from(vec![
                Span::styled("Model:    ", Style::default().fg(Color::Yellow)),
                Span::raw(before),
                Span::styled(
                    cursor_char.to_string(),
                    Style::default().bg(Color::White).fg(Color::Black),
                ),
                Span::raw(after),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled("Model:    ", Style::default().fg(Color::Yellow)),
                Span::raw(&state.llm_model),
            ]));
        }
    } else {
        // Show placeholder for OpenAI
        lines.push(Line::from(vec![
            Span::styled("Model:    ", Style::default().fg(Color::DarkGray)),
            Span::styled("(uses gpt-4o)", Style::default().fg(Color::DarkGray)),
        ]));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn draw_provider_dropdown(frame: &mut Frame, anchor: Rect, state: &SettingsState) {
    let dropdown_area = Rect {
        x: anchor.x + 12,
        y: anchor.y + 2,
        width: 15,
        height: 4,
    };

    frame.render_widget(Clear, dropdown_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(dropdown_area);
    frame.render_widget(block, dropdown_area);

    let mut lines = Vec::new();
    for (i, provider) in LlmProvider::all().iter().enumerate() {
        let is_selected = i == state.provider_dropdown_index;
        let style = if is_selected {
            Style::default().bg(Color::Cyan).fg(Color::Black)
        } else {
            Style::default()
        };
        lines.push(Line::styled(
            format!(" {} ", provider.display_name()),
            style,
        ));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn draw_section(frame: &mut Frame, area: Rect, title: &str, fields: &[(&str, &str, bool, usize)]) {
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = Vec::new();
    for (label, value, focused, cursor) in fields {
        let label_span = Span::styled(*label, Style::default().fg(Color::Yellow));

        if *focused {
            let chars: Vec<char> = value.chars().collect();
            let cursor_pos = (*cursor).min(chars.len());
            let before: String = chars.iter().take(cursor_pos).collect();
            let cursor_char = chars.get(cursor_pos).copied().unwrap_or(' ');
            let after: String = chars.iter().skip(cursor_pos + 1).collect();

            lines.push(Line::from(vec![
                label_span,
                Span::raw(before),
                Span::styled(
                    cursor_char.to_string(),
                    Style::default().bg(Color::White).fg(Color::Black),
                ),
                Span::raw(after),
            ]));
        } else {
            lines.push(Line::from(vec![label_span, Span::raw(*value)]));
        }
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn draw_status_bar(frame: &mut Frame, area: Rect, state: &SettingsState) {
    let mut shortcuts = vec![
        ("Tab ", "next"),
        ("S-Tab ", "prev"),
        ("Ctrl+S ", "save"),
        ("ESC ", "back"),
    ];

    if state.has_changes {
        shortcuts.push(("", "[unsaved]"));
    }

    let spans: Vec<Span> = shortcuts
        .iter()
        .flat_map(|(key, action)| {
            if key.is_empty() {
                vec![Span::styled(
                    format!(" {}", action),
                    Style::default().fg(Color::Red),
                )]
            } else {
                vec![
                    Span::styled(*key, Style::default().fg(Color::Yellow)),
                    Span::styled(
                        format!("{}  ", action),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]
            }
        })
        .collect();

    let status = Paragraph::new(Line::from(spans)).style(Style::default().bg(Color::Black));

    frame.render_widget(status, area);
}
