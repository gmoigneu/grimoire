use crate::db::{Database, ItemStore, SettingsStore};
use crate::export::ClaudeExporter;
use crate::llm::{complete_sync, LlmRequest};
use crate::models::{Category, Item};
use crate::ui::{
    AiPopupState, ConfirmDialog, EditField, EditState, HelpState, SearchState, SettingsState, ViewState,
};
use color_eyre::eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Main,
    View,
    Edit,
    Search,
    Settings,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Sidebar,
    ItemList,
}

pub struct App {
    pub should_quit: bool,
    pub screen: Screen,
    pub focus: Focus,

    // Database
    pub db: Database,

    // Data
    pub items: Vec<Item>,
    pub category_counts: Vec<(Category, usize)>,
    pub tags: Vec<(String, usize)>,

    // Selection state
    pub selected_category: Option<Category>,
    pub selected_tag: Option<String>,
    pub selected_item_index: usize,
    pub sidebar_index: usize,

    // Vim-style key state
    pub pending_key: Option<char>,

    // Screen states
    pub view_state: ViewState,
    pub edit_state: EditState,
    pub search_state: SearchState,
    pub settings_state: SettingsState,
    pub help_state: HelpState,

    // Overlays
    pub confirm_dialog: Option<ConfirmDialog>,
    pub show_ai_popup: bool,
    pub ai_popup_state: AiPopupState,

    // Message to display
    pub status_message: Option<String>,
}

impl App {
    pub fn new() -> Result<Self> {
        let db = Database::new()?;

        // Load settings
        let settings_store = SettingsStore::new(&db.conn);
        let mut settings_state = SettingsState::default();

        if let Ok(Some(key)) = settings_store.get("anthropic_key") {
            settings_state.anthropic_key = key;
        }
        if let Ok(Some(model)) = settings_store.get("anthropic_model") {
            settings_state.anthropic_model = model;
        }
        if let Ok(Some(key)) = settings_store.get("openai_key") {
            settings_state.openai_key = key;
        }
        if let Ok(Some(path)) = settings_store.get("export_path") {
            settings_state.export_path = path;
        }

        let mut app = Self {
            should_quit: false,
            screen: Screen::Main,
            focus: Focus::ItemList,
            db,
            items: Vec::new(),
            category_counts: Vec::new(),
            tags: Vec::new(),
            selected_category: None,
            selected_tag: None,
            selected_item_index: 0,
            sidebar_index: 0,
            pending_key: None,
            view_state: ViewState::default(),
            edit_state: EditState::new_item(),
            search_state: SearchState::default(),
            settings_state,
            help_state: HelpState::default(),
            confirm_dialog: None,
            show_ai_popup: false,
            ai_popup_state: AiPopupState::default(),
            status_message: None,
        };

        app.refresh_data()?;
        Ok(app)
    }

