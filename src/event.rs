use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use crate::app::{App, Direction, Mode};

pub fn handle_events(app: &mut App, timeout: Duration) -> std::io::Result<bool> {
    if event::poll(timeout)? {
        if let Event::Key(key) = event::read()? {
            handle_key(app, key);
        }
    }
    Ok(!app.should_quit)
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match app.mode {
        Mode::Normal => handle_normal_mode(app, key),
        Mode::ColorPick => handle_color_pick_mode(app, key),
        Mode::Help => handle_help_mode(app, key),
        Mode::ConfirmQuit => handle_confirm_quit_mode(app, key),
        Mode::ConfirmCopy => handle_confirm_copy_mode(app, key),
        Mode::SaveAs => handle_save_as_mode(app, key),
        Mode::SaveAsConfirm => handle_save_as_confirm_mode(app, key),
        Mode::AddLayer => handle_add_layer_mode(app, key),
        Mode::RenameLayer => handle_rename_layer_mode(app, key),
        Mode::ConfirmDelete => handle_confirm_delete_mode(app, key),
    }
}

fn handle_normal_mode(app: &mut App, key: KeyEvent) {
    if handle_normal_navigation(app, key) {
        return;
    }
    handle_normal_action(app, key);
}

fn handle_normal_navigation(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Char('h') | KeyCode::Left => app.move_cursor(Direction::Left),
        KeyCode::Char('j') | KeyCode::Down => app.move_cursor(Direction::Down),
        KeyCode::Char('k') | KeyCode::Up => app.move_cursor(Direction::Up),
        KeyCode::Char('l') | KeyCode::Right => app.move_cursor(Direction::Right),
        KeyCode::Tab => app.switch_half(),
        KeyCode::Char('J') | KeyCode::PageDown => app.next_layer(),
        KeyCode::Char('K') | KeyCode::PageUp => app.prev_layer(),
        _ => return false,
    }
    true
}

fn handle_normal_action(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') => app.request_quit(),
        KeyCode::Char('Q') => app.should_quit = true,

        // Layer management
        KeyCode::Char('a') => app.start_add_layer(),
        KeyCode::Char('d') => app.duplicate_layer(),
        KeyCode::Char('n') => app.start_rename_layer(),
        KeyCode::Char('x') => app.start_delete_layer(),

        // Color picking
        KeyCode::Enter => app.enter_color_pick(),
        KeyCode::Char(c) if c.is_ascii_digit() => {
            let idx = c.to_digit(10).unwrap() as usize;
            app.apply_quick_color(idx);
        }

        // Editing
        KeyCode::Char('y') => app.copy_color(),
        KeyCode::Char('p') => app.paste_color(),
        KeyCode::Delete | KeyCode::Backspace => app.clear_color(),
        KeyCode::Char('u') => app.undo(),
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => app.redo(),

        // File operations
        KeyCode::Char('s') => app.save(),
        KeyCode::Char('S') => app.save_as(),
        KeyCode::Char('f') => app.increase_fade(),
        KeyCode::Char('F') => app.decrease_fade(),
        KeyCode::Char('c') => app.request_copy(),

        KeyCode::Char('?') => app.mode = Mode::Help,
        _ => {}
    }
}

fn handle_color_pick_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.mode = Mode::Normal,
        KeyCode::Enter => {
            app.apply_selected_color();
        }
        KeyCode::Char('h') | KeyCode::Left => app.move_color_selection(Direction::Left),
        KeyCode::Char('j') | KeyCode::Down => app.move_color_selection(Direction::Down),
        KeyCode::Char('k') | KeyCode::Up => app.move_color_selection(Direction::Up),
        KeyCode::Char('l') | KeyCode::Right => app.move_color_selection(Direction::Right),
        _ => {}
    }
}

fn handle_help_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') | KeyCode::Enter => {
            app.mode = Mode::Normal;
        }
        _ => {}
    }
}

fn handle_confirm_quit_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            app.should_quit = true;
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.mode = Mode::Normal;
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            app.save();
            app.should_quit = true;
        }
        _ => {}
    }
}

fn handle_confirm_copy_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            // Copy without saving
            app.copy_to_clipboard();
            app.mode = Mode::Normal;
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.mode = Mode::Normal;
        }
        KeyCode::Char('s') | KeyCode::Char('S') => {
            // Save first, then copy
            app.save();
            app.copy_to_clipboard();
            app.mode = Mode::Normal;
        }
        _ => {}
    }
}

fn handle_save_as_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.cancel_save_as();
        }
        KeyCode::Enter => {
            app.try_save_as();
        }
        KeyCode::Backspace => {
            app.filename_input.pop();
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Clear entire input
            app.filename_input.clear();
        }
        KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Delete last word
            // First remove trailing spaces
            while app.filename_input.ends_with(' ') {
                app.filename_input.pop();
            }
            // Then remove non-space characters
            while app.filename_input.chars().last().is_some_and(|c| c != ' ' && c != '/') {
                app.filename_input.pop();
            }
        }
        KeyCode::Char(c) => {
            app.filename_input.push(c);
        }
        _ => {}
    }
}

fn handle_save_as_confirm_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            // Overwrite the file
            app.execute_save_as();
        }
        KeyCode::Char('n') | KeyCode::Char('N') => {
            // Go back to filename input
            app.mode = Mode::SaveAs;
        }
        KeyCode::Esc => {
            // Cancel entirely
            app.cancel_save_as();
        }
        _ => {}
    }
}

fn handle_add_layer_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.cancel_layer_input();
        }
        KeyCode::Enter => {
            app.confirm_add_layer();
        }
        KeyCode::Backspace => {
            app.layer_name_input.pop();
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.layer_name_input.clear();
        }
        KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            while app.layer_name_input.ends_with(' ') {
                app.layer_name_input.pop();
            }
            while app.layer_name_input.chars().last().is_some_and(|c| c != ' ') {
                app.layer_name_input.pop();
            }
        }
        KeyCode::Char(c) => {
            app.layer_name_input.push(c);
        }
        _ => {}
    }
}

fn handle_rename_layer_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.cancel_layer_input();
        }
        KeyCode::Enter => {
            app.confirm_rename_layer();
        }
        KeyCode::Backspace => {
            app.layer_name_input.pop();
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.layer_name_input.clear();
        }
        KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            while app.layer_name_input.ends_with(' ') {
                app.layer_name_input.pop();
            }
            while app.layer_name_input.chars().last().is_some_and(|c| c != ' ') {
                app.layer_name_input.pop();
            }
        }
        KeyCode::Char(c) => {
            app.layer_name_input.push(c);
        }
        _ => {}
    }
}

fn handle_confirm_delete_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            app.confirm_delete_layer();
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.mode = Mode::Normal;
        }
        _ => {}
    }
}
