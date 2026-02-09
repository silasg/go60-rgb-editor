use std::path::PathBuf;
use std::time::Instant;

use crate::domain::cursor;
pub use crate::domain::cursor::Direction;
use crate::domain::{Config, RgbPos};
use crate::ui::COLORS_PER_PICKER_ROW;
use crate::domain::undo::UndoHistory;

const FADE_STEP_MS: u16 = 5;
const STATUS_TIMEOUT_SECS: u64 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    ColorPick,
    Help,
    ConfirmQuit,
    ConfirmCopy,
    SaveAs,
    SaveAsConfirm,
}

fn move_within_section(section: &[usize], pos: usize, delta: isize) -> usize {
    let new_pos = pos as isize + delta;
    if new_pos >= 0 && (new_pos as usize) < section.len() {
        section[new_pos as usize]
    } else {
        section[pos]
    }
}

pub struct App {
    pub config: Config,
    pub file_path: PathBuf,
    pub mode: Mode,
    pub current_layer: usize,
    pub cursor: RgbPos,
    pub selected_color: usize,
    history: UndoHistory<Config>,
    pub status_message: Option<(String, Instant)>,
    pub modified: bool,
    pub should_quit: bool,
    pub yanked_color: Option<String>,
    pub filename_input: String,
}

impl App {
    pub fn new(config: Config, file_path: PathBuf) -> Self {
        Self {
            config,
            file_path,
            mode: Mode::Normal,
            current_layer: 0,
            cursor: RgbPos::default(),
            selected_color: 0,
            history: UndoHistory::new(),
            status_message: None,
            modified: false,
            should_quit: false,
            yanked_color: None,
            filename_input: String::new(),
        }
    }

    pub fn file_name(&self) -> &str {
        self.file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
    }

    pub fn current_layer(&self) -> Option<&crate::domain::Layer> {
        self.config.layers.get(self.current_layer)
    }

    pub fn current_layer_mut(&mut self) -> Option<&mut crate::domain::Layer> {
        self.config.layers.get_mut(self.current_layer)
    }

    pub fn move_cursor(&mut self, direction: Direction) {
        cursor::move_cursor(&mut self.cursor, direction);
    }

    pub fn switch_half(&mut self) {
        cursor::switch_half(&mut self.cursor);
    }

    pub fn next_layer(&mut self) {
        if !self.config.layers.is_empty() {
            self.current_layer = (self.current_layer + 1) % self.config.layers.len();
        }
    }

    pub fn prev_layer(&mut self) {
        if !self.config.layers.is_empty() {
            if self.current_layer == 0 {
                self.current_layer = self.config.layers.len() - 1;
            } else {
                self.current_layer -= 1;
            }
        }
    }

    pub fn increase_fade(&mut self) {
        self.adjust_fade(FADE_STEP_MS as i32);
    }

    pub fn decrease_fade(&mut self) {
        self.adjust_fade(-(FADE_STEP_MS as i32));
    }

    fn adjust_fade(&mut self, delta: i32) {
        if self.current_layer().is_some() {
            self.push_undo();
            if let Some(layer) = self.current_layer_mut() {
                let new_delay = (layer.fade_delay as i32 + delta).max(0) as u16;
                layer.fade_delay = new_delay;
                self.modified = true;
                self.show_status(&format!("Fade: {}ms", new_delay));
            }
        }
    }

    pub fn set_current_key_color(&mut self, color: &str) {
        self.push_undo();
        let pos = self.cursor;
        if let Some(layer) = self.current_layer_mut() {
            layer.set_color(&pos, color.to_string());
            self.modified = true;
        }
    }

    pub fn push_undo(&mut self) {
        self.history.save(self.config.clone());
    }

    pub fn undo(&mut self) {
        if let Some(prev_config) = self.history.undo(self.config.clone()) {
            self.config = prev_config;
            self.modified = true;
            self.show_status("Undo");
        } else {
            self.show_status("Nothing to undo");
        }
    }

    pub fn redo(&mut self) {
        if let Some(next_config) = self.history.redo(self.config.clone()) {
            self.config = next_config;
            self.modified = true;
            self.show_status("Redo");
        } else {
            self.show_status("Nothing to redo");
        }
    }

