//! Translation of the terminal events into actions.

use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};

use crate::tui::app::{App, Dialog, Effect, Level};
use crate::tui::render;

/// Handles an event, returning the side effect it asks for.
pub fn handle(app: &mut App, event: Event) -> Option<Effect> {
    match event {
        Event::Key(key) => handle_key(app, key),
        Event::Mouse(mouse) => handle_mouse(app, mouse),
        _ => None,
    }
}

/// Handles a key press.
fn handle_key(app: &mut App, key: KeyEvent) -> Option<Effect> {
    if key.kind != KeyEventKind::Press {
        return None;
    }

    // The messages of the status bar are replaced by the next action.
    app.status = None;

    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
        app.quit = true;
        return None;
    }

    if app.dialog.is_some() {
        return handle_dialog_key(app, key);
    }

    if app.filter.editing {
        return handle_filter_key(app, key);
    }

    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.quit = true;
            None
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.quit = true;
            None
        }
        KeyCode::Up => {
            app.move_selection(-1);
            None
        }
        KeyCode::Down => {
            app.move_selection(1);
            None
        }
        KeyCode::PageUp => {
            app.move_selection(-10);
            None
        }
        KeyCode::PageDown => {
            app.move_selection(10);
            None
        }
        KeyCode::Home => {
            app.select_edge(false);
            None
        }
        KeyCode::End => {
            app.select_edge(true);
            None
        }
        KeyCode::Enter => {
            if app.selected_row().map(|row| row.is_dir).unwrap_or(false) {
                app.enter_selected();
                None
            } else {
                app.request_open()
            }
        }
        KeyCode::Right => {
            app.enter_selected();
            None
        }
        KeyCode::Left | KeyCode::Backspace => {
            app.go_back();
            None
        }
        KeyCode::Char('-') => {
            app.go_back();
            None
        }
        KeyCode::Char('o') | KeyCode::Char('e') => app.request_open(),
        KeyCode::Char('x') => {
            app.ask_destination();
            None
        }
        KeyCode::Char('/') => {
            app.start_filter();
            None
        }
        KeyCode::Char('?') => {
            app.show_help();
            None
        }
        KeyCode::Char('r') => {
            match app.reload() {
                Ok(()) => app.set_status("the archive was reloaded", Level::Info),
                Err(error) => app.set_status(error.to_string(), Level::Error),
            }
            None
        }
        _ => None,
    }
}

/// Handles a key press while a dialog is open.
fn handle_dialog_key(app: &mut App, key: KeyEvent) -> Option<Effect> {
    let dialog = app.dialog.as_mut()?;

    match dialog {
        Dialog::Message { .. } | Dialog::Help => {
            app.close_dialog();
            None
        }
        Dialog::Confirm { .. } => match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => app.confirm(),
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                app.close_dialog();
                None
            }
            _ => None,
        },
        Dialog::Input { value, .. } => match key.code {
            KeyCode::Enter => app.submit_input(),
            KeyCode::Esc => {
                app.close_dialog();
                None
            }
            KeyCode::Backspace => {
                value.pop();
                None
            }
            KeyCode::Char(character) => {
                value.push(character);
                None
            }
            _ => None,
        },
    }
}

/// Handles a key press while the filter is being edited.
fn handle_filter_key(app: &mut App, key: KeyEvent) -> Option<Effect> {
    match key.code {
        KeyCode::Enter => {
            app.filter_apply();
            None
        }
        KeyCode::Esc => {
            app.filter_clear();
            None
        }
        KeyCode::Backspace => {
            app.filter_backspace();
            None
        }
        KeyCode::Char(character) => {
            app.filter_push(character);
            None
        }
        _ => None,
    }
}

/// Handles a mouse event.
fn handle_mouse(app: &mut App, mouse: MouseEvent) -> Option<Effect> {
    match mouse.kind {
        MouseEventKind::ScrollUp => {
            app.move_selection(-3);
            None
        }
        MouseEventKind::ScrollDown => {
            app.move_selection(3);
            None
        }
        MouseEventKind::Down(MouseButton::Left) => {
            let area = render::areas(mouse_area(app));
            let inner = area[1].inner(ratatui::layout::Margin::new(1, 1));
            if mouse.row >= inner.y && mouse.row < inner.y + inner.height {
                let index = (mouse.row - inner.y) as usize + app.state.offset();
                if index < app.rows().len() {
                    app.state.select(Some(index));
                }
            }
            None
        }
        _ => None,
    }
}

/// The area of the screen, used to translate the mouse position.
fn mouse_area(app: &App) -> ratatui::layout::Rect {
    let _ = app;
    let size = crossterm::terminal::size().unwrap_or((80, 24));
    ratatui::layout::Rect::new(0, 0, size.0, size.1)
}
