use std::time::Instant;

use crate::model::Config;

/// Application mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Normal navigation mode
    Normal,
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

    /// Convert data column to visual column for a given row
    /// Visual columns account for the shifted positions of rows 4 and 5
    fn to_visual_col(&self, row: usize, col: usize) -> usize {
        if self.cursor.is_left {
            match row {
                0..=3 => col,
                4 => col + 2,      // Row 4 shifted 2 keys toward center
                5 => col + 5,      // Row 5 (thumbs) shifted 5 keys toward center
                _ => col,
            }
        } else {
            // Right half is mirrored
            match row {
                0..=3 => col,
                4 => col + 1,      // Row 4 shifted toward center
                5 => col.saturating_sub(2),  // Row 5 (thumbs) toward center
                _ => col,
            }
        }
    }

    /// Convert visual column to data column for a given row
    /// Returns the closest valid data column
    fn from_visual_col(&self, row: usize, visual_col: usize) -> usize {
        let max_col = if row < 4 { 6 } else { 3 };
        
        let data_col = if self.cursor.is_left {
            match row {
                0..=3 => visual_col,
                4 => visual_col.saturating_sub(2),
                5 => visual_col.saturating_sub(5),
                _ => visual_col,
            }
        } else {
            match row {
                0..=3 => visual_col,
                4 => visual_col.saturating_sub(1),
                5 => visual_col + 2,
                _ => visual_col,
            }
        };
        
        data_col.min(max_col - 1)
    }

    /// Move cursor in a direction
    pub fn move_cursor(&mut self, direction: Direction) {
        match direction {
            Direction::Up => {
                if self.cursor.row > 0 {
                    let visual_col = self.to_visual_col(self.cursor.row, self.cursor.col);
                    self.cursor.row -= 1;
                    self.cursor.col = self.from_visual_col(self.cursor.row, visual_col);
                }
            }
            Direction::Down => {
                if self.cursor.row < 5 {
                    let visual_col = self.to_visual_col(self.cursor.row, self.cursor.col);
                    self.cursor.row += 1;
                    self.cursor.col = self.from_visual_col(self.cursor.row, visual_col);
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
        use crate::model::ColorKind;
        
        // Separate colors by type
        let mut regular: Vec<usize> = Vec::new();
        let mut locks: Vec<usize> = Vec::new();
        let mut aliases: Vec<usize> = Vec::new();
        
        for (i, color) in self.config.palette.colors.iter().enumerate() {
            match &color.kind {
                ColorKind::Regular => regular.push(i),
                ColorKind::LockIndicator { .. } => locks.push(i),
                ColorKind::Alias { .. } => aliases.push(i),
            }
        }
        
        let cols = 17; // Colors per row in regular section (RED to PNK)
        let current = self.selected_color;
        
        // Determine which section we're in
        let in_regular = regular.contains(&current);
        let in_locks = locks.contains(&current);
        let in_aliases = aliases.contains(&current);
        
        match direction {
            Direction::Up => {
                if in_aliases && !locks.is_empty() {
                    // Move from aliases to locks
                    let pos = aliases.iter().position(|&x| x == current).unwrap_or(0);
                    self.selected_color = locks[pos.min(locks.len() - 1)];
                } else if in_locks && !regular.is_empty() {
                    // Move from locks to last row of regular
                    let pos = locks.iter().position(|&x| x == current).unwrap_or(0);
                    let last_row_start = (regular.len() - 1) / cols * cols;
                    let target = last_row_start + pos;
                    self.selected_color = regular[target.min(regular.len() - 1)];
                } else if in_regular {
                    // Move up within regular colors
                    let pos = regular.iter().position(|&x| x == current).unwrap_or(0);
                    if pos >= cols {
                        self.selected_color = regular[pos - cols];
                    }
                }
            }
            Direction::Down => {
                if in_regular {
                    let pos = regular.iter().position(|&x| x == current).unwrap_or(0);
                    if pos + cols < regular.len() {
                        // Move down within regular colors
                        self.selected_color = regular[pos + cols];
                    } else if !locks.is_empty() {
                        // Move from regular to locks
                        let col = pos % cols;
                        self.selected_color = locks[col.min(locks.len() - 1)];
                    }
                } else if in_locks && !aliases.is_empty() {
                    // Move from locks to aliases
                    let pos = locks.iter().position(|&x| x == current).unwrap_or(0);
                    self.selected_color = aliases[pos.min(aliases.len() - 1)];
                }
            }
            Direction::Left => {
                if in_regular {
                    let pos = regular.iter().position(|&x| x == current).unwrap_or(0);
                    if pos > 0 {
                        self.selected_color = regular[pos - 1];
                    }
                } else if in_locks {
                    let pos = locks.iter().position(|&x| x == current).unwrap_or(0);
                    if pos > 0 {
                        self.selected_color = locks[pos - 1];
                    }
                } else if in_aliases {
                    let pos = aliases.iter().position(|&x| x == current).unwrap_or(0);
                    if pos > 0 {
                        self.selected_color = aliases[pos - 1];
                    }
                }
            }
            Direction::Right => {
                if in_regular {
                    let pos = regular.iter().position(|&x| x == current).unwrap_or(0);
                    if pos + 1 < regular.len() {
                        self.selected_color = regular[pos + 1];
                    }
                } else if in_locks {
                    let pos = locks.iter().position(|&x| x == current).unwrap_or(0);
                    if pos + 1 < locks.len() {
                        self.selected_color = locks[pos + 1];
                    }
                } else if in_aliases {
                    let pos = aliases.iter().position(|&x| x == current).unwrap_or(0);
                    if pos + 1 < aliases.len() {
                        self.selected_color = aliases[pos + 1];
                    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_app() -> App {
        use crate::model::{Config, ColorPalette, Layer};
        use std::path::PathBuf;

        let mut config = Config::new(PathBuf::from("test.txt"));
        config.palette = ColorPalette::new();
        config.layers.push(Layer::new("Test".to_string(), "LAYER_Test".to_string()));
        
        App::new(config)
    }

    #[test]
    fn test_visual_col_mapping_left_half() {
        let mut app = create_test_app();
        app.cursor.is_left = true;

        // Row 0-3: visual = data
        assert_eq!(app.to_visual_col(0, 0), 0);
        assert_eq!(app.to_visual_col(3, 5), 5);

        // Row 4: visual = data + 2
        assert_eq!(app.to_visual_col(4, 0), 2);
        assert_eq!(app.to_visual_col(4, 1), 3);
        assert_eq!(app.to_visual_col(4, 2), 4);

        // Row 5: visual = data + 5
        assert_eq!(app.to_visual_col(5, 0), 5);
        assert_eq!(app.to_visual_col(5, 1), 6);
        assert_eq!(app.to_visual_col(5, 2), 7);
    }

    #[test]
    fn test_from_visual_col_left_half() {
        let mut app = create_test_app();
        app.cursor.is_left = true;

        // Row 0-3: data = visual
        assert_eq!(app.from_visual_col(3, 0), 0);
        assert_eq!(app.from_visual_col(3, 5), 5);

        // Row 4: data = visual - 2, clamped to 0-2
        assert_eq!(app.from_visual_col(4, 2), 0);
        assert_eq!(app.from_visual_col(4, 3), 1);
        assert_eq!(app.from_visual_col(4, 4), 2);
        assert_eq!(app.from_visual_col(4, 0), 0);  // Clamped

        // Row 5: data = visual - 5, clamped to 0-2
        assert_eq!(app.from_visual_col(5, 5), 0);
        assert_eq!(app.from_visual_col(5, 6), 1);
        assert_eq!(app.from_visual_col(5, 7), 2);
    }

    #[test]
    fn test_navigation_down_from_row3_to_row4_left() {
        let mut app = create_test_app();
        app.cursor.is_left = true;

        // r3,2 -> r4,0 (visual col 2 maps to data col 0 in row 4)
        app.cursor.row = 3;
        app.cursor.col = 2;
        app.move_cursor(Direction::Down);
        assert_eq!(app.cursor.row, 4);
        assert_eq!(app.cursor.col, 0);

        // r3,3 -> r4,1
        app.cursor.row = 3;
        app.cursor.col = 3;
        app.move_cursor(Direction::Down);
        assert_eq!(app.cursor.row, 4);
        assert_eq!(app.cursor.col, 1);

        // r3,4 -> r4,2
        app.cursor.row = 3;
        app.cursor.col = 4;
        app.move_cursor(Direction::Down);
        assert_eq!(app.cursor.row, 4);
        assert_eq!(app.cursor.col, 2);

        // r3,5 -> r5,0 (visual col 5 maps to data col 0 in row 5)
        app.cursor.row = 3;
        app.cursor.col = 5;
        app.move_cursor(Direction::Down);
        assert_eq!(app.cursor.row, 4);
        // visual col 5 in row 4 would be data col 3, clamped to 2
        assert_eq!(app.cursor.col, 2);
    }

    #[test]
    fn test_navigation_up_from_row4_to_row3_left() {
        let mut app = create_test_app();
        app.cursor.is_left = true;

        // r4,0 -> r3,2 (visual col 2)
        app.cursor.row = 4;
        app.cursor.col = 0;
        app.move_cursor(Direction::Up);
        assert_eq!(app.cursor.row, 3);
        assert_eq!(app.cursor.col, 2);

        // r4,1 -> r3,3 (visual col 3)
        app.cursor.row = 4;
        app.cursor.col = 1;
        app.move_cursor(Direction::Up);
        assert_eq!(app.cursor.row, 3);
        assert_eq!(app.cursor.col, 3);

        // r4,2 -> r3,4 (visual col 4)
        app.cursor.row = 4;
        app.cursor.col = 2;
        app.move_cursor(Direction::Up);
        assert_eq!(app.cursor.row, 3);
        assert_eq!(app.cursor.col, 4);
    }

    #[test]
    fn test_navigation_up_from_row5_to_row4_left() {
        let mut app = create_test_app();
        app.cursor.is_left = true;

        // r5,0 -> r4,2 (visual col 5, closest in row 4 is col 2 at visual 4)
        app.cursor.row = 5;
        app.cursor.col = 0;
        app.move_cursor(Direction::Up);
        assert_eq!(app.cursor.row, 4);
        assert_eq!(app.cursor.col, 2);  // visual 5 - 2 = 3, but row 4 max is 2
    }
}
