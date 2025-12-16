use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsField {
    AnthropicKey,
    AnthropicModel,
    OpenAIKey,
    ExportPath,
}

impl SettingsField {
    pub fn next(&self) -> Self {
        match self {
            SettingsField::AnthropicKey => SettingsField::AnthropicModel,
            SettingsField::AnthropicModel => SettingsField::OpenAIKey,
            SettingsField::OpenAIKey => SettingsField::ExportPath,
            SettingsField::ExportPath => SettingsField::AnthropicKey,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            SettingsField::AnthropicKey => SettingsField::ExportPath,
            SettingsField::AnthropicModel => SettingsField::AnthropicKey,
            SettingsField::OpenAIKey => SettingsField::AnthropicModel,
            SettingsField::ExportPath => SettingsField::OpenAIKey,
        }
    }
}

pub struct SettingsState {
    pub anthropic_key: String,
    pub anthropic_model: String,
    pub openai_key: String,
    pub export_path: String,
    pub focused_field: SettingsField,
    pub cursor_pos: usize,
    pub has_changes: bool,
    pub test_result: Option<String>,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            anthropic_key: String::new(),
            anthropic_model: "claude-sonnet-4-20250514".to_string(),
            openai_key: String::new(),
            export_path: "~/.claude".to_string(),
            focused_field: SettingsField::AnthropicKey,
            cursor_pos: 0,
            has_changes: false,
            test_result: None,
        }
    }
}

impl SettingsState {
    pub fn current_field_value(&self) -> &str {
        match self.focused_field {
            SettingsField::AnthropicKey => &self.anthropic_key,
            SettingsField::AnthropicModel => &self.anthropic_model,
            SettingsField::OpenAIKey => &self.openai_key,
            SettingsField::ExportPath => &self.export_path,
        }
    }

    pub fn set_current_field(&mut self, value: String) {
        self.has_changes = true;
        match self.focused_field {
            SettingsField::AnthropicKey => self.anthropic_key = value,
            SettingsField::AnthropicModel => self.anthropic_model = value,
            SettingsField::OpenAIKey => self.openai_key = value,
            SettingsField::ExportPath => self.export_path = value,
        }
    }

    pub fn insert_char(&mut self, c: char) {
        let field_value = self.current_field_value().to_string();
        let mut chars: Vec<char> = field_value.chars().collect();
        chars.insert(self.cursor_pos.min(chars.len()), c);
        self.cursor_pos += 1;
        self.set_current_field(chars.into_iter().collect());
    }

    pub fn delete_char(&mut self) {
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

    /// Mask the API key for display
    fn mask_key(key: &str) -> String {
        if key.len() <= 8 {
            "*".repeat(key.len())
        } else {
            format!("{}...{}", &key[..4], &key[key.len()-4..])
        }
    }
}

pub fn draw(frame: &mut Frame, state: &SettingsState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Title bar
            Constraint::Min(0),     // Content
            Constraint::Length(1),  // Status bar
        ])
        .split(frame.area());

    // Title bar
    let title_bar = Paragraph::new(Line::from(vec![
        Span::styled(" Settings ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("                                                        "),
        Span::styled("[ESC] Back", Style::default().fg(Color::DarkGray)),
    ]));
    frame.render_widget(title_bar, chunks[0]);

    // Content
    draw_content(frame, chunks[1], state);

    // Status bar
    draw_status_bar(frame, chunks[2], state);
}

fn draw_content(frame: &mut Frame, area: Rect, state: &SettingsState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),  // Anthropic section
            Constraint::Length(4),  // OpenAI section
            Constraint::Length(4),  // Export section
            Constraint::Length(4),  // Data section
            Constraint::Min(0),     // Spacer
        ])
        .split(inner);

    // Anthropic section
    draw_section(frame, chunks[0], " LLM Configuration (Anthropic) ", &[
        ("API Key:  ", &SettingsState::mask_key(&state.anthropic_key), state.focused_field == SettingsField::AnthropicKey, state.cursor_pos),
        ("Model:    ", &state.anthropic_model, state.focused_field == SettingsField::AnthropicModel, state.cursor_pos),
    ]);

    // OpenAI section
    draw_section(frame, chunks[1], " OpenAI (optional) ", &[
        ("API Key:  ", &SettingsState::mask_key(&state.openai_key), state.focused_field == SettingsField::OpenAIKey, state.cursor_pos),
    ]);

    // Export section
    draw_section(frame, chunks[2], " Export Settings ", &[
        ("Path:     ", &state.export_path, state.focused_field == SettingsField::ExportPath, state.cursor_pos),
    ]);

    // Data section (read-only info)
    let data_block = Block::default()
        .title(" Data ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let data_inner = data_block.inner(chunks[3]);
    frame.render_widget(data_block, chunks[3]);

    let data_info = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Database: ", Style::default().fg(Color::Yellow)),
            Span::styled("~/.local/share/grimoire/grimoire.db", Style::default().fg(Color::DarkGray)),
        ]),
    ]);
    frame.render_widget(data_info, data_inner);

    // Test result
    if let Some(ref result) = state.test_result {
        let style = if result.starts_with("Success") {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Red)
        };
        let result_para = Paragraph::new(result.as_str()).style(style);
        frame.render_widget(result_para, chunks[4]);
    }
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
                Span::styled(cursor_char.to_string(), Style::default().bg(Color::White).fg(Color::Black)),
                Span::raw(after),
            ]));
        } else {
            lines.push(Line::from(vec![
                label_span,
                Span::raw(*value),
            ]));
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
                vec![Span::styled(format!(" {}", action), Style::default().fg(Color::Red))]
            } else {
                vec![
                    Span::styled(*key, Style::default().fg(Color::Yellow)),
                    Span::styled(format!("{}  ", action), Style::default().fg(Color::DarkGray)),
                ]
            }
        })
        .collect();

    let status = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(Color::Black));

    frame.render_widget(status, area);
}