    pub fn save(&mut self) {
        match crate::io::save_config_with_backup(&self.config, &self.file_path) {
            Ok(()) => {
                self.modified = false;
                self.show_status("Saved!");
            }
            Err(e) => {
                self.show_status(&format!("Save failed: {}", e));
            }
        }
    }

    pub fn save_as(&mut self) {
        // Pre-populate with current filename
        self.filename_input = self.file_path
            .to_string_lossy()
            .to_string();
        self.mode = Mode::SaveAs;
    }

    pub fn try_save_as(&mut self) {
        if self.filename_input.is_empty() {
            self.show_status("Filename cannot be empty");
            return;
        }

        let path = PathBuf::from(&self.filename_input);

        // Check if file already exists (and is different from current file)
        if path.exists() && path != self.file_path {
            self.mode = Mode::SaveAsConfirm;
        } else {
            self.execute_save_as();
        }
    }

    pub fn execute_save_as(&mut self) {
        let path = PathBuf::from(&self.filename_input);
        match crate::io::save_config_with_backup(&self.config, &path) {
            Ok(()) => {
                self.file_path = path;
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

    pub fn cancel_save_as(&mut self) {
        self.filename_input.clear();
        self.mode = Mode::Normal;
    }

    pub fn copy_to_clipboard(&mut self) {
        let content = match std::fs::read_to_string(&self.file_path) {
            Ok(c) => c,
            Err(e) => {
                self.show_status(&format!("Read failed: {}", e));
                return;
            }
        };

        match crate::io::copy_to_clipboard(&content) {
            Ok(()) => self.show_status("Copied to clipboard!"),
            Err(e) => self.show_status(&e),
        }
    }

    pub fn show_status(&mut self, message: &str) {
        self.status_message = Some((message.to_string(), Instant::now()));
    }

    pub fn clear_expired_status(&mut self) {
        if let Some((_, time)) = &self.status_message {
            if time.elapsed().as_secs() >= STATUS_TIMEOUT_SECS {
                self.status_message = None;
            }
        }
    }

    pub fn get_current_color(&self) -> Option<&str> {
        let layer = self.current_layer()?;
        layer.get_color(&self.cursor)
    }

    pub fn copy_color(&mut self) {
        let color = self.get_current_color().map(|s| s.to_string());
        if let Some(c) = color {
            self.show_status(&format!("Copied: {}", c));
            self.yanked_color = Some(c);
        }
    }

    pub fn paste_color(&mut self) {
        if let Some(color) = self.yanked_color.clone() {
            self.set_current_key_color(&color);
            self.show_status(&format!("Pasted: {}", color));
        } else {
            self.show_status("Nothing to paste");
        }
    }

    pub fn clear_color(&mut self) {
        self.set_current_key_color("___");
        self.show_status("Cleared");
    }

    pub fn move_color_selection(&mut self, direction: Direction) {
        let categories = self.config.palette.categorize();
        let current = self.selected_color;
        let cols = COLORS_PER_PICKER_ROW;

        let sections = [&categories.regular, &categories.locks, &categories.aliases];

        let (section_idx, pos) = match sections.iter().enumerate()
            .find_map(|(i, s)| s.iter().position(|&x| x == current).map(|p| (i, p)))
        {
            Some(found) => found,
            None => return,
        };
        let section = sections[section_idx];

        match direction {
            Direction::Left => {
                self.selected_color = move_within_section(section, pos, -1);
            }
            Direction::Right => {
                self.selected_color = move_within_section(section, pos, 1);
            }
            Direction::Up => {
                self.selected_color = self.jump_to_prev_section(sections, section_idx, pos, cols);
            }
            Direction::Down => {
                self.selected_color = self.jump_to_next_section(sections, section_idx, pos, cols);
            }
        }
    }

    fn jump_to_prev_section(
        &self, sections: [&Vec<usize>; 3], section_idx: usize, pos: usize, cols: usize,
    ) -> usize {
        if section_idx == 0 {
            // Within regular: move up one row
            if pos >= cols { return sections[0][pos - cols]; }
            return self.selected_color;
        }
        let target = sections[section_idx - 1];
        if target.is_empty() { return self.selected_color; }

        let target_pos = if section_idx - 1 == 0 {
            // Jumping into regular: land on last row at same column
            let last_row_start = (target.len() - 1) / cols * cols;
            last_row_start + pos
        } else {
            pos
        };
        target[target_pos.min(target.len() - 1)]
    }

    fn jump_to_next_section(
        &self, sections: [&Vec<usize>; 3], section_idx: usize, pos: usize, cols: usize,
    ) -> usize {
        if section_idx == 0 && pos + cols < sections[0].len() {
            // Within regular: move down one row
            return sections[0][pos + cols];
        }
        if section_idx + 1 >= sections.len() { return self.selected_color; }

        let target = sections[section_idx + 1];
        if target.is_empty() { return self.selected_color; }

        let target_pos = if section_idx == 0 { pos % cols } else { pos };
        target[target_pos.min(target.len() - 1)]
    }

    pub fn apply_selected_color(&mut self) {
        if let Some(color) = self.config.palette.colors.get(self.selected_color) {
            let abbrev = color.abbrev.clone();
            self.set_current_key_color(&abbrev);
            self.mode = Mode::Normal;
        }
    }

    pub fn apply_quick_color(&mut self, index: usize) {
        if let Some(color) = self.config.palette.colors.get(index) {
            let abbrev = color.abbrev.clone();
            self.set_current_key_color(&abbrev);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Half;

    fn create_test_app() -> App {
        use crate::domain::{Config, ColorPalette, Layer};

        let mut config = Config::new();
        config.palette = ColorPalette::new();
        config.layers.push(Layer::new("Test".to_string(), "LAYER_Test".to_string()));

        App::new(config, PathBuf::from("test.txt"))
    }

    #[test]
    fn test_navigation_down_from_row3_to_row4_left() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.half = Half::Left;
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
        app.cursor.half = Half::Left;
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
        app.cursor.half = Half::Left;
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
        let mut config = crate::domain::Config::new();
        config.palette = crate::domain::ColorPalette::new();
        let mut app = App::new(config, temp_file.path().to_path_buf());

        // Act
        app.copy_to_clipboard();

        // Assert
        assert!(app.status_message.is_some());
    }

    #[test]
    fn test_copy_to_clipboard_with_nonexistent_file() {
        // Arrange
        let nonexistent_path = PathBuf::from("/nonexistent/path/file.txt");
        let mut config = crate::domain::Config::new();
        config.palette = crate::domain::ColorPalette::new();
        let mut app = App::new(config, nonexistent_path);

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
        let mut config = crate::domain::Config::new();
        config.palette = crate::domain::ColorPalette::new();
        let mut app = App::new(config, source_file.path().to_path_buf());
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
        let mut config = crate::domain::Config::new();
        config.palette = crate::domain::ColorPalette::new();
        let mut app = App::new(config, temp_file.path().to_path_buf());
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
        let mut config = crate::domain::Config::new();
        config.palette = crate::domain::ColorPalette::new();
        let mut app = App::new(config, source_path);
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
        let mut config = crate::domain::Config::new();
        config.palette = crate::domain::ColorPalette::new();
        let mut app = App::new(config, source_path);
        app.modified = true;
        let new_path = temp_dir.path().join("new_file.txt");
        app.filename_input = new_path.to_string_lossy().to_string();

        // Act
        app.execute_save_as();

        // Assert
        assert_eq!(app.file_path, new_path);
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
        app.cursor.row = row;
        app.cursor.col = col;
        app.cursor.half = Half::Left;
        let original_color = app.current_layer().unwrap().get_color(&RgbPos { row: row, col: col, half: Half::Left }).unwrap().to_string();

        // Act
        app.set_current_key_color("RED");
        app.undo();

        // Assert
        let restored_color = app.current_layer().unwrap().get_color(&RgbPos { row: row, col: col, half: Half::Left }).unwrap();
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
        app.cursor.half = Half::Left;
        app.set_current_key_color("RED");
        app.undo();

        // Act
        app.redo();

        // Assert
        let reapplied_color = app.current_layer().unwrap().get_color(&RgbPos { row: 0, col: 0, half: Half::Left }).unwrap();
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
    fn test_new_change_after_undo_clears_redo() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.row = 0;
        app.cursor.col = 0;
        app.cursor.half = Half::Left;
        app.set_current_key_color("RED");
        app.undo();

        // Act
        app.set_current_key_color("CYN");

        // Assert — redo should have nothing since the new change cleared it
        app.redo();
        let (msg, _) = app.status_message.as_ref().unwrap();
        assert!(msg.contains("Nothing to redo"), "redo should be cleared after a new change");
    }

    // --- Copy / Paste color ---

    #[test]
    fn test_copy_color_stores_current_key_color() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.row = 0;
        app.cursor.col = 0;
        app.cursor.half = Half::Left;
        app.set_current_key_color("RED");

        // Act
        app.copy_color();

        // Assert
        assert_eq!(
            app.yanked_color.as_deref(), Some("RED"),
            "copied color should match the current key's color"
        );
    }

    #[test]
    fn test_paste_color_applies_yanked_color_to_cursor_position() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.row = 0;
        app.cursor.col = 0;
        app.cursor.half = Half::Left;
        app.set_current_key_color("RED");
        app.copy_color();
        app.cursor.col = 1;

        // Act
        app.paste_color();

        // Assert
        let pasted_color = app.current_layer().unwrap().get_color(&RgbPos { row: 0, col: 1, half: Half::Left }).unwrap();
        assert_eq!(pasted_color, "RED", "paste should apply the copied color to the new position");
    }

    #[test]
    fn test_paste_without_copy_shows_nothing_to_paste() {
        // Arrange
        let mut app = create_test_app();
        assert!(app.yanked_color.is_none());

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
        app.cursor.half = Half::Left;
        app.set_current_key_color("RED");

        // Act
        app.clear_color();

        // Assert
        let off_color = "___";
        let cleared_color = app.current_layer().unwrap().get_color(&RgbPos { row: 0, col: 0, half: Half::Left }).unwrap();
        assert_eq!(
            cleared_color, off_color,
            "clear should set the key color to off ('{}')", off_color
        );
    }

    // --- Fade duration ---

    #[test]
    fn test_increase_fade_adds_one_step() {
        // Arrange
        let mut app = create_test_app();
        let initial_fade = app.current_layer().unwrap().fade_delay;

        // Act
        app.increase_fade();

        // Assert
        let increased_fade = app.current_layer().unwrap().fade_delay;
        assert_eq!(
            increased_fade, initial_fade + FADE_STEP_MS,
            "increase_fade should add {}ms: expected {}, got {}", FADE_STEP_MS, initial_fade + FADE_STEP_MS, increased_fade
        );
        assert!(app.modified, "increase_fade should mark the app as modified");
    }

    #[test]
    fn test_decrease_fade_subtracts_one_step() {
        // Arrange
        let mut app = create_test_app();
        let initial_fade = app.current_layer().unwrap().fade_delay;
        assert!(initial_fade >= FADE_STEP_MS, "test requires initial fade >= {}ms", FADE_STEP_MS);

        // Act
        app.decrease_fade();

        // Assert
        let decreased_fade = app.current_layer().unwrap().fade_delay;
        assert_eq!(
            decreased_fade, initial_fade - FADE_STEP_MS,
            "decrease_fade should subtract {}ms: expected {}, got {}", FADE_STEP_MS, initial_fade - FADE_STEP_MS, decreased_fade
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
        app.config.layers.push(crate::domain::Layer::new("Second".to_string(), "LAYER_Second".to_string()));
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
        app.config.layers.push(crate::domain::Layer::new("Second".to_string(), "LAYER_Second".to_string()));
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
        app.cursor.half = Half::Left;

        // Act
        app.switch_half();

        // Assert
        assert_eq!(app.cursor.half, Half::Right, "switch_half should toggle from left to right");
    }

    #[test]
    fn test_switch_half_clamps_column_on_thumb_row() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.half = Half::Left;
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
        app.cursor.half = Half::Right;
        app.cursor.row = 0;
        app.cursor.col = 0;
        let main_row_max_col = 5;

        // Act
        app.move_cursor(Direction::Left);

        // Assert
        assert_eq!(app.cursor.half, Half::Left, "moving left from col 0 on right half should wrap to left half");
        assert_eq!(
            app.cursor.col, main_row_max_col,
            "wrapping to left half should place cursor at last column"
        );
    }

    #[test]
    fn test_move_right_at_left_half_end_wraps_to_right_half() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.half = Half::Left;
        app.cursor.row = 0;
        app.cursor.col = 5;

        // Act
        app.move_cursor(Direction::Right);

        // Assert
        assert_eq!(app.cursor.half, Half::Right, "moving right from last col on left half should wrap to right half");
        assert_eq!(app.cursor.col, 0, "wrapping to right half should place cursor at col 0");
    }

    #[test]
    fn test_move_left_at_left_half_start_stays() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.half = Half::Left;
        app.cursor.row = 0;
        app.cursor.col = 0;

        // Act
        app.move_cursor(Direction::Left);

        // Assert
        assert_eq!(app.cursor.half, Half::Left, "should stay on left half");
        assert_eq!(app.cursor.col, 0, "should stay at col 0 when already at leftmost position");
    }

