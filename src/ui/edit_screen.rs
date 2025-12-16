pub use crate::models::{Category, Item};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditField {
    Name,
    Category,
    Tags,
    Model,
    Tools,
    Description,
    Content,
}

impl EditField {
    pub fn next(&self, category: Category) -> Self {
        match self {
            EditField::Name => EditField::Category,
            EditField::Category => EditField::Tags,
            EditField::Tags => match category {
                Category::Agent | Category::Command => EditField::Model,
                Category::Skill => EditField::Tools,
                Category::Prompt => EditField::Description,
            },
            EditField::Model => EditField::Tools,
            EditField::Tools => EditField::Description,
            EditField::Description => EditField::Content,
            EditField::Content => EditField::Name,
        }
    }

    pub fn prev(&self, category: Category) -> Self {
        match self {
            EditField::Name => EditField::Content,
            EditField::Category => EditField::Name,
            EditField::Tags => EditField::Category,
            EditField::Model => EditField::Tags,
            EditField::Tools => match category {
                Category::Agent | Category::Command => EditField::Model,
                _ => EditField::Tags,
            },
            EditField::Description => match category {
                Category::Agent | Category::Command => EditField::Tools,
                Category::Skill => EditField::Tools,
                Category::Prompt => EditField::Tags,
            },
            EditField::Content => EditField::Description,
        }
    }
}

pub struct EditState {
    pub item: Item,
    pub is_new: bool,
    pub focused_field: EditField,
    pub cursor_pos: usize,
    pub has_changes: bool,
    pub content_scroll: u16,
    pub show_category_dropdown: bool,
    pub category_dropdown_index: usize,
}

impl EditState {
    pub fn new_item() -> Self {
        Self {
            item: Item::default(),
            is_new: true,
            focused_field: EditField::Name,
            cursor_pos: 0,
            has_changes: false,
            content_scroll: 0,
            show_category_dropdown: false,
            category_dropdown_index: 0,
        }
    }

    pub fn edit_item(item: Item) -> Self {
        let cursor_pos = item.name.len();
        let category_index = Category::all()
            .iter()
            .position(|c| *c == item.category)
            .unwrap_or(0);
        Self {
            item,
            is_new: false,
            focused_field: EditField::Name,
            cursor_pos,
            has_changes: false,
            content_scroll: 0,
            show_category_dropdown: false,
            category_dropdown_index: category_index,
        }
    }

    pub fn open_category_dropdown(&mut self) {
        self.category_dropdown_index = Category::all()
            .iter()
            .position(|c| *c == self.item.category)
            .unwrap_or(0);
        self.show_category_dropdown = true;
    }

    pub fn select_category_from_dropdown(&mut self) {
        self.item.category = Category::all()[self.category_dropdown_index];
        self.show_category_dropdown = false;
        self.has_changes = true;
    }

    pub fn dropdown_next(&mut self) {
        self.category_dropdown_index = (self.category_dropdown_index + 1) % Category::all().len();
    }

    pub fn dropdown_prev(&mut self) {
        let len = Category::all().len();
        self.category_dropdown_index = (self.category_dropdown_index + len - 1) % len;
    }

    pub fn current_field_value(&self) -> &str {
        match self.focused_field {
            EditField::Name => &self.item.name,
            EditField::Category => self.item.category.as_str(),
            EditField::Tags => self.item.tags.as_deref().unwrap_or(""),
            EditField::Model => self.item.model.as_deref().unwrap_or(""),
            EditField::Tools => self.item.tools.as_deref()
                .or(self.item.allowed_tools.as_deref())
                .unwrap_or(""),
            EditField::Description => self.item.description.as_deref().unwrap_or(""),
            EditField::Content => &self.item.content,
        }
    }

    pub fn set_current_field(&mut self, value: String) {
        self.has_changes = true;
        match self.focused_field {
            EditField::Name => self.item.name = value,
            EditField::Category => self.item.category = Category::from_str(&value),
            EditField::Tags => self.item.tags = if value.is_empty() { None } else { Some(value) },
            EditField::Model => self.item.model = if value.is_empty() { None } else { Some(value) },
            EditField::Tools => {
                let val = if value.is_empty() { None } else { Some(value) };
                match self.item.category {
                    Category::Agent => self.item.tools = val,
                    Category::Skill | Category::Command => self.item.allowed_tools = val,
                    _ => {}
                }
            }
            EditField::Description => self.item.description = if value.is_empty() { None } else { Some(value) },
            EditField::Content => self.item.content = value,
        }
    }

