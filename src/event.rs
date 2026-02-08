use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use crate::app::{App, Direction, Mode};

/// Handle events and return whether the app should continue
pub fn handle_events(app: &mut App, timeout: Duration) -> std::io::Result<bool> {
    if event::poll(timeout)? {
        if let Event::Key(key) = event::read()? {
            handle_key(app, key);
        }
    }
    Ok(!app.should_quit)
}

/// Handle a key event
fn handle_key(app: &mut App, key: KeyEvent) {
    match app.mode {
        Mode::Normal => handle_normal_mode(app, key),
        Mode::ColorPick => handle_color_pick_mode(app, key),
        Mode::Help => handle_help_mode(app, key),
        Mode::ConfirmQuit => handle_confirm_quit_mode(app, key),
        Mode::ConfirmCopy => handle_confirm_copy_mode(app, key),
        Mode::SaveAs => handle_save_as_mode(app, key),
        Mode::SaveAsConfirm => handle_save_as_confirm_mode(app, key),
    }
}

fn handle_normal_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        // Quit
        KeyCode::Char('q') => {
            if app.modified {
                app.mode = Mode::ConfirmQuit;
            } else {
                app.should_quit = true;
            }
        }
        KeyCode::Char('Q') => {
            app.should_quit = true;
        }

        // Navigation
        KeyCode::Char('h') | KeyCode::Left => app.move_cursor(Direction::Left),
        KeyCode::Char('j') | KeyCode::Down => app.move_cursor(Direction::Down),
        KeyCode::Char('k') | KeyCode::Up => app.move_cursor(Direction::Up),
        KeyCode::Char('l') | KeyCode::Right => app.move_cursor(Direction::Right),
        KeyCode::Tab => app.switch_half(),

        // Layer navigation (Shift+J/K or PageDown/PageUp)
        KeyCode::Char('J') | KeyCode::PageDown => app.next_layer(),
        KeyCode::Char('K') | KeyCode::PageUp => app.prev_layer(),

        // Color picking - initialize selection to current key's color
        KeyCode::Enter => {
            if let Some(color) = app.get_current_color() {
                if let Some(&idx) = app.config.palette.by_abbrev.get(color) {
                    app.selected_color = idx;
                }
            }
            app.mode = Mode::ColorPick;
        }

        // Quick color selection (0-9)
        KeyCode::Char(c) if c.is_ascii_digit() => {
            let idx = c.to_digit(10).unwrap() as usize;
            app.quick_color(idx);
        }

        // Copy/paste (vim-style)
        KeyCode::Char('y') => app.copy_color(),
        KeyCode::Char('p') => app.paste_color(),

        // Clear color (set to black)
        KeyCode::Delete | KeyCode::Backspace => app.clear_color(),

        // Undo/redo
        KeyCode::Char('u') => app.undo(),
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => app.redo(),

        // Save
        KeyCode::Char('s') => app.save(),
        KeyCode::Char('S') => app.save_as(),

        // Fade duration
        KeyCode::Char('f') => app.increase_fade(),
        KeyCode::Char('F') => app.decrease_fade(),

        // Copy file to clipboard
        KeyCode::Char('c') => {
            if app.modified {
                app.mode = Mode::ConfirmCopy;
            } else {
                app.copy_to_clipboard();
            }
        }

        // Help
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
            while app.filename_input.chars().last().map_or(false, |c| c == ' ') {
                app.filename_input.pop();
            }
            // Then remove non-space characters
            while app.filename_input.chars().last().map_or(false, |c| c != ' ' && c != '/') {
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