    #[test]
    fn test_move_right_at_right_half_end_stays() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.half = Half::Right;
        app.cursor.row = 0;
        app.cursor.col = 5;

        // Act
        app.move_cursor(Direction::Right);

        // Assert
        assert_eq!(app.cursor.half, Half::Right, "should stay on right half");
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

    // --- Quick color selection ---

    #[test]
    fn test_apply_quick_color_applies_palette_color_by_index() {
        // Arrange
        let mut app = create_test_app();
        let red = crate::domain::ColorDef::new("RED".to_string(), crate::domain::RgbColor::new(255, 0, 0));
        let grn = crate::domain::ColorDef::new("GRN".to_string(), crate::domain::RgbColor::new(0, 255, 0));
        app.config.palette.add(red);
        app.config.palette.add(grn);
        app.cursor.row = 0;
        app.cursor.col = 0;
        app.cursor.half = Half::Left;

        // Act
        let green_palette_index = 1;
        app.apply_quick_color(green_palette_index);

        // Assert
        let applied_color = app.current_layer().unwrap().get_color(&RgbPos { row: 0, col: 0, half: Half::Left }).unwrap();
        assert_eq!(
            applied_color, "GRN",
            "apply_quick_color(1) should apply the second palette color ('GRN'), got '{}'", applied_color
        );
    }