    pub fn insert_char(&mut self, c: char) {
        let field_value = self.current_field_value().to_string();
        let mut chars: Vec<char> = field_value.chars().collect();

        if self.cursor_pos > chars.len() {
            self.cursor_pos = chars.len();
        }

        chars.insert(self.cursor_pos, c);
        self.cursor_pos += 1;
        self.set_current_field(chars.into_iter().collect());
    }

    pub fn delete_char(&mut self) {
        let field_value = self.current_field_value().to_string();
        let mut chars: Vec<char> = field_value.chars().collect();

        if self.cursor_pos > 0 && !chars.is_empty() {
            chars.remove(self.cursor_pos - 1);
            self.cursor_pos -= 1;
            self.set_current_field(chars.into_iter().collect());
        }
    }

    pub fn delete_char_forward(&mut self) {
        let field_value = self.current_field_value().to_string();
        let mut chars: Vec<char> = field_value.chars().collect();

        if self.cursor_pos < chars.len() {
            chars.remove(self.cursor_pos);
            self.set_current_field(chars.into_iter().collect());
        }
    }

    pub fn move_cursor_left(&mut self) {
        self.cursor_pos = self.cursor_pos.saturating_sub(1);
    }

    pub fn move_cursor_right(&mut self) {
        let len = self.current_field_value().chars().count();
        self.cursor_pos = (self.cursor_pos + 1).min(len);
    }

    pub fn move_cursor_start(&mut self) {
        self.cursor_pos = 0;
    }

    pub fn move_cursor_end(&mut self) {
        self.cursor_pos = self.current_field_value().chars().count();
    }

    pub fn next_field(&mut self) {
        self.focused_field = self.focused_field.next(self.item.category);
        self.cursor_pos = self.current_field_value().chars().count();
    }

    pub fn prev_field(&mut self) {
        self.focused_field = self.focused_field.prev(self.item.category);
        self.cursor_pos = self.current_field_value().chars().count();
    }
}

pub fn draw(frame: &mut Frame, state: &EditState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Title bar
            Constraint::Length(7),  // Form fields (top section)
            Constraint::Length(6),  // Description
            Constraint::Min(0),     // Content
            Constraint::Length(1),  // Status bar
        ])
        .split(frame.area());

    // Title bar
    let title = if state.is_new {
        format!(" New {} ", state.item.category.display_name())
    } else {
        format!(" Edit {}: {} ", state.item.category.display_name(), state.item.name)
    };
    let title_bar = Paragraph::new(Line::from(vec![
        Span::styled(title, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("                                                        "),
        Span::styled("[ESC] Cancel", Style::default().fg(Color::DarkGray)),
    ]));
    frame.render_widget(title_bar, chunks[0]);

    // Form fields (returns category field rect for dropdown positioning)
    let category_field_rect = draw_form_fields(frame, chunks[1], state);

    // Description field
    draw_description_field(frame, chunks[2], state);

    // Content field
    draw_content_field(frame, chunks[3], state);

    // Status bar
    draw_status_bar(frame, chunks[4], state);

    // Draw dropdown LAST so it appears on top of everything
    if state.show_category_dropdown {
        draw_category_dropdown(frame, category_field_rect, state);
    }
}

fn draw_form_fields(frame: &mut Frame, area: Rect, state: &EditState) -> Rect {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let field_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    // Name field
    draw_field(frame, field_chunks[0], "Name:     ", &state.item.name,
               state.focused_field == EditField::Name, state.cursor_pos);

    // Category field (with dropdown indicator)
    let cat_display = format!("[{}] ▼", state.item.category.display_name());
    draw_field(frame, field_chunks[1], "Category: ", &cat_display,
               state.focused_field == EditField::Category, 0);

    // Tags field
    draw_field(frame, field_chunks[2], "Tags:     ",
               state.item.tags.as_deref().unwrap_or(""),
               state.focused_field == EditField::Tags, state.cursor_pos);

    // Category-specific fields
    match state.item.category {
        Category::Agent | Category::Command => {
            draw_field(frame, field_chunks[3], "Model:    ",
                       state.item.model.as_deref().unwrap_or(""),
                       state.focused_field == EditField::Model, state.cursor_pos);

            let tools = state.item.tools.as_deref()
                .or(state.item.allowed_tools.as_deref())
                .unwrap_or("");
            draw_field(frame, field_chunks[4], "Tools:    ", tools,
                       state.focused_field == EditField::Tools, state.cursor_pos);
        }
        Category::Skill => {
            let tools = state.item.allowed_tools.as_deref().unwrap_or("");
            draw_field(frame, field_chunks[3], "Tools:    ", tools,
                       state.focused_field == EditField::Tools, state.cursor_pos);
        }
        Category::Prompt => {}
    }

    // Return category field rect for dropdown positioning
    field_chunks[1]
}

