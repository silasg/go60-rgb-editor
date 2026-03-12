// Each integration test file compiles its own copy of this module,
// so functions used by some test files appear unused in others.
#![allow(dead_code)]

use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::TestBackend, buffer::Buffer, Terminal};

use go60_rgb_editor::app::App;
use go60_rgb_editor::event::handle_key;
use go60_rgb_editor::ui::draw;

/// Create an App loaded with the sample fixture config file.
pub fn create_e2e_app() -> App {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_config.txt");
    let content =
        std::fs::read_to_string(&fixture_path).expect("fixture file should exist");
    let config =
        go60_rgb_editor_domain::parser::parse_config(&content).expect("fixture should parse");
    App::new(config, fixture_path)
}

/// Send a plain key event (no modifiers) through the full dispatch pipeline.
pub fn send_key(app: &mut App, code: KeyCode) {
    let event = KeyEvent::new(code, KeyModifiers::NONE);
    handle_key(app, event);
}

/// Send a key event with modifiers through the full dispatch pipeline.
pub fn send_key_modified(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    let event = KeyEvent::new(code, modifiers);
    handle_key(app, event);
}

/// Type a string as a sequence of Char key events.
pub fn type_str(app: &mut App, text: &str) {
    for c in text.chars() {
        send_key(app, KeyCode::Char(c));
    }
}

/// Render the app to a `TestBackend` and return the `Buffer`.
pub fn render(app: &App, width: u16, height: u16) -> Buffer {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("terminal should initialize");
    terminal
        .draw(|frame| draw(frame, app))
        .expect("draw should succeed");
    terminal.backend().buffer().clone()
}

/// Check whether the buffer contains the given text anywhere.
pub fn buffer_contains(buffer: &Buffer, text: &str) -> bool {
    let full_text = extract_full_text(buffer);
    full_text.contains(text)
}

/// Extract all text from the buffer as a single string (rows joined with newlines).
fn extract_full_text(buffer: &Buffer) -> String {
    let area = buffer.area;
    let mut lines = Vec::new();
    for y in area.y..area.y + area.height {
        let mut line = String::new();
        for x in area.x..area.x + area.width {
            let cell = &buffer[(x, y)];
            line.push_str(cell.symbol());
        }
        lines.push(line);
    }
    lines.join("\n")
}


