use serde_json::json;
use wasm_bindgen::prelude::*;

use go60_rgb_editor_domain::color::{ColorKind, ColorPalette};
use go60_rgb_editor_domain::config::Config;
use go60_rgb_editor_domain::cursor::Direction;
use go60_rgb_editor_domain::{parse_config, write_config, EditorState, Half, RgbPos};

/// Opaque editor handle exposed to JavaScript via WebAssembly.
///
/// Wraps `EditorState` in Wasm memory. JS calls methods on this handle
/// to mutate editor state, and reads the current state via getter methods.
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

    /// Set the cursor to a specific position.
    pub fn set_cursor(&mut self, half: &str, row: usize, col: usize) {
        let half = match half {
            "left" => Half::Left,
            "right" => Half::Right,
            _ => return,
        };
        self.inner.set_cursor(half, row, col);
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
            Half::Left => "left".to_string(),
            Half::Right => "right".to_string(),
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

    // --- Bulk data getters (JSON) ---

    /// Full UI state as JSON (call after any mutation to re-render).
    /// Returns: { cursor, currentLayerIndex, layerCount, modified, layers, palette }
    pub fn get_state_json(&self) -> String {
        let state = &self.inner;
        let half_str = match state.cursor.half {
            Half::Left => "left",
            Half::Right => "right",
        };

        let layers = serialize_layers(&state.config);
        let palette = serialize_palette(&state.config.palette);

        json!({
            "cursor": { "row": state.cursor.row, "col": state.cursor.col, "half": half_str },
            "currentLayerIndex": state.current_layer,
            "layerCount": state.config.layers.len(),
            "modified": state.modified,
            "layers": layers,
            "palette": palette
        })
        .to_string()
    }

    /// Current layer's key grid as JSON.
    /// Returns: { left: [[abbrev]], right: [[abbrev]], fadeDelay }
    pub fn get_layer_grid_json(&self, index: usize) -> String {
        if let Some(layer) = self.inner.config.layers.get(index) {
            json!({
                "left": layer.left_half,
                "right": layer.right_half,
                "fadeDelay": layer.fade_delay
            })
            .to_string()
        } else {
            json!({ "left": [], "right": [], "fadeDelay": 0 }).to_string()
        }
    }

    /// Set the current layer by index.
    pub fn set_layer(&mut self, index: usize) {
        if index < self.inner.config.layers.len() {
            self.inner.current_layer = index;
        }
    }

    // --- Positional color editing (for direct key clicks) ---

    /// Set color at a specific position (bypasses cursor).
    pub fn set_color_at(
        &mut self,
        half: &str,
        row: usize,
        col: usize,
        abbrev: &str,
    ) -> bool {
        let half = match half {
            "left" => Half::Left,
            "right" => Half::Right,
            _ => return false,
        };
        let pos = RgbPos { row, col, half };
        self.inner.set_key_color_at(&pos, abbrev)
    }

    /// Clear color at a specific position.
    pub fn clear_color_at(&mut self, half: &str, row: usize, col: usize) -> bool {
        self.set_color_at(half, row, col, "___")
    }

    /// Get the current layer's fade delay.
    pub fn fade_delay(&self) -> i32 {
        self.inner
            .current_layer()
            .map(|l| l.fade_delay as i32)
            .unwrap_or(-1)
    }
}

// --- Serialization helpers (not exposed to WASM) ---

fn serialize_layers(config: &Config) -> Vec<serde_json::Value> {
    config
        .layers
        .iter()
        .map(|l| json!({ "name": l.name, "fadeDelay": l.fade_delay }))
        .collect()
}

fn serialize_palette(palette: &ColorPalette) -> serde_json::Value {
    let categories = palette.categorize();

    let regular: Vec<serde_json::Value> = categories
        .regular
        .iter()
        .map(|&i| serialize_color(palette, i))
        .collect();

    let locks: Vec<serde_json::Value> = categories
        .locks
        .iter()
        .map(|&i| serialize_color(palette, i))
        .collect();

    let aliases: Vec<serde_json::Value> = categories
        .aliases
        .iter()
        .map(|&i| serialize_color(palette, i))
        .collect();

    json!({ "regular": regular, "locks": locks, "aliases": aliases })
}

fn serialize_color(palette: &ColorPalette, index: usize) -> serde_json::Value {
    let c = &palette.colors[index];
    let rgb = palette
        .get_effective_rgb(&c.abbrev)
        .cloned()
        .unwrap_or_default();
    let mut val = json!({ "abbrev": c.abbrev, "r": rgb.r, "g": rgb.g, "b": rgb.b });

    match &c.kind {
        ColorKind::LockIndicator {
            off_color,
            on_color,
        } => {
            val["offColor"] = json!(off_color);
            val["onColor"] = json!(on_color);
        }
        ColorKind::Alias { target } => {
            val["target"] = json!(target);
        }
        _ => {}
    }

    val
}
