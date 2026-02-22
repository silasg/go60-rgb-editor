use wasm_bindgen::prelude::*;

use go60_rgb_editor_domain::cursor::Direction;
use go60_rgb_editor_domain::{parse_config, write_config, EditorState};

/// Opaque editor handle exposed to JavaScript via WebAssembly.
///
/// Wraps `EditorState` in Wasm memory. JS calls methods on this handle
/// to mutate editor state, and reads the current state via `to_json()`.
/// Undo/redo history stays internal in Rust memory.
#[wasm_bindgen]
pub struct Editor {
    inner: EditorState,
}

#[wasm_bindgen]
impl Editor {
    /// Create an Editor by parsing a TailorKey config file string.
    #[wasm_bindgen(constructor)]
    pub fn new(config_text: &str) -> Result<Editor, String> {
        let config = parse_config(config_text)?;
        Ok(Editor {
            inner: EditorState::new(config),
        })
    }

    /// Serialize the current config back to the TailorKey file format.
    pub fn serialize(&self) -> String {
        write_config(&self.inner.config)
    }

    // --- Cursor navigation ---

    pub fn move_up(&mut self) {
        self.inner.move_cursor(Direction::Up);
    }

    pub fn move_down(&mut self) {
        self.inner.move_cursor(Direction::Down);
    }

    pub fn move_left(&mut self) {
        self.inner.move_cursor(Direction::Left);
    }

    pub fn move_right(&mut self) {
        self.inner.move_cursor(Direction::Right);
    }

    pub fn switch_half(&mut self) {
        self.inner.switch_half();
    }

    // --- Layer navigation ---

    pub fn next_layer(&mut self) {
        self.inner.next_layer();
    }

    pub fn prev_layer(&mut self) {
        self.inner.prev_layer();
    }

    // --- Color editing ---

    /// Set the color at the current cursor position. Returns true on success.
    pub fn set_color(&mut self, abbrev: &str) -> bool {
        self.inner.set_key_color(abbrev)
    }

    /// Clear the color at the current cursor position.
    pub fn clear_color(&mut self) -> bool {
        self.inner.clear_key_color()
    }

    /// Yank (copy) the color at the current cursor position.
    /// Returns the abbreviation, or empty string if no color.
    pub fn yank_color(&mut self) -> String {
        self.inner.yank_color().unwrap_or_default()
    }

    /// Paste the previously yanked color. Returns true on success.
    pub fn paste_color(&mut self) -> bool {
        self.inner.paste_color().is_some()
    }

    // --- Undo / redo ---

    pub fn undo(&mut self) -> bool {
        self.inner.undo()
    }

    pub fn redo(&mut self) -> bool {
        self.inner.redo()
    }

    // --- Fade ---

    /// Adjust fade delay by delta milliseconds. Returns the new value, or -1 if no layer.
    pub fn adjust_fade(&mut self, delta: i32) -> i32 {
        self.inner
            .adjust_fade(delta)
            .map(|v| v as i32)
            .unwrap_or(-1)
    }

    // --- Layer management ---

    pub fn add_layer(&mut self, name: &str) -> Result<(), String> {
        self.inner.add_layer(name)
    }

    pub fn duplicate_layer(&mut self) -> Result<String, String> {
        self.inner.duplicate_layer()
    }

    pub fn rename_layer(&mut self, new_name: &str) -> Result<(), String> {
        self.inner.rename_layer(new_name)
    }

    pub fn delete_layer(&mut self) -> Result<String, String> {
        self.inner.delete_layer()
    }

    // --- State query ---

    /// Whether the config has been modified since last save.
    pub fn is_modified(&self) -> bool {
        self.inner.modified
    }

    pub fn mark_saved(&mut self) {
        self.inner.mark_saved();
    }

    /// Get the current cursor row.
    pub fn cursor_row(&self) -> usize {
        self.inner.cursor.row
    }

    /// Get the current cursor column.
    pub fn cursor_col(&self) -> usize {
        self.inner.cursor.col
    }

    /// Get the current cursor half ("left" or "right").
    pub fn cursor_half(&self) -> String {
        match self.inner.cursor.half {
            go60_rgb_editor_domain::Half::Left => "left".to_string(),
            go60_rgb_editor_domain::Half::Right => "right".to_string(),
        }
    }

    /// Get the current layer index.
    pub fn current_layer_index(&self) -> usize {
        self.inner.current_layer
    }

    /// Get the color abbreviation at the current cursor position.
    pub fn current_color(&self) -> String {
        self.inner.current_color().unwrap_or("").to_string()
    }

    /// Get the number of layers.
    pub fn layer_count(&self) -> usize {
        self.inner.config.layers.len()
    }
}
