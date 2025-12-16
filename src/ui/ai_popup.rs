use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiAction {
    ImprovePrompt,
    MakeConcise,
    AddExamples,
    CustomRequest,
}

impl AiAction {
    pub fn all() -> &'static [AiAction] {
        &[
            AiAction::ImprovePrompt,
            AiAction::MakeConcise,
            AiAction::AddExamples,
            AiAction::CustomRequest,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            AiAction::ImprovePrompt => "Improve this prompt",
            AiAction::MakeConcise => "Make it more concise",
            AiAction::AddExamples => "Add examples",
            AiAction::CustomRequest => "Custom request...",
        }
    }

    pub fn system_prompt(&self) -> &'static str {
        match self {
            AiAction::ImprovePrompt => {
                "You are an expert prompt engineer. Improve the following prompt to be clearer, \
                 more effective, and better structured. Maintain the original intent while \
                 enhancing clarity and specificity. Return only the improved prompt, no explanations."
            }
            AiAction::MakeConcise => {
                "You are an expert editor. Make the following prompt more concise while \
                 preserving all essential information and functionality. Remove redundancy \
                 and verbosity. Return only the revised prompt, no explanations."
            }
            AiAction::AddExamples => {
                "You are an expert prompt engineer. Add 2-3 concrete examples to the following \
                 prompt to better illustrate the expected behavior. The examples should be \
                 practical and relevant. Return only the enhanced prompt with examples, no explanations."
            }
            AiAction::CustomRequest => "",
        }
    }
}

#[derive(Default)]
pub struct AiPopupState {
    pub selected_action: usize,
    pub custom_input: String,
    pub cursor_pos: usize,
    pub is_loading: bool,
    pub loading_tick: usize,
    pub result: Option<String>,
    pub error: Option<String>,
}

impl AiPopupState {
    pub fn tick_loading(&mut self) {
        if self.is_loading {
            self.loading_tick = (self.loading_tick + 1) % 4;
        }
    }

    pub fn loading_spinner(&self) -> &'static str {
        match self.loading_tick {
            0 => "⠋",
            1 => "⠙",
            2 => "⠹",
            _ => "⠸",
        }
    }

    pub fn select_next(&mut self) {
        self.selected_action = (self.selected_action + 1) % AiAction::all().len();
    }

    pub fn select_prev(&mut self) {
        let len = AiAction::all().len();
        self.selected_action = (self.selected_action + len - 1) % len;
    }

    pub fn selected_action(&self) -> AiAction {
        AiAction::all()[self.selected_action]
    }

    pub fn is_custom(&self) -> bool {
        self.selected_action() == AiAction::CustomRequest
    }

    pub fn insert_char(&mut self, c: char) {
        self.custom_input.insert(self.cursor_pos, c);
        self.cursor_pos += 1;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_pos > 0 {
            self.custom_input.remove(self.cursor_pos - 1);
            self.cursor_pos -= 1;
        }
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

pub fn draw(frame: &mut Frame, state: &AiPopupState, content_preview: &str, has_llm: bool) {
    let area = centered_rect(50, 60, frame.area());

    // Clear the area behind the popup
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" AI Assistant ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Show warning if no LLM is configured
    if !has_llm {
        draw_no_llm_warning(frame, inner);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(6), // Actions
            Constraint::Length(3), // Custom input (if selected)
            Constraint::Min(3),    // Preview/Result
            Constraint::Length(1), // Status bar
        ])
        .split(inner);

    // Header
    let header = Paragraph::new("How can I help?").style(
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );
    frame.render_widget(header, chunks[0]);

    // Actions
    draw_actions(frame, chunks[1], state);

    // Custom input
    if state.is_custom() {
        draw_custom_input(frame, chunks[2], state);
    }

    // Result or loading indicator
    draw_result(frame, chunks[3], state, content_preview);

    // Status bar
    draw_status_bar(frame, chunks[4], state);
}

fn draw_no_llm_warning(frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let warning_icon = Paragraph::new("⚠")
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(warning_icon, chunks[1]);

    let message = Paragraph::new("No LLM API key configured")
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(message, chunks[2]);

    let hint = Paragraph::new("Go to Settings (s) to add an Anthropic or OpenAI API key")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(ratatui::layout::Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(hint, chunks[3]);

    let status = Paragraph::new(Line::from(vec![
        Span::styled("ESC ", Style::default().fg(Color::Yellow)),
        Span::styled("close", Style::default().fg(Color::DarkGray)),
    ]));
    frame.render_widget(status, chunks[5]);
}

fn draw_actions(frame: &mut Frame, area: Rect, state: &AiPopupState) {
    let mut lines = Vec::new();

    for (i, action) in AiAction::all().iter().enumerate() {
        let is_selected = i == state.selected_action;
        let prefix = if is_selected { "> " } else { "  " };

        let style = if is_selected {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        lines.push(Line::styled(format!("{}{}", prefix, action.label()), style));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

fn draw_custom_input(frame: &mut Frame, area: Rect, state: &AiPopupState) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chars: Vec<char> = state.custom_input.chars().collect();
    let cursor = state.cursor_pos.min(chars.len());
    let before: String = chars.iter().take(cursor).collect();
    let cursor_char = chars.get(cursor).copied().unwrap_or(' ');
    let after: String = chars.iter().skip(cursor + 1).collect();

    let line = Line::from(vec![
        Span::raw(before),
        Span::styled(
            cursor_char.to_string(),
            Style::default().bg(Color::White).fg(Color::Black),
        ),
        Span::raw(after),
    ]);

    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, inner);
}

fn draw_result(frame: &mut Frame, area: Rect, state: &AiPopupState, content_preview: &str) {
    let title = if state.is_loading {
        format!(" {} Processing... ", state.loading_spinner())
    } else {
        " Preview ".to_string()
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if state.is_loading {
            Color::Yellow
        } else {
            Color::DarkGray
        }));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let content = if state.is_loading {
        Paragraph::new("Waiting for AI response...").style(Style::default().fg(Color::Yellow))
    } else if let Some(ref error) = state.error {
        Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red))
            .wrap(Wrap { trim: true })
    } else if let Some(ref result) = state.result {
        Paragraph::new(result.as_str())
            .style(Style::default().fg(Color::Green))
            .wrap(Wrap { trim: true })
    } else {
        // Show content preview
        let preview = if content_preview.len() > 200 {
            format!("{}...", &content_preview[..200])
        } else {
            content_preview.to_string()
        };
        Paragraph::new(preview)
            .style(Style::default().fg(Color::DarkGray))
            .wrap(Wrap { trim: true })
    };

    frame.render_widget(content, inner);
}

fn draw_status_bar(frame: &mut Frame, area: Rect, state: &AiPopupState) {
    let shortcuts = if state.is_loading {
        vec![("", "Processing...")]
    } else if state.result.is_some() {
        vec![("Enter ", "apply"), ("ESC ", "cancel")]
    } else {
        vec![("j/k ", "select"), ("Enter ", "run"), ("ESC ", "close")]
    };

    let spans: Vec<Span> = shortcuts
        .iter()
        .flat_map(|(key, action)| {
            if key.is_empty() {
                vec![Span::styled(*action, Style::default().fg(Color::Yellow))]
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

    let paragraph = Paragraph::new(Line::from(spans));
    frame.render_widget(paragraph, area);
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