    pub fn refresh_data(&mut self) -> Result<()> {
        let store = ItemStore::new(&self.db.conn);

        self.items = match (&self.selected_category, &self.selected_tag) {
            (Some(cat), _) => store.list_by_category(*cat)?,
            (None, Some(tag)) => store.list_by_tag(tag)?,
            (None, None) => store.list_recent(100)?,
        };

        self.category_counts = store.count_by_category()?;
        self.tags = store.get_tags_with_counts()?;

        if self.selected_item_index >= self.items.len() && !self.items.is_empty() {
            self.selected_item_index = self.items.len() - 1;
        }

        Ok(())
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| crate::ui::draw(frame, &mut self))?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key(key)?;
                }
            }
        }

        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        if key.kind != KeyEventKind::Press {
            return Ok(());
        }

        // Clear status message on any key press
        self.status_message = None;

        // Handle confirmation dialog first
        if self.confirm_dialog.is_some() {
            return self.handle_dialog_key(key);
        }

        // Handle AI popup
        if self.show_ai_popup {
            return self.handle_ai_popup_key(key);
        }

        // Check for pending vim sequences
        if let Some(pending) = self.pending_key.take() {
            return self.handle_vim_sequence(pending, key.code);
        }

        match self.screen {
            Screen::Main => self.handle_main_key(key)?,
            Screen::View => self.handle_view_key(key)?,
            Screen::Edit => self.handle_edit_key(key)?,
            Screen::Search => self.handle_search_key(key)?,
            Screen::Settings => self.handle_settings_key(key)?,
            Screen::Help => self.handle_help_key(key)?,
        }

        Ok(())
    }

    fn handle_main_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('j') | KeyCode::Down => self.move_down(),
            KeyCode::Char('k') | KeyCode::Up => self.move_up(),
            KeyCode::Char('h') | KeyCode::Left => {
                self.focus = Focus::Sidebar;
                self.handle_sidebar_selection()?;
            }
            KeyCode::Char('l') | KeyCode::Right => self.focus = Focus::ItemList,

            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => self.page_down(),
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => self.page_up(),

            KeyCode::Char('g') => self.pending_key = Some('g'),
            KeyCode::Char('d') => self.pending_key = Some('d'),
            KeyCode::Char('y') => self.pending_key = Some('y'),
            KeyCode::Char('G') => self.go_to_bottom(),

            KeyCode::Enter => {
                if self.focus == Focus::Sidebar {
                    self.handle_sidebar_selection()?;
                } else {
                    self.view_selected()?;
                }
            }
            KeyCode::Char('e') => self.edit_selected()?,
            KeyCode::Char('n') => self.new_item()?,
            KeyCode::Char('c') => self.copy_selected()?,
            KeyCode::Char('/') => self.open_search()?,
            KeyCode::Char('s') => self.open_settings()?,
            KeyCode::Char('x') => self.export_selected()?,
            KeyCode::Char('?') => self.screen = Screen::Help,

            KeyCode::Char('1') => self.select_category(Some(Category::Prompt))?,
            KeyCode::Char('2') => self.select_category(Some(Category::Agent))?,
            KeyCode::Char('3') => self.select_category(Some(Category::Skill))?,
            KeyCode::Char('4') => self.select_category(Some(Category::Command))?,
            KeyCode::Char('0') => self.select_category(None)?,

            KeyCode::Esc => {
                self.selected_category = None;
                self.selected_tag = None;
                self.refresh_data()?;
            }

            _ => {}
        }

        Ok(())
    }

    fn handle_sidebar_selection(&mut self) -> Result<()> {
        if self.sidebar_index == 0 {
            // Recent Items
            self.selected_category = None;
            self.selected_tag = None;
            self.refresh_data()?;
        } else if self.sidebar_index <= 4 {
            // Category selection (indices 1-4)
            let category = Category::all()[self.sidebar_index - 1];
            self.select_category(Some(category))?;
        } else {
            // Tag selection (indices 5+)
            let tag_index = self.sidebar_index - 5;
            if let Some((tag, _)) = self.tags.get(tag_index) {
                self.selected_tag = Some(tag.clone());
                self.selected_category = None;
                self.refresh_data()?;
            }
        }
        Ok(())
    }

    fn handle_vim_sequence(&mut self, first: char, second: KeyCode) -> Result<()> {
        match (first, second) {
            ('g', KeyCode::Char('g')) => self.go_to_top(),
            ('d', KeyCode::Char('d')) => self.delete_selected()?,
            ('y', KeyCode::Char('y')) => self.copy_selected()?,
            _ => {}
        }
        Ok(())
    }

    fn handle_view_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => self.screen = Screen::Main,
            KeyCode::Char('j') | KeyCode::Down => {
                if self.view_state.scroll < self.view_state.max_scroll {
                    self.view_state.scroll += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.view_state.scroll = self.view_state.scroll.saturating_sub(1);
            }
            KeyCode::Char('e') => self.edit_selected()?,
            KeyCode::Char('c') => self.copy_selected()?,
            KeyCode::Char('y') => self.pending_key = Some('y'),
            KeyCode::Char('d') => self.pending_key = Some('d'),
            KeyCode::Char('x') => self.export_selected()?,
            KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.show_ai_popup = true;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_edit_key(&mut self, key: KeyEvent) -> Result<()> {
        // Handle category dropdown if open
        if self.edit_state.show_category_dropdown {
            match key.code {
                KeyCode::Esc => {
                    self.edit_state.show_category_dropdown = false;
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    self.edit_state.select_category_from_dropdown();
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.edit_state.dropdown_next();
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.edit_state.dropdown_prev();
                }
                _ => {}
            }
            return Ok(());
        }

        match key.code {
            KeyCode::Esc => {
                if self.edit_state.has_changes {
                    self.confirm_dialog = Some(ConfirmDialog::discard_changes());
                } else {
                    self.screen = Screen::Main;
                }
            }
            KeyCode::Tab => self.edit_state.next_field(),
            KeyCode::BackTab => self.edit_state.prev_field(),
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.save_item()?;
            }
            KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if self.edit_state.focused_field == EditField::Content
                    || self.edit_state.focused_field == EditField::Description
                {
                    self.show_ai_popup = true;
                }
            }
            KeyCode::Char(' ') | KeyCode::Enter => {
                if self.edit_state.focused_field == EditField::Category {
                    // Open category dropdown
                    self.edit_state.open_category_dropdown();
                } else if self.edit_state.focused_field == EditField::Content
                    || self.edit_state.focused_field == EditField::Description
                {
                    self.edit_state.insert_char(if key.code == KeyCode::Enter { '\n' } else { ' ' });
                }
            }
            KeyCode::Char(c) => {
                if self.edit_state.focused_field != EditField::Category {
                    self.edit_state.insert_char(c);
                }
            }
            KeyCode::Backspace => self.edit_state.delete_char(),
            KeyCode::Delete => self.edit_state.delete_char_forward(),
            KeyCode::Left => self.edit_state.move_cursor_left(),
            KeyCode::Right => self.edit_state.move_cursor_right(),
            KeyCode::Home => self.edit_state.move_cursor_start(),
            KeyCode::End => self.edit_state.move_cursor_end(),
            _ => {}
        }
        Ok(())
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.screen = Screen::Main;
                self.search_state.clear();
            }
            KeyCode::Enter => {
                if let Some(item) = self.search_state.selected_item().cloned() {
                    // Find item in main list or add it
                    if let Some(idx) = self.items.iter().position(|i| i.id == item.id) {
                        self.selected_item_index = idx;
                    }
                    self.screen = Screen::Main;
                    self.search_state.clear();
                }
            }
            KeyCode::Char('j') | KeyCode::Down => self.search_state.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.search_state.select_prev(),
            KeyCode::Char('c') => {
                if let Some(item) = self.search_state.selected_item() {
                    self.copy_content(&item.content);
                }
            }
            KeyCode::Char(c) => {
                self.search_state.insert_char(c);
                self.perform_search()?;
            }
            KeyCode::Backspace => {
                self.search_state.delete_char();
                self.perform_search()?;
            }
            KeyCode::Left => self.search_state.move_cursor_left(),
            KeyCode::Right => self.search_state.move_cursor_right(),
            _ => {}
        }
        Ok(())
    }

    fn handle_settings_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                if self.settings_state.has_changes {
                    self.confirm_dialog = Some(ConfirmDialog::discard_changes());
                } else {
                    self.screen = Screen::Main;
                }
            }
            KeyCode::Tab => self.settings_state.next_field(),
            KeyCode::BackTab => self.settings_state.prev_field(),
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.save_settings()?;
            }
            KeyCode::Char(c) => self.settings_state.insert_char(c),
            KeyCode::Backspace => self.settings_state.delete_char(),
            _ => {}
        }
        Ok(())
    }

    fn handle_help_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') => self.screen = Screen::Main,
            KeyCode::Char('j') | KeyCode::Down => self.help_state.scroll_down(),
            KeyCode::Char('k') | KeyCode::Up => self.help_state.scroll_up(),
            _ => {}
        }
        Ok(())
    }

    fn handle_dialog_key(&mut self, key: KeyEvent) -> Result<()> {
        if let Some(ref mut dialog) = self.confirm_dialog {
            match key.code {
                KeyCode::Left | KeyCode::Right | KeyCode::Tab | KeyCode::Char('h') | KeyCode::Char('l') => {
                    dialog.toggle_selection();
                }
                KeyCode::Enter => {
                    let confirmed = dialog.selected;
                    let title = dialog.title.clone();
                    self.confirm_dialog = None;

                    if confirmed {
                        if title.contains("Delete") {
                            self.perform_delete()?;
                        } else if title.contains("Unsaved") {
                            // Discard changes
                            match self.screen {
                                Screen::Edit => self.screen = Screen::Main,
                                Screen::Settings => self.screen = Screen::Main,
                                _ => {}
                            }
                        }
                    }
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.confirm_dialog = None;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn handle_ai_popup_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.show_ai_popup = false;
                self.ai_popup_state.clear();
            }
            KeyCode::Char('j') | KeyCode::Down => self.ai_popup_state.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.ai_popup_state.select_prev(),
            KeyCode::Enter => {
                if self.ai_popup_state.result.is_some() {
                    // Apply the result
                    if let Some(result) = self.ai_popup_state.result.take() {
                        if self.edit_state.focused_field == EditField::Content {
                            self.edit_state.item.content = result;
                        } else if self.edit_state.focused_field == EditField::Description {
                            self.edit_state.item.description = Some(result);
                        }
                        self.edit_state.has_changes = true;
                    }
                    self.show_ai_popup = false;
                    self.ai_popup_state.clear();
                } else {
                    // Run AI completion
                    self.run_ai_completion()?;
                }
            }
            KeyCode::Char(c) if self.ai_popup_state.is_custom() => {
                self.ai_popup_state.insert_char(c);
            }
            KeyCode::Backspace if self.ai_popup_state.is_custom() => {
                self.ai_popup_state.delete_char();
            }
            _ => {}
        }
        Ok(())
    }

    fn run_ai_completion(&mut self) -> Result<()> {
        let content = &self.edit_state.item.content;
        let action = self.ai_popup_state.selected_action();

        let system_prompt = action.system_prompt().to_string();
        let user_message = if self.ai_popup_state.is_custom() && !self.ai_popup_state.custom_input.is_empty() {
            format!("Request: {}\n\nContent to process:\n{}", self.ai_popup_state.custom_input, content)
        } else {
            format!("Content to process:\n{}", content)
        };

        self.ai_popup_state.is_loading = true;
        self.ai_popup_state.error = None;

        let request = LlmRequest {
            system_prompt,
            user_message,
            max_tokens: 4096,
        };

        match complete_sync(
            Some(&self.settings_state.anthropic_key),
            Some(&self.settings_state.anthropic_model),
            Some(&self.settings_state.openai_key),
            request,
        ) {
            Ok(response) => {
                self.ai_popup_state.result = Some(response.content);
                self.ai_popup_state.is_loading = false;
            }
            Err(e) => {
                self.ai_popup_state.error = Some(e.to_string());
                self.ai_popup_state.is_loading = false;
            }
        }

        Ok(())
    }

    // Navigation helpers
    fn move_down(&mut self) {
        match self.focus {
            Focus::ItemList => {
                if !self.items.is_empty() {
                    self.selected_item_index =
                        (self.selected_item_index + 1).min(self.items.len() - 1);
                }
            }
            Focus::Sidebar => {
                let max_index = 5 + self.tags.len(); // Recent + 4 categories + tags
                self.sidebar_index = (self.sidebar_index + 1).min(max_index.saturating_sub(1));
            }
        }
    }

    fn move_up(&mut self) {
        match self.focus {
            Focus::ItemList => {
                self.selected_item_index = self.selected_item_index.saturating_sub(1);
            }
            Focus::Sidebar => {
                self.sidebar_index = self.sidebar_index.saturating_sub(1);
            }
        }
    }

    fn go_to_top(&mut self) {
        match self.focus {
            Focus::ItemList => self.selected_item_index = 0,
            Focus::Sidebar => self.sidebar_index = 0,
        }
    }

    fn go_to_bottom(&mut self) {
        match self.focus {
            Focus::ItemList => {
                if !self.items.is_empty() {
                    self.selected_item_index = self.items.len() - 1;
                }
            }
            Focus::Sidebar => {
                let max_index = 5 + self.tags.len(); // Recent + 4 categories + tags
                self.sidebar_index = max_index.saturating_sub(1);
            }
        }
    }

    fn page_down(&mut self) {
        if self.focus == Focus::ItemList && !self.items.is_empty() {
            self.selected_item_index = (self.selected_item_index + 10).min(self.items.len() - 1);
        }
    }

    fn page_up(&mut self) {
        if self.focus == Focus::ItemList {
            self.selected_item_index = self.selected_item_index.saturating_sub(10);
        }
    }

    // Action helpers
    fn select_category(&mut self, category: Option<Category>) -> Result<()> {
        self.selected_category = category;
        self.selected_tag = None;
        self.selected_item_index = 0;
        self.refresh_data()
    }

    fn view_selected(&mut self) -> Result<()> {
        if !self.items.is_empty() {
            self.view_state = ViewState::default();
            self.screen = Screen::View;
        }
        Ok(())
    }

    fn edit_selected(&mut self) -> Result<()> {
        if let Some(item) = self.items.get(self.selected_item_index).cloned() {
            self.edit_state = EditState::edit_item(item);
            self.screen = Screen::Edit;
        }
        Ok(())
    }

    fn new_item(&mut self) -> Result<()> {
        let mut new_state = EditState::new_item();
        // Set category based on current filter
        if let Some(cat) = self.selected_category {
            new_state.item.category = cat;
        }
        self.edit_state = new_state;
        self.screen = Screen::Edit;
        Ok(())
    }

    fn copy_selected(&mut self) -> Result<()> {
        if let Some(item) = self.items.get(self.selected_item_index) {
            self.copy_content(&item.content);
        }
        Ok(())
    }

    fn copy_content(&self, content: &str) {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            let _ = clipboard.set_text(content);
        }
    }

    fn delete_selected(&mut self) -> Result<()> {
        if let Some(item) = self.items.get(self.selected_item_index) {
            self.confirm_dialog = Some(ConfirmDialog::delete(&item.name));
        }
        Ok(())
    }

    fn perform_delete(&mut self) -> Result<()> {
        if let Some(item) = self.items.get(self.selected_item_index) {
            if let Some(id) = item.id {
                let store = ItemStore::new(&self.db.conn);
                store.delete(id)?;
                self.refresh_data()?;
            }
        }
        Ok(())
    }

    fn export_selected(&mut self) -> Result<()> {
        if let Some(item) = self.items.get(self.selected_item_index) {
            if item.category == Category::Prompt {
                self.status_message = Some("Prompts are copy-only (press 'c' to copy)".to_string());
                return Ok(());
            }

            let exporter = ClaudeExporter::new(&self.settings_state.export_path);
            match exporter.export(item) {
                Ok(path) => {
                    self.status_message = Some(format!("Exported to {}", path.display()));
                }
                Err(e) => {
                    self.status_message = Some(format!("Export failed: {}", e));
                }
            }
        }
        Ok(())
    }

    fn open_search(&mut self) -> Result<()> {
        self.search_state = SearchState::default();
        self.screen = Screen::Search;
        Ok(())
    }

    fn open_settings(&mut self) -> Result<()> {
        self.settings_state.has_changes = false;
        self.screen = Screen::Settings;
        Ok(())
    }

    fn perform_search(&mut self) -> Result<()> {
        if self.search_state.query.is_empty() {
            self.search_state.results.clear();
            return Ok(());
        }

        let store = ItemStore::new(&self.db.conn);
        self.search_state.results = store.search(&self.search_state.query)?;
        self.search_state.selected_index = 0;
        Ok(())
    }

    fn save_item(&mut self) -> Result<()> {
        // Validate
        if let Err(errors) = self.edit_state.item.validate() {
            self.status_message = Some(errors.join(", "));
            return Ok(());
        }

        let store = ItemStore::new(&self.db.conn);

        if self.edit_state.is_new {
            store.insert(&self.edit_state.item)?;
        } else {
            store.update(&self.edit_state.item)?;
        }

        self.edit_state.has_changes = false;
        self.screen = Screen::Main;
        self.refresh_data()?;
        Ok(())
    }

    fn save_settings(&mut self) -> Result<()> {
        let store = SettingsStore::new(&self.db.conn);

        store.set("anthropic_key", &self.settings_state.anthropic_key)?;
        store.set("anthropic_model", &self.settings_state.anthropic_model)?;
        store.set("openai_key", &self.settings_state.openai_key)?;
        store.set("export_path", &self.settings_state.export_path)?;

        self.settings_state.has_changes = false;
        self.status_message = Some("Settings saved".to_string());
        Ok(())
    }

    pub fn selected_item(&self) -> Option<&Item> {
        self.items.get(self.selected_item_index)
    }

    pub fn get_category_count(&self, category: Category) -> usize {
        self.category_counts
            .iter()
            .find(|(c, _)| *c == category)
            .map(|(_, count)| *count)
            .unwrap_or(0)
    }
}
