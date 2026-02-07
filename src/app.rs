use std::time::Instant;

use crate::model::Config;

/// Application mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Normal navigation mode
    Normal,
    /// Layer selection mode
    LayerSelect,
    /// Color picker mode
    ColorPick,
    /// Help popup
    Help,
    /// Confirm quit dialog
    ConfirmQuit,
}

/// Cursor position on the keyboard
#[derive(Debug, Clone, Copy, Default)]
pub struct Cursor {
    pub row: usize,
    pub col: usize,
    pub is_left: bool,
}

/// Direction for cursor movement
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// Main application state
pub struct App {
    pub config: Config,
    pub mode: Mode,
    pub current_layer: usize,
    pub cursor: Cursor,
    pub selected_color: usize,
    pub undo_stack: Vec<Config>,
    pub redo_stack: Vec<Config>,
    pub status_message: Option<(String, Instant)>,
    pub modified: bool,
    pub should_quit: bool,
    pub copied_color: Option<String>,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            mode: Mode::Normal,
            current_layer: 0,
            cursor: Cursor::default(),
            selected_color: 0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            status_message: None,
            modified: false,
            should_quit: false,
            copied_color: None,
        }
    }

    /// Get the current layer
    pub fn current_layer(&self) -> Option<&crate::model::Layer> {
        self.config.layers.get(self.current_layer)
    }

    /// Get the current layer mutably
    pub fn current_layer_mut(&mut self) -> Option<&mut crate::model::Layer> {
        self.config.layers.get_mut(self.current_layer)
    }

    /// Move cursor in a direction
    pub fn move_cursor(&mut self, direction: Direction) {
        match direction {
            Direction::Up => {
                if self.cursor.row > 0 {
                    self.cursor.row -= 1;
                    // Adjust column for thumb rows
                    self.clamp_cursor_col();
                }
            }
            Direction::Down => {
                if self.cursor.row < 5 {
                    self.cursor.row += 1;
                    self.clamp_cursor_col();
                }
            }
            Direction::Left => {
                let max_col = self.max_col_for_row();
                if self.cursor.col > 0 {
                    self.cursor.col -= 1;
                } else if !self.cursor.is_left {
                    // Wrap to left half
                    self.cursor.is_left = true;
                    self.cursor.col = max_col - 1;
                }
            }
            Direction::Right => {
                let max_col = self.max_col_for_row();
                if self.cursor.col < max_col - 1 {
                    self.cursor.col += 1;
                } else if self.cursor.is_left {
                    // Wrap to right half
                    self.cursor.is_left = false;
                    self.cursor.col = 0;
                }
            }
        }
    }

    /// Clamp cursor column to valid range for current row
    fn clamp_cursor_col(&mut self) {
        let max_col = self.max_col_for_row();
        if self.cursor.col >= max_col {
            self.cursor.col = max_col - 1;
        }
    }

    /// Get max column for current row
    fn max_col_for_row(&self) -> usize {
        if self.cursor.row < 4 {
            6
        } else {
            3
        }
    }

    /// Switch between left and right half
    pub fn switch_half(&mut self) {
        self.cursor.is_left = !self.cursor.is_left;
        self.clamp_cursor_col();
    }

    /// Go to next layer
    pub fn next_layer(&mut self) {
        if !self.config.layers.is_empty() {
            self.current_layer = (self.current_layer + 1) % self.config.layers.len();
        }
    }

    /// Go to previous layer
    pub fn prev_layer(&mut self) {
        if !self.config.layers.is_empty() {
            if self.current_layer == 0 {
                self.current_layer = self.config.layers.len() - 1;
            } else {
                self.current_layer -= 1;
            }
        }
    }

    /// Set the color at current cursor position
    pub fn set_current_key_color(&mut self, color: &str) {
        self.push_undo();
        let row = self.cursor.row;
        let col = self.cursor.col;
        let is_left = self.cursor.is_left;
        if let Some(layer) = self.current_layer_mut() {
            layer.set_color(row, col, is_left, color.to_string());
            self.modified = true;
        }
    }

    /// Push current state to undo stack
    pub fn push_undo(&mut self) {
        self.undo_stack.push(self.config.clone());
        self.redo_stack.clear();
        // Limit stack size
        if self.undo_stack.len() > 50 {
            self.undo_stack.remove(0);
        }
    }

    /// Undo last change
    pub fn undo(&mut self) {
        if let Some(prev_config) = self.undo_stack.pop() {
            self.redo_stack.push(self.config.clone());
            self.config = prev_config;
            self.modified = true;
            self.show_status("Undo");
        } else {
            self.show_status("Nothing to undo");
        }
    }

    /// Redo last undone change
    pub fn redo(&mut self) {
        if let Some(next_config) = self.redo_stack.pop() {
            self.undo_stack.push(self.config.clone());
            self.config = next_config;
            self.modified = true;
            self.show_status("Redo");
        } else {
            self.show_status("Nothing to redo");
        }
    }

    /// Save the config
    pub fn save(&mut self) {
        match self.config.save() {
            Ok(()) => {
                self.modified = false;
                self.show_status("Saved!");
            }
            Err(e) => {
                self.show_status(&format!("Save failed: {}", e));
            }
        }
    }

    /// Show a status message
    pub fn show_status(&mut self, message: &str) {
        self.status_message = Some((message.to_string(), Instant::now()));
    }

    /// Clear expired status messages
    pub fn tick(&mut self) {
        if let Some((_, time)) = &self.status_message {
            if time.elapsed().as_secs() >= 3 {
                self.status_message = None;
            }
        }
    }

    /// Get the color at current cursor position
    pub fn get_current_color(&self) -> Option<&str> {
        let layer = self.current_layer()?;
        layer.get_color(self.cursor.row, self.cursor.col, self.cursor.is_left)
    }

    /// Copy color at cursor
    pub fn copy_color(&mut self) {
        let color = self.get_current_color().map(|s| s.to_string());
        if let Some(c) = color {
            self.show_status(&format!("Copied: {}", c));
            self.copied_color = Some(c);
        }
    }

    /// Paste copied color at cursor
    pub fn paste_color(&mut self) {
        if let Some(color) = self.copied_color.clone() {
            self.set_current_key_color(&color);
            self.show_status(&format!("Pasted: {}", color));
        } else {
            self.show_status("Nothing to paste");
        }
    }

    /// Move selection in color picker
    pub fn move_color_selection(&mut self, direction: Direction) {
        let cols = 14; // Colors per row in picker
        let len = self.config.palette.colors.len();

        match direction {
            Direction::Up => {
                if self.selected_color >= cols {
                    self.selected_color -= cols;
                }
            }
            Direction::Down => {
                if self.selected_color + cols < len {
                    self.selected_color += cols;
                }
            }
            Direction::Left => {
                if self.selected_color > 0 {
                    self.selected_color -= 1;
                }
            }
            Direction::Right => {
                if self.selected_color + 1 < len {
                    self.selected_color += 1;
                }
            }
        }
    }

    /// Apply selected color from picker
    pub fn apply_selected_color(&mut self) {
        if let Some(color) = self.config.palette.colors.get(self.selected_color) {
            let abbrev = color.abbrev.clone();
            self.set_current_key_color(&abbrev);
            self.mode = Mode::Normal;
        }
    }

    /// Quick select color by index (0-9)
    pub fn quick_color(&mut self, index: usize) {
        if let Some(color) = self.config.palette.colors.get(index) {
            let abbrev = color.abbrev.clone();
            self.set_current_key_color(&abbrev);
        }
    }
}
