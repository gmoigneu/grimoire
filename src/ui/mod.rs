mod main_screen;
mod view_screen;
mod edit_screen;
mod search;
mod settings_screen;
mod help_screen;
mod dialog;
mod ai_popup;

pub use view_screen::ViewState;
pub use edit_screen::{EditState, EditField};
pub use search::SearchState;
pub use settings_screen::SettingsState;
pub use help_screen::HelpState;
pub use dialog::ConfirmDialog;
pub use ai_popup::AiPopupState;

use crate::app::{App, Screen};
use ratatui::Frame;

pub fn draw(frame: &mut Frame, app: &mut App) {
    // Draw the base screen
    match app.screen {
        Screen::Main => main_screen::draw(frame, app),
        Screen::View => {
            let item = app.selected_item().cloned();
            view_screen::draw(frame, item.as_ref(), &mut app.view_state);
        }
        Screen::Edit => edit_screen::draw(frame, &app.edit_state),
        Screen::Search => {
            main_screen::draw(frame, app);
            search::draw(frame, &app.search_state);
        }
        Screen::Settings => settings_screen::draw(frame, &app.settings_state),
        Screen::Help => {
            main_screen::draw(frame, app);
            help_screen::draw(frame, &mut app.help_state);
        }
    }

    // Draw overlays
    if let Some(ref dialog) = app.confirm_dialog {
        dialog::draw(frame, dialog);
    }

    if app.show_ai_popup {
        let content = app.edit_state.item.content.clone();
        let has_llm = !app.settings_state.anthropic_key.is_empty()
            || !app.settings_state.openai_key.is_empty();
        ai_popup::draw(frame, &app.ai_popup_state, &content, has_llm);
    }
}