    #[test]
    fn test_apply_quick_color_with_out_of_range_index_does_nothing() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.row = 0;
        app.cursor.col = 0;
        app.cursor.half = Half::Left;
        let color_before = app.current_layer().unwrap().get_color(&RgbPos { row: 0, col: 0, half: Half::Left }).unwrap().to_string();
        let out_of_range_index = 99;

        // Act
        app.apply_quick_color(out_of_range_index);

        // Assert
        let color_after = app.current_layer().unwrap().get_color(&RgbPos { row: 0, col: 0, half: Half::Left }).unwrap();
        assert_eq!(
            color_after, color_before,
            "apply_quick_color with out-of-range index should not change the color"
        );
    }

    // --- Apply selected color from picker ---

    #[test]
    fn test_apply_selected_color_sets_key_and_returns_to_normal_mode() {
        // Arrange
        let mut app = create_test_app();
        let cyan = crate::domain::ColorDef::new("CYN".to_string(), crate::domain::RgbColor::new(0, 255, 255));
        app.config.palette.add(cyan);
        app.cursor.row = 0;
        app.cursor.col = 0;
        app.cursor.half = Half::Left;
        app.mode = Mode::ColorPick;
        app.selected_color = 0;

        // Act
        app.apply_selected_color();

        // Assert
        let applied_color = app.current_layer().unwrap().get_color(&RgbPos { row: 0, col: 0, half: Half::Left }).unwrap();
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
        app.clear_expired_status();

        // Assert
        assert!(
            app.status_message.is_none(),
            "clear_expired_status should clear messages older than {} seconds", STATUS_TIMEOUT_SECS
        );
    }

    #[test]
    fn test_tick_keeps_fresh_status_message() {
        // Arrange
        let mut app = create_test_app();
        app.show_status("fresh message");

        // Act
        app.clear_expired_status();

        // Assert
        assert!(
            app.status_message.is_some(),
            "clear_expired_status should keep messages that are less than {} seconds old", STATUS_TIMEOUT_SECS
        );
    }

    // --- set_current_key_color marks modified ---

    #[test]
    fn test_set_current_key_color_marks_modified() {
        // Arrange
        let mut app = create_test_app();
        app.cursor.row = 0;
        app.cursor.col = 0;
        app.cursor.half = Half::Left;
        assert!(!app.modified);

        // Act
        app.set_current_key_color("RED");

        // Assert
        assert!(app.modified, "setting a key color should mark the app as modified");
    }
}