fn draw_field(frame: &mut Frame, area: Rect, label: &str, value: &str, focused: bool, cursor: usize) {
    let style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let label_span = Span::styled(label, Style::default().fg(Color::Yellow));

    let value_display = if focused {
        // Show cursor
        let chars: Vec<char> = value.chars().collect();
        let before: String = chars.iter().take(cursor).collect();
        let cursor_char = chars.get(cursor).copied().unwrap_or(' ');
        let after: String = chars.iter().skip(cursor + 1).collect();

        vec![
            label_span,
            Span::raw(before),
            Span::styled(cursor_char.to_string(), Style::default().bg(Color::White).fg(Color::Black)),
            Span::raw(after),
        ]
    } else {
        vec![
            label_span,
            Span::styled(value, style),
        ]
    };

    let line = Line::from(value_display);
    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}

fn draw_description_field(frame: &mut Frame, area: Rect, state: &EditState) {
    let focused = state.focused_field == EditField::Description;
    let border_color = if focused { Color::Cyan } else { Color::DarkGray };

    let required = match state.item.category {
        Category::Agent | Category::Skill => " (required)",
        _ => " (optional)",
    };

    let block = Block::default()
        .title(format!(" Description{} ", required))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let desc = state.item.description.as_deref().unwrap_or("");
    let paragraph = if focused {
        // Show with cursor
        let chars: Vec<char> = desc.chars().collect();
        let cursor = state.cursor_pos.min(chars.len());
        let before: String = chars.iter().take(cursor).collect();
        let cursor_char = chars.get(cursor).copied().unwrap_or(' ');
        let after: String = chars.iter().skip(cursor + 1).collect();

        Paragraph::new(Line::from(vec![
            Span::raw(before),
            Span::styled(cursor_char.to_string(), Style::default().bg(Color::White).fg(Color::Black)),
            Span::raw(after),
        ]))
    } else {
        Paragraph::new(desc)
    };

    frame.render_widget(paragraph.wrap(Wrap { trim: false }), inner);
}

fn draw_content_field(frame: &mut Frame, area: Rect, state: &EditState) {
    let focused = state.focused_field == EditField::Content;
    let border_color = if focused { Color::Cyan } else { Color::DarkGray };

    let block = Block::default()
        .title(" Content (required) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let content = &state.item.content;
    let paragraph = if focused {
        // Show with cursor
        let chars: Vec<char> = content.chars().collect();
        let cursor = state.cursor_pos.min(chars.len());
        let before: String = chars.iter().take(cursor).collect();
        let cursor_char = chars.get(cursor).copied().unwrap_or(' ');
        let after: String = chars.iter().skip(cursor + 1).collect();

        Paragraph::new(Line::from(vec![
            Span::raw(before),
            Span::styled(cursor_char.to_string(), Style::default().bg(Color::White).fg(Color::Black)),
            Span::raw(after),
        ]))
    } else {
        Paragraph::new(content.as_str())
    };

    frame.render_widget(paragraph.wrap(Wrap { trim: false }).scroll((state.content_scroll, 0)), inner);
}

fn draw_status_bar(frame: &mut Frame, area: Rect, state: &EditState) {
    // Show dropdown-specific shortcuts when dropdown is open
    if state.show_category_dropdown {
        let shortcuts = [
            ("j/k ", "navigate"),
            ("Enter ", "select"),
            ("ESC ", "close"),
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
        return;
    }

    let mut shortcuts = vec![
        ("Tab ", "next"),
        ("S-Tab ", "prev"),
    ];

    if state.focused_field == EditField::Category {
        shortcuts.push(("Enter ", "select category"));
    } else if state.focused_field == EditField::Content || state.focused_field == EditField::Description {
        shortcuts.push(("C-a ", "ai-assist"));
    }

    shortcuts.push(("Ctrl+S ", "save"));
    shortcuts.push(("ESC ", "cancel"));

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

fn draw_category_dropdown(frame: &mut Frame, anchor: Rect, state: &EditState) {
    // Position dropdown below the category field
    let dropdown_area = Rect {
        x: anchor.x + 10, // After "Category: "
        y: anchor.y + 1,
        width: 15,
        height: 6, // 4 items + 2 for border
    };

    // Clear the area behind dropdown
    frame.render_widget(Clear, dropdown_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(dropdown_area);
    frame.render_widget(block, dropdown_area);

    // Draw category options
    let mut lines = Vec::new();
    for (i, category) in Category::all().iter().enumerate() {
        let is_selected = i == state.category_dropdown_index;
        let prefix = if is_selected { "> " } else { "  " };

        let style = if is_selected {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        lines.push(Line::styled(format!("{}{}", prefix, category.display_name()), style));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}
