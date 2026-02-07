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
        Mode::LayerSelect => handle_layer_select_mode(app, key),
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

        // Layer navigation
        KeyCode::Char('n') => app.next_layer(),
        KeyCode::Char('p') => app.prev_layer(),

        // Color picking
        KeyCode::Enter => app.mode = Mode::ColorPick,

        // Quick color selection (0-9)
        KeyCode::Char(c) if c.is_ascii_digit() => {
            let idx = c.to_digit(10).unwrap() as usize;
            app.quick_color(idx);
        }

        // Copy/paste
        KeyCode::Char('y') => app.copy_color(),
        KeyCode::Char('Y') => app.paste_color(),

        // Undo/redo
        KeyCode::Char('u') => app.undo(),
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => app.redo(),

        // Save
        KeyCode::Char('s') => app.save(),

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

fn handle_layer_select_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.mode = Mode::Normal,
        KeyCode::Char('j') | KeyCode::Down => app.next_layer(),
        KeyCode::Char('k') | KeyCode::Up => app.prev_layer(),
        KeyCode::Enter => app.mode = Mode::Normal,
        _ => {}
    }
}
