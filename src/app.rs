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
    /// Confirm copy to clipboard dialog
    ConfirmCopy,
    /// Save As filename input
    SaveAs,
    /// Confirm overwrite existing file
    SaveAsConfirm,
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
    pub filename_input: String,
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
            filename_input: String::new(),
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

    /// Increase fade duration by 5ms
    pub fn increase_fade(&mut self) {
        if self.current_layer().is_some() {
            self.push_undo();
            if let Some(layer) = self.current_layer_mut() {
                layer.fade_delay += 5;
                let fade = layer.fade_delay;
                self.modified = true;
                self.show_status(&format!("Fade: {}ms", fade));
            }
        }
    }

    /// Decrease fade duration by 5ms (minimum 0)
    pub fn decrease_fade(&mut self) {
        if self.current_layer().is_some() {
            self.push_undo();
            if let Some(layer) = self.current_layer_mut() {
                layer.fade_delay = layer.fade_delay.saturating_sub(5);
                let fade = layer.fade_delay;
                self.modified = true;
                self.show_status(&format!("Fade: {}ms", fade));
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

    /// Open Save As dialog
    pub fn save_as(&mut self) {
        // Pre-populate with current filename
        self.filename_input = self.config.file_path
            .to_string_lossy()
            .to_string();
        self.mode = Mode::SaveAs;
    }

    /// Attempt to save to the entered filename (checks for existing file)
    pub fn try_save_as(&mut self) {
        if self.filename_input.is_empty() {
            self.show_status("Filename cannot be empty");
            return;
        }

        let path = std::path::PathBuf::from(&self.filename_input);
        
        // Check if file already exists (and is different from current file)
        if path.exists() && path != self.config.file_path {
            self.mode = Mode::SaveAsConfirm;
        } else {
            self.execute_save_as();
        }
    }

    /// Execute the Save As operation
    pub fn execute_save_as(&mut self) {
        let path = std::path::PathBuf::from(&self.filename_input);
        match self.config.save_as(&path) {
            Ok(()) => {
                self.config.file_path = path;
                self.modified = false;
                self.mode = Mode::Normal;
                self.filename_input.clear();
                self.show_status("Saved!");
            }
            Err(e) => {
                self.show_status(&format!("Save failed: {}", e));
                self.mode = Mode::SaveAs;
            }
        }
    }

    /// Cancel the Save As operation
    pub fn cancel_save_as(&mut self) {
        self.filename_input.clear();
        self.mode = Mode::Normal;
    }

    /// Copy file contents to system clipboard
    pub fn copy_to_clipboard(&mut self) {
        use std::process::{Command, Stdio};
        use std::io::Write;

        // Read the current file contents
        let content = match std::fs::read_to_string(&self.config.file_path) {
            Ok(c) => c,
            Err(e) => {
                self.show_status(&format!("Read failed: {}", e));
                return;
            }
        };

        // Try pbcopy (macOS), then xclip (Linux), then xsel (Linux)
        let result = Command::new("pbcopy")
            .stdin(Stdio::piped())
            .spawn()
            .or_else(|_| Command::new("xclip").args(["-selection", "clipboard"]).stdin(Stdio::piped()).spawn())
            .or_else(|_| Command::new("xsel").args(["--clipboard", "--input"]).stdin(Stdio::piped()).spawn());

        match result {
            Ok(mut child) => {
                // Write to stdin and drop it to close the pipe
                let write_result = {
                    if let Some(mut stdin) = child.stdin.take() {
                        stdin.write_all(content.as_bytes())
                    } else {
                        Err(std::io::Error::new(std::io::ErrorKind::Other, "no stdin"))
                    }
                };
                // stdin is now dropped/closed, so pbcopy will complete
                
                if write_result.is_ok() {
                    let _ = child.wait();
                    self.show_status("Copied to clipboard!");
                } else {
                    self.show_status("Clipboard write failed");
                }
            }
            Err(_) => {
                self.show_status("No clipboard command available");
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

    /// Clear color at cursor (set to black ___)
    pub fn clear_color(&mut self) {
        self.set_current_key_color("___");
        self.show_status("Cleared");
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
        // Arrange
        let mut app = create_test_app();
        app.cursor.is_left = true;

        // Act - main rows (0-3) have no offset
        let main_row_first_col_visual = app.to_visual_col(0, 0);
        let main_row_last_col_visual = app.to_visual_col(3, 5);

        // Assert
        assert_eq!(main_row_first_col_visual, 0);
        assert_eq!(main_row_last_col_visual, 5);

        // Act - row 4 (inner thumb) is offset by 2 toward center
        let inner_thumb_col0_visual = app.to_visual_col(4, 0);
        let inner_thumb_col1_visual = app.to_visual_col(4, 1);
        let inner_thumb_col2_visual = app.to_visual_col(4, 2);

        // Assert
        let inner_thumb_offset = 2;
        assert_eq!(inner_thumb_col0_visual, 0 + inner_thumb_offset);
        assert_eq!(inner_thumb_col1_visual, 1 + inner_thumb_offset);
        assert_eq!(inner_thumb_col2_visual, 2 + inner_thumb_offset);

        // Act - row 5 (outer thumb) is offset by 5 toward center
        let outer_thumb_col0_visual = app.to_visual_col(5, 0);
        let outer_thumb_col1_visual = app.to_visual_col(5, 1);
        let outer_thumb_col2_visual = app.to_visual_col(5, 2);

        // Assert
        let outer_thumb_offset = 5;
        assert_eq!(outer_thumb_col0_visual, 0 + outer_thumb_offset);
        assert_eq!(outer_thumb_col1_visual, 1 + outer_thumb_offset);
        assert_eq!(outer_thumb_col2_visual, 2 + outer_thumb_offset);
    }

    #[test]
    fn test_from_visual_col_left_half() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.is_left = true;
        let inner_thumb_offset = 2;
        let outer_thumb_offset = 5;

        // Act - main rows (0-3) have no offset
        let main_row_first_col_data = app.from_visual_col(3, 0);
        let main_row_last_col_data = app.from_visual_col(3, 5);

        // Assert
        assert_eq!(main_row_first_col_data, 0);
        assert_eq!(main_row_last_col_data, 5);

        // Act - row 4 (inner thumb) visual col minus offset, clamped to 0-2
        let inner_thumb_visual2_data = app.from_visual_col(4, inner_thumb_offset + 0);
        let inner_thumb_visual3_data = app.from_visual_col(4, inner_thumb_offset + 1);
        let inner_thumb_visual4_data = app.from_visual_col(4, inner_thumb_offset + 2);
        let inner_thumb_clamped_data = app.from_visual_col(4, 0);

        // Assert
        assert_eq!(inner_thumb_visual2_data, 0);
        assert_eq!(inner_thumb_visual3_data, 1);
        assert_eq!(inner_thumb_visual4_data, 2);
        assert_eq!(inner_thumb_clamped_data, 0);

        // Act - row 5 (outer thumb) visual col minus offset, clamped to 0-2
        let outer_thumb_visual5_data = app.from_visual_col(5, outer_thumb_offset + 0);
        let outer_thumb_visual6_data = app.from_visual_col(5, outer_thumb_offset + 1);
        let outer_thumb_visual7_data = app.from_visual_col(5, outer_thumb_offset + 2);

        // Assert
        assert_eq!(outer_thumb_visual5_data, 0);
        assert_eq!(outer_thumb_visual6_data, 1);
        assert_eq!(outer_thumb_visual7_data, 2);
    }

    #[test]
    fn test_navigation_down_from_row3_to_row4_left() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.is_left = true;
        let main_row = 3;
        let inner_thumb_row = 4;
        let inner_thumb_first_aligned_main_col = 2;
        let inner_thumb_max_col = 2;

        // Arrange
        app.cursor.row = main_row;
        app.cursor.col = inner_thumb_first_aligned_main_col;
        // Act
        app.move_cursor(Direction::Down);
        // Assert
        assert_eq!(app.cursor.row, inner_thumb_row);
        assert_eq!(app.cursor.col, 0);

        // Arrange
        app.cursor.row = main_row;
        app.cursor.col = inner_thumb_first_aligned_main_col + 1;
        // Act
        app.move_cursor(Direction::Down);
        // Assert
        assert_eq!(app.cursor.row, inner_thumb_row);
        assert_eq!(app.cursor.col, 1);

        // Arrange
        app.cursor.row = main_row;
        app.cursor.col = inner_thumb_first_aligned_main_col + 2;
        // Act
        app.move_cursor(Direction::Down);
        // Assert
        assert_eq!(app.cursor.row, inner_thumb_row);
        assert_eq!(app.cursor.col, 2);

        // Arrange
        app.cursor.row = main_row;
        app.cursor.col = 5;
        // Act
        app.move_cursor(Direction::Down);
        // Assert
        assert_eq!(app.cursor.row, inner_thumb_row);
        assert_eq!(app.cursor.col, inner_thumb_max_col);
    }

    #[test]
    fn test_navigation_up_from_row4_to_row3_left() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.is_left = true;
        let inner_thumb_row = 4;
        let main_row = 3;
        let inner_thumb_offset = 2;

        // Arrange
        app.cursor.row = inner_thumb_row;
        app.cursor.col = 0;
        // Act
        app.move_cursor(Direction::Up);
        // Assert
        assert_eq!(app.cursor.row, main_row);
        assert_eq!(app.cursor.col, 0 + inner_thumb_offset);

        // Arrange
        app.cursor.row = inner_thumb_row;
        app.cursor.col = 1;
        // Act
        app.move_cursor(Direction::Up);
        // Assert
        assert_eq!(app.cursor.row, main_row);
        assert_eq!(app.cursor.col, 1 + inner_thumb_offset);

        // Arrange
        app.cursor.row = inner_thumb_row;
        app.cursor.col = 2;
        // Act
        app.move_cursor(Direction::Up);
        // Assert
        assert_eq!(app.cursor.row, main_row);
        assert_eq!(app.cursor.col, 2 + inner_thumb_offset);
    }

    #[test]
    fn test_navigation_up_from_row5_to_row4_left() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.is_left = true;
        let outer_thumb_row = 5;
        let inner_thumb_row = 4;
        let inner_thumb_max_col = 2;

        // Arrange
        app.cursor.row = outer_thumb_row;
        app.cursor.col = 0;
        // Act
        app.move_cursor(Direction::Up);
        // Assert
        assert_eq!(app.cursor.row, inner_thumb_row);
        assert_eq!(app.cursor.col, inner_thumb_max_col);
    }

    #[test]
    fn test_copy_to_clipboard_with_valid_file() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Arrange
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test content").unwrap();
        let mut config = crate::model::Config::new(temp_file.path().to_path_buf());
        config.palette = crate::model::ColorPalette::new();
        let mut app = App::new(config);

        // Act
        app.copy_to_clipboard();

        // Assert
        assert!(app.status_message.is_some());
    }

    #[test]
    fn test_copy_to_clipboard_with_nonexistent_file() {
        use std::path::PathBuf;

        // Arrange
        let nonexistent_path = PathBuf::from("/nonexistent/path/file.txt");
        let mut config = crate::model::Config::new(nonexistent_path);
        config.palette = crate::model::ColorPalette::new();
        let mut app = App::new(config);

        // Act
        app.copy_to_clipboard();

        // Assert
        assert!(app.status_message.is_some());
        let (msg, _) = app.status_message.as_ref().unwrap();
        assert!(msg.contains("Read failed"));
    }

    #[test]
    fn test_save_as_opens_dialog_with_current_filename() {
        // Arrange
        let mut app = create_test_app();
        assert_eq!(app.mode, Mode::Normal);
        assert!(app.filename_input.is_empty());

        // Act
        app.save_as();

        // Assert
        assert_eq!(app.mode, Mode::SaveAs);
        assert_eq!(app.filename_input, "test.txt");
    }

    #[test]
    fn test_cancel_save_as() {
        // Arrange
        let mut app = create_test_app();
        app.save_as();
        app.filename_input = "modified.txt".to_string();

        // Act
        app.cancel_save_as();

        // Assert
        assert_eq!(app.mode, Mode::Normal);
        assert!(app.filename_input.is_empty());
    }

    #[test]
    fn test_try_save_as_empty_filename_shows_error() {
        // Arrange
        let mut app = create_test_app();
        app.mode = Mode::SaveAs;
        app.filename_input = String::new();

        // Act
        app.try_save_as();

        // Assert
        assert_eq!(app.mode, Mode::SaveAs);
        let (msg, _) = app.status_message.as_ref().unwrap();
        assert!(msg.contains("empty"));
    }

    #[test]
    fn test_try_save_as_existing_file_prompts_confirmation() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Arrange
        let source_file = NamedTempFile::new().unwrap();
        let mut config = crate::model::Config::new(source_file.path().to_path_buf());
        config.palette = crate::model::ColorPalette::new();
        let mut app = App::new(config);
        let mut existing_target_file = NamedTempFile::new().unwrap();
        writeln!(existing_target_file, "existing content").unwrap();
        app.mode = Mode::SaveAs;
        app.filename_input = existing_target_file.path().to_string_lossy().to_string();

        // Act
        app.try_save_as();

        // Assert
        assert_eq!(app.mode, Mode::SaveAsConfirm);
    }

    #[test]
    fn test_try_save_as_same_file_no_confirmation() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Arrange
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "content").unwrap();
        let mut config = crate::model::Config::new(temp_file.path().to_path_buf());
        config.palette = crate::model::ColorPalette::new();
        let mut app = App::new(config);
        app.mode = Mode::SaveAs;
        app.filename_input = temp_file.path().to_string_lossy().to_string();

        // Act
        app.try_save_as();

        // Assert
        assert_eq!(app.mode, Mode::Normal);
    }

    #[test]
    fn test_try_save_as_new_file_no_confirmation() {
        use tempfile::TempDir;

        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.txt");
        std::fs::write(&source_path, "content").unwrap();
        let mut config = crate::model::Config::new(source_path);
        config.palette = crate::model::ColorPalette::new();
        let mut app = App::new(config);
        app.mode = Mode::SaveAs;
        let new_file_path = temp_dir.path().join("new_file.txt");
        app.filename_input = new_file_path.to_string_lossy().to_string();

        // Act
        app.try_save_as();

        // Assert
        assert_eq!(app.mode, Mode::Normal);
        assert!(!app.modified);
    }

    #[test]
    fn test_execute_save_as_updates_file_path() {
        use tempfile::TempDir;

        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.txt");
        std::fs::write(&source_path, "content").unwrap();
        let mut config = crate::model::Config::new(source_path.clone());
        config.palette = crate::model::ColorPalette::new();
        let mut app = App::new(config);
        app.modified = true;
        let new_path = temp_dir.path().join("new_file.txt");
        app.filename_input = new_path.to_string_lossy().to_string();

        // Act
        app.execute_save_as();

        // Assert
        assert_eq!(app.config.file_path, new_path);
        assert!(!app.modified);
        assert_eq!(app.mode, Mode::Normal);
        assert!(app.filename_input.is_empty());
    }

    #[test]
    fn test_execute_save_as_invalid_path_shows_error() {
        // Arrange
        let mut app = create_test_app();
        let invalid_path = "/nonexistent/directory/file.txt";
        app.filename_input = invalid_path.to_string();

        // Act
        app.execute_save_as();

        // Assert
        assert_eq!(app.mode, Mode::SaveAs);
        let (msg, _) = app.status_message.as_ref().unwrap();
        assert!(msg.contains("Save failed"));
    }

    // --- Undo / Redo ---

    #[test]
    fn test_undo_restores_previous_color() {
        // Arrange
        let mut app = create_test_app();
        let row = 0;
        let col = 0;
        let is_left = true;
        app.cursor.row = row;
        app.cursor.col = col;
        app.cursor.is_left = is_left;
        let original_color = app.current_layer().unwrap().get_color(row, col, is_left).unwrap().to_string();

        // Act
        app.set_current_key_color("RED");
        app.undo();

        // Assert
        let restored_color = app.current_layer().unwrap().get_color(row, col, is_left).unwrap();
        assert_eq!(
            restored_color, original_color,
            "undo should restore color from '{}' back to '{}'", "RED", original_color
        );
    }

    #[test]
    fn test_redo_reapplies_undone_color() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.row = 0;
        app.cursor.col = 0;
        app.cursor.is_left = true;
        app.set_current_key_color("RED");
        app.undo();

        // Act
        app.redo();

        // Assert
        let reapplied_color = app.current_layer().unwrap().get_color(0, 0, true).unwrap();
        assert_eq!(reapplied_color, "RED", "redo should reapply the undone color change");
    }

    #[test]
    fn test_undo_with_empty_stack_shows_nothing_to_undo() {
        // Arrange
        let mut app = create_test_app();

        // Act
        app.undo();

        // Assert
        let (msg, _) = app.status_message.as_ref().expect("should show a status message");
        assert!(
            msg.contains("Nothing to undo"),
            "expected 'Nothing to undo' message, got: '{}'", msg
        );
    }

    #[test]
    fn test_redo_with_empty_stack_shows_nothing_to_redo() {
        // Arrange
        let mut app = create_test_app();

        // Act
        app.redo();

        // Assert
        let (msg, _) = app.status_message.as_ref().expect("should show a status message");
        assert!(
            msg.contains("Nothing to redo"),
            "expected 'Nothing to redo' message, got: '{}'", msg
        );
    }

    #[test]
    fn test_new_change_after_undo_clears_redo_stack() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.row = 0;
        app.cursor.col = 0;
        app.cursor.is_left = true;
        app.set_current_key_color("RED");
        app.undo();
        assert!(!app.redo_stack.is_empty(), "redo stack should not be empty after undo");

        // Act
        app.set_current_key_color("CYN");

        // Assert
        assert!(
            app.redo_stack.is_empty(),
            "redo stack should be cleared after a new change"
        );
    }

    #[test]
    fn test_undo_stack_limited_to_50_entries() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.row = 0;
        app.cursor.col = 0;
        app.cursor.is_left = true;
        let max_undo_size = 50;

        // Act
        for i in 0..=max_undo_size + 10 {
            app.set_current_key_color(&format!("C{:02}", i % 100));
        }

        // Assert
        assert!(
            app.undo_stack.len() <= max_undo_size,
            "undo stack should be limited to {} entries, got {}",
            max_undo_size, app.undo_stack.len()
        );
    }

    // --- Copy / Paste color ---

    #[test]
    fn test_copy_color_stores_current_key_color() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.row = 0;
        app.cursor.col = 0;
        app.cursor.is_left = true;
        app.set_current_key_color("RED");

        // Act
        app.copy_color();

        // Assert
        assert_eq!(
            app.copied_color.as_deref(), Some("RED"),
            "copied color should match the current key's color"
        );
    }

    #[test]
    fn test_paste_color_applies_copied_color_to_cursor_position() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.row = 0;
        app.cursor.col = 0;
        app.cursor.is_left = true;
        app.set_current_key_color("RED");
        app.copy_color();
        app.cursor.col = 1;

        // Act
        app.paste_color();

        // Assert
        let pasted_color = app.current_layer().unwrap().get_color(0, 1, true).unwrap();
        assert_eq!(pasted_color, "RED", "paste should apply the copied color to the new position");
    }

    #[test]
    fn test_paste_without_copy_shows_nothing_to_paste() {
        // Arrange
        let mut app = create_test_app();
        assert!(app.copied_color.is_none());

        // Act
        app.paste_color();

        // Assert
        let (msg, _) = app.status_message.as_ref().expect("should show a status message");
        assert!(
            msg.contains("Nothing to paste"),
            "expected 'Nothing to paste' message, got: '{}'", msg
        );
    }

    // --- Clear color ---

    #[test]
    fn test_clear_color_sets_key_to_off() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.row = 0;
        app.cursor.col = 0;
        app.cursor.is_left = true;
        app.set_current_key_color("RED");

        // Act
        app.clear_color();

        // Assert
        let off_color = "___";
        let cleared_color = app.current_layer().unwrap().get_color(0, 0, true).unwrap();
        assert_eq!(
            cleared_color, off_color,
            "clear should set the key color to off ('{}')", off_color
        );
    }

    // --- Fade duration ---

    #[test]
    fn test_increase_fade_adds_5ms() {
        // Arrange
        let mut app = create_test_app();
        let initial_fade = app.current_layer().unwrap().fade_delay;

        // Act
        app.increase_fade();

        // Assert
        let increased_fade = app.current_layer().unwrap().fade_delay;
        assert_eq!(
            increased_fade, initial_fade + 5,
            "increase_fade should add 5ms: expected {}, got {}", initial_fade + 5, increased_fade
        );
        assert!(app.modified, "increase_fade should mark the app as modified");
    }

    #[test]
    fn test_decrease_fade_subtracts_5ms() {
        // Arrange
        let mut app = create_test_app();
        let initial_fade = app.current_layer().unwrap().fade_delay;
        assert!(initial_fade >= 5, "test requires initial fade >= 5ms");

        // Act
        app.decrease_fade();

        // Assert
        let decreased_fade = app.current_layer().unwrap().fade_delay;
        assert_eq!(
            decreased_fade, initial_fade - 5,
            "decrease_fade should subtract 5ms: expected {}, got {}", initial_fade - 5, decreased_fade
        );
    }

    #[test]
    fn test_decrease_fade_does_not_go_below_zero() {
        // Arrange
        let mut app = create_test_app();
        app.current_layer_mut().unwrap().fade_delay = 3;

        // Act
        app.decrease_fade();

        // Assert
        let clamped_fade = app.current_layer().unwrap().fade_delay;
        assert_eq!(
            clamped_fade, 0,
            "decrease_fade should saturate at 0, got {}", clamped_fade
        );
    }

    // --- Layer navigation ---

    #[test]
    fn test_next_layer_wraps_around() {
        // Arrange
        let mut app = create_test_app();
        app.config.layers.push(crate::model::Layer::new("Second".to_string(), "LAYER_Second".to_string()));
        let layer_count = app.config.layers.len();
        app.current_layer = layer_count - 1;

        // Act
        app.next_layer();

        // Assert
        assert_eq!(
            app.current_layer, 0,
            "next_layer should wrap from last layer ({}) to first (0)", layer_count - 1
        );
    }

    #[test]
    fn test_prev_layer_wraps_around() {
        // Arrange
        let mut app = create_test_app();
        app.config.layers.push(crate::model::Layer::new("Second".to_string(), "LAYER_Second".to_string()));
        let layer_count = app.config.layers.len();
        app.current_layer = 0;

        // Act
        app.prev_layer();

        // Assert
        assert_eq!(
            app.current_layer, layer_count - 1,
            "prev_layer should wrap from first layer (0) to last ({})", layer_count - 1
        );
    }

    #[test]
    fn test_next_layer_with_empty_layers_does_nothing() {
        // Arrange
        let mut app = create_test_app();
        app.config.layers.clear();

        // Act
        app.next_layer();

        // Assert
        assert_eq!(app.current_layer, 0, "next_layer should not change index when layers are empty");
    }

    #[test]
    fn test_prev_layer_with_empty_layers_does_nothing() {
        // Arrange
        let mut app = create_test_app();
        app.config.layers.clear();

        // Act
        app.prev_layer();

        // Assert
        assert_eq!(app.current_layer, 0, "prev_layer should not change index when layers are empty");
    }

    // --- Switch half ---

    #[test]
    fn test_switch_half_toggles_left_to_right() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.is_left = true;

        // Act
        app.switch_half();

        // Assert
        assert!(!app.cursor.is_left, "switch_half should toggle from left to right");
    }

    #[test]
    fn test_switch_half_clamps_column_on_thumb_row() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.is_left = true;
        app.cursor.row = 0;
        app.cursor.col = 5;

        // Act
        app.switch_half();
        app.cursor.row = 4;
        app.switch_half();

        // Assert
        let thumb_row_max_col = 2;
        assert!(
            app.cursor.col <= thumb_row_max_col,
            "switch_half should clamp column to valid range for thumb row, got col={}",
            app.cursor.col
        );
    }

    // --- Cursor movement: left/right wrapping ---

    #[test]
    fn test_move_left_at_right_half_start_wraps_to_left_half() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.is_left = false;
        app.cursor.row = 0;
        app.cursor.col = 0;
        let main_row_max_col = 5;

        // Act
        app.move_cursor(Direction::Left);

        // Assert
        assert!(app.cursor.is_left, "moving left from col 0 on right half should wrap to left half");
        assert_eq!(
            app.cursor.col, main_row_max_col,
            "wrapping to left half should place cursor at last column"
        );
    }

    #[test]
    fn test_move_right_at_left_half_end_wraps_to_right_half() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.is_left = true;
        app.cursor.row = 0;
        app.cursor.col = 5;

        // Act
        app.move_cursor(Direction::Right);

        // Assert
        assert!(!app.cursor.is_left, "moving right from last col on left half should wrap to right half");
        assert_eq!(app.cursor.col, 0, "wrapping to right half should place cursor at col 0");
    }

    #[test]
    fn test_move_left_at_left_half_start_stays() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.is_left = true;
        app.cursor.row = 0;
        app.cursor.col = 0;

        // Act
        app.move_cursor(Direction::Left);

        // Assert
        assert!(app.cursor.is_left, "should stay on left half");
        assert_eq!(app.cursor.col, 0, "should stay at col 0 when already at leftmost position");
    }

    #[test]
    fn test_move_right_at_right_half_end_stays() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.is_left = false;
        app.cursor.row = 0;
        app.cursor.col = 5;

        // Act
        app.move_cursor(Direction::Right);

        // Assert
        assert!(!app.cursor.is_left, "should stay on right half");
        assert_eq!(app.cursor.col, 5, "should stay at last col when already at rightmost position");
    }

    // --- Cursor movement: vertical boundaries ---

    #[test]
    fn test_move_up_at_row0_stays() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.row = 0;
        app.cursor.col = 3;

        // Act
        app.move_cursor(Direction::Up);

        // Assert
        assert_eq!(app.cursor.row, 0, "should stay at row 0 when moving up from top");
    }

    #[test]
    fn test_move_down_at_row5_stays() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.row = 5;
        app.cursor.col = 0;

        // Act
        app.move_cursor(Direction::Down);

        // Assert
        assert_eq!(app.cursor.row, 5, "should stay at row 5 when moving down from bottom");
    }

    // --- Visual column mapping: right half ---

    #[test]
    fn test_visual_col_mapping_right_half() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.is_left = false;

        // Act & Assert: main rows have no offset
        assert_eq!(app.to_visual_col(0, 0), 0, "right half main row col 0 should have no offset");
        assert_eq!(app.to_visual_col(3, 5), 5, "right half main row col 5 should have no offset");

        // Act & Assert: row 4 offset by 1
        assert_eq!(app.to_visual_col(4, 0), 1, "right half row 4 col 0 should offset by 1");
        assert_eq!(app.to_visual_col(4, 2), 3, "right half row 4 col 2 should offset by 1");

        // Act & Assert: row 5 subtracts 2
        assert_eq!(app.to_visual_col(5, 2), 0, "right half row 5 col 2 should map to visual 0");
        assert_eq!(app.to_visual_col(5, 0), 0, "right half row 5 col 0 should saturate to visual 0");
    }

    #[test]
    fn test_from_visual_col_right_half() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.is_left = false;

        // Act & Assert: main rows have no offset
        assert_eq!(app.from_visual_col(0, 3), 3, "right half main row visual 3 should map to data 3");

        // Act & Assert: row 4 subtracts 1
        assert_eq!(app.from_visual_col(4, 1), 0, "right half row 4 visual 1 should map to data 0");
        assert_eq!(app.from_visual_col(4, 3), 2, "right half row 4 visual 3 should map to data 2");

        // Act & Assert: row 5 adds 2, clamped
        assert_eq!(app.from_visual_col(5, 0), 2, "right half row 5 visual 0 should map to data 2");
    }

    // --- Quick color selection ---

    #[test]
    fn test_quick_color_applies_palette_color_by_index() {
        // Arrange
        let mut app = create_test_app();
        let red = crate::model::ColorDef::new("RED".to_string(), "RED_RGB".to_string(), crate::model::RgbColor::new(255, 0, 0));
        let grn = crate::model::ColorDef::new("GRN".to_string(), "GRN_RGB".to_string(), crate::model::RgbColor::new(0, 255, 0));
        app.config.palette.add(red);
        app.config.palette.add(grn);
        app.cursor.row = 0;
        app.cursor.col = 0;
        app.cursor.is_left = true;

        // Act
        let green_palette_index = 1;
        app.quick_color(green_palette_index);

        // Assert
        let applied_color = app.current_layer().unwrap().get_color(0, 0, true).unwrap();
        assert_eq!(
            applied_color, "GRN",
            "quick_color(1) should apply the second palette color ('GRN'), got '{}'", applied_color
        );
    }

    #[test]
    fn test_quick_color_with_out_of_range_index_does_nothing() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.row = 0;
        app.cursor.col = 0;
        app.cursor.is_left = true;
        let color_before = app.current_layer().unwrap().get_color(0, 0, true).unwrap().to_string();
        let out_of_range_index = 99;

        // Act
        app.quick_color(out_of_range_index);

        // Assert
        let color_after = app.current_layer().unwrap().get_color(0, 0, true).unwrap();
        assert_eq!(
            color_after, color_before,
            "quick_color with out-of-range index should not change the color"
        );
    }

    // --- Apply selected color from picker ---

    #[test]
    fn test_apply_selected_color_sets_key_and_returns_to_normal_mode() {
        // Arrange
        let mut app = create_test_app();
        let cyan = crate::model::ColorDef::new("CYN".to_string(), "CYN_RGB".to_string(), crate::model::RgbColor::new(0, 255, 255));
        app.config.palette.add(cyan);
        app.cursor.row = 0;
        app.cursor.col = 0;
        app.cursor.is_left = true;
        app.mode = Mode::ColorPick;
        app.selected_color = 0;

        // Act
        app.apply_selected_color();

        // Assert
        let applied_color = app.current_layer().unwrap().get_color(0, 0, true).unwrap();
        assert_eq!(applied_color, "CYN", "should apply the selected palette color");
        assert_eq!(app.mode, Mode::Normal, "should return to Normal mode after applying color");
    }

    // --- Tick / status message expiry ---

    #[test]
    fn test_tick_clears_expired_status_message() {
        // Arrange
        let mut app = create_test_app();
        let expired_time = Instant::now() - std::time::Duration::from_secs(5);
        app.status_message = Some(("old message".to_string(), expired_time));

        // Act
        app.tick();

        // Assert
        assert!(
            app.status_message.is_none(),
            "tick should clear status messages older than 3 seconds"
        );
    }

    #[test]
    fn test_tick_keeps_fresh_status_message() {
        // Arrange
        let mut app = create_test_app();
        app.show_status("fresh message");

        // Act
        app.tick();

        // Assert
        assert!(
            app.status_message.is_some(),
            "tick should keep status messages that are less than 3 seconds old"
        );
    }

    // --- set_current_key_color marks modified ---

    #[test]
    fn test_set_current_key_color_marks_modified() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.row = 0;
        app.cursor.col = 0;
        app.cursor.is_left = true;
        assert!(!app.modified);

        // Act
        app.set_current_key_color("RED");

        // Assert
        assert!(app.modified, "setting a key color should mark the app as modified");
    }
}
