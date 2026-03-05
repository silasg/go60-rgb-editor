use super::config::Config;
use super::cursor::{self, Direction};
use super::undo::UndoHistory;
use super::{Half, Layer, RgbPos};

pub struct EditorState {
    pub config: Config,
    pub cursor: RgbPos,
    pub current_layer: usize,
    pub yanked_color: Option<String>,
    pub modified: bool,
    history: UndoHistory<Config>,
}

impl EditorState {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            cursor: RgbPos::default(),
            current_layer: 0,
            yanked_color: None,
            modified: false,
            history: UndoHistory::new(),
        }
    }

    pub fn current_layer(&self) -> Option<&Layer> {
        self.config.layers.get(self.current_layer)
    }

    pub fn current_layer_mut(&mut self) -> Option<&mut Layer> {
        self.config.layers.get_mut(self.current_layer)
    }

    pub fn move_cursor(&mut self, direction: Direction) {
        cursor::move_cursor(&mut self.cursor, direction);
    }

    pub fn switch_half(&mut self) {
        cursor::switch_half(&mut self.cursor);
    }

    pub fn set_cursor(&mut self, half: Half, row: usize, col: usize) {
        self.cursor = RgbPos { row, col, half };
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

    /// Set the color at the current cursor position. Returns true if a layer exists.
    pub fn set_key_color(&mut self, color: &str) -> bool {
        self.push_undo();
        let pos = self.cursor;
        if let Some(layer) = self.current_layer_mut() {
            layer.set_color(&pos, color.to_string());
            self.modified = true;
            true
        } else {
            false
        }
    }

    /// Set the color at a specific position. Returns true if a layer exists.
    pub fn set_key_color_at(&mut self, pos: &RgbPos, color: &str) -> bool {
        self.push_undo();
        if let Some(layer) = self.current_layer_mut() {
            layer.set_color(pos, color.to_string());
            self.modified = true;
            true
        } else {
            false
        }
    }

    /// Clear the color at the current cursor position.
    pub fn clear_key_color(&mut self) -> bool {
        self.set_key_color("___")
    }

    /// Copy (yank) the color at the current cursor position. Returns the copied abbreviation.
    pub fn yank_color(&mut self) -> Option<String> {
        let color = self.current_color()?.to_string();
        self.yanked_color = Some(color.clone());
        Some(color)
    }

    /// Paste the previously yanked color. Returns the pasted abbreviation.
    pub fn paste_color(&mut self) -> Option<String> {
        let color = self.yanked_color.clone()?;
        self.set_key_color(&color);
        Some(color)
    }

    /// Get the color abbreviation at the current cursor position.
    pub fn current_color(&self) -> Option<&str> {
        let layer = self.current_layer()?;
        layer.get_color(&self.cursor)
    }

    /// Adjust the fade delay by the given delta (positive or negative).
    /// Returns the new fade value, or None if no layer exists.
    pub fn adjust_fade(&mut self, delta: i32) -> Option<u16> {
        if self.current_layer().is_some() {
            self.push_undo();
            if let Some(layer) = self.current_layer_mut() {
                let new_delay = (layer.fade_delay as i32 + delta).max(0) as u16;
                layer.fade_delay = new_delay;
                self.modified = true;
                return Some(new_delay);
            }
        }
        None
    }

    pub fn push_undo(&mut self) {
        self.history.save(self.config.clone());
    }

    /// Undo the last change. Returns true if undo happened.
    pub fn undo(&mut self) -> bool {
        if let Some(prev_config) = self.history.undo(self.config.clone()) {
            self.config = prev_config;
            self.modified = true;
            true
        } else {
            false
        }
    }

    /// Redo a previously undone change. Returns true if redo happened.
    pub fn redo(&mut self) -> bool {
        if let Some(next_config) = self.history.redo(self.config.clone()) {
            self.config = next_config;
            self.modified = true;
            true
        } else {
            false
        }
    }

    // --- Layer management ---

    /// Add a new empty layer after the current layer. Returns `Ok(())` on success.
    pub fn add_layer(&mut self, name: &str) -> Result<(), String> {
        self.validate_layer_name(name, None)?;
        let macro_name = format!("LAYER_{}", name);
        let layer = Layer::new(name.to_string(), macro_name);
        self.push_undo();
        let insert_pos = self.current_layer + 1;
        self.config.layers.insert(insert_pos, layer);
        self.current_layer = insert_pos;
        self.modified = true;
        Ok(())
    }

    /// Duplicate the current layer with an auto-generated unique name.
    /// Returns the new layer's name on success.
    pub fn duplicate_layer(&mut self) -> Result<String, String> {
        if self.config.layers.is_empty() {
            return Err("No layers to duplicate".to_string());
        }
        let source = self.config.layers[self.current_layer].clone();
        let new_name = self.generate_unique_copy_name(&source.name);
        let macro_name = format!("LAYER_{}", new_name);
        let mut new_layer = source;
        new_layer.name = new_name.clone();
        new_layer.macro_name = macro_name;
        self.push_undo();
        let insert_pos = self.current_layer + 1;
        self.config.layers.insert(insert_pos, new_layer);
        self.current_layer = insert_pos;
        self.modified = true;
        Ok(new_name)
    }

    /// Rename the current layer. Returns `Ok(())` on success.
    pub fn rename_layer(&mut self, new_name: &str) -> Result<(), String> {
        self.validate_layer_name(new_name, Some(self.current_layer))?;
        self.push_undo();
        if let Some(layer) = self.config.layers.get_mut(self.current_layer) {
            layer.name = new_name.to_string();
            layer.macro_name = format!("LAYER_{}", new_name);
        }
        self.modified = true;
        Ok(())
    }

    /// Delete the current layer. Returns the deleted layer's name on success.
    pub fn delete_layer(&mut self) -> Result<String, String> {
        if self.config.layers.len() <= 1 {
            return Err("Cannot delete the last remaining layer".to_string());
        }
        self.push_undo();
        let removed = self.config.layers.remove(self.current_layer);
        if self.current_layer >= self.config.layers.len() {
            self.current_layer = self.config.layers.len() - 1;
        }
        self.modified = true;
        Ok(removed.name)
    }

    /// Validate a layer name: not empty, only `[A-Za-z0-9_]`, max 50 chars, unique.
    /// `exclude_index` allows excluding one layer (for rename of the same layer).
    fn validate_layer_name(&self, name: &str, exclude_index: Option<usize>) -> Result<(), String> {
        if name.is_empty() {
            return Err("Layer name cannot be empty".to_string());
        }
        if name.len() > 50 {
            return Err("Layer name cannot exceed 50 characters".to_string());
        }
        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err("Layer name can only contain letters, digits, and underscores".to_string());
        }
        for (i, layer) in self.config.layers.iter().enumerate() {
            if Some(i) == exclude_index {
                continue;
            }
            if layer.name == name {
                return Err(format!("Layer name '{}' already exists", name));
            }
        }
        Ok(())
    }

    /// Generate a unique copy name: `Name_copy`, `Name_copy_2`, `Name_copy_3`, ...
    fn generate_unique_copy_name(&self, base_name: &str) -> String {
        let existing_names: Vec<&str> = self.config.layers.iter().map(|l| l.name.as_str()).collect();
        let candidate = format!("{}_copy", base_name);
        if !existing_names.contains(&candidate.as_str()) {
            return candidate;
        }
        let mut counter = 2;
        loop {
            let candidate = format!("{}_copy_{}", base_name, counter);
            if !existing_names.contains(&candidate.as_str()) {
                return candidate;
            }
            counter += 1;
        }
    }

    /// Mark the editor state as saved (not modified).
    pub fn mark_saved(&mut self) {
        self.modified = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ColorPalette, Config, Half, Layer, RgbPos};

    fn create_test_editor() -> EditorState {
        let mut config = Config::new();
        config.palette = ColorPalette::new();
        config
            .layers
            .push(Layer::new("Test".to_string(), "LAYER_Test".to_string()));
        EditorState::new(config)
    }

    // --- Cursor navigation ---

    #[test]
    fn test_navigation_down_from_row3_to_row4_left() {
        // Arrange
        let mut editor = create_test_editor();
        editor.cursor.half = Half::Left;

        // Act & Assert
        editor.cursor.row = 3;
        editor.cursor.col = 2;
        editor.move_cursor(Direction::Down);
        assert_eq!(editor.cursor.row, 4);
        assert_eq!(editor.cursor.col, 0);

        editor.cursor.row = 3;
        editor.cursor.col = 3;
        editor.move_cursor(Direction::Down);
        assert_eq!(editor.cursor.row, 4);
        assert_eq!(editor.cursor.col, 1);

        editor.cursor.row = 3;
        editor.cursor.col = 4;
        editor.move_cursor(Direction::Down);
        assert_eq!(editor.cursor.row, 4);
        assert_eq!(editor.cursor.col, 2);

        editor.cursor.row = 3;
        editor.cursor.col = 5;
        editor.move_cursor(Direction::Down);
        assert_eq!(editor.cursor.row, 4);
        assert_eq!(editor.cursor.col, 2);
    }

    #[test]
    fn test_navigation_up_from_row4_to_row3_left() {
        // Arrange
        let mut editor = create_test_editor();
        editor.cursor.half = Half::Left;

        // Act & Assert
        editor.cursor.row = 4;
        editor.cursor.col = 0;
        editor.move_cursor(Direction::Up);
        assert_eq!(editor.cursor.row, 3);
        assert_eq!(editor.cursor.col, 2);

        editor.cursor.row = 4;
        editor.cursor.col = 1;
        editor.move_cursor(Direction::Up);
        assert_eq!(editor.cursor.row, 3);
        assert_eq!(editor.cursor.col, 3);

        editor.cursor.row = 4;
        editor.cursor.col = 2;
        editor.move_cursor(Direction::Up);
        assert_eq!(editor.cursor.row, 3);
        assert_eq!(editor.cursor.col, 4);
    }

    #[test]
    fn test_navigation_up_from_row5_to_row4_left() {
        // Arrange
        let mut editor = create_test_editor();
        editor.cursor.half = Half::Left;

        // Act
        editor.cursor.row = 5;
        editor.cursor.col = 0;
        editor.move_cursor(Direction::Up);

        // Assert
        assert_eq!(editor.cursor.row, 4);
        assert_eq!(editor.cursor.col, 2);
    }

    // --- Undo / Redo ---

    #[test]
    fn test_undo_restores_previous_color() {
        // Arrange
        let mut editor = create_test_editor();
        editor.cursor = RgbPos {
            row: 0,
            col: 0,
            half: Half::Left,
        };
        let original_color = editor
            .current_color()
            .unwrap()
            .to_string();

        // Act
        editor.set_key_color("RED");
        editor.undo();

        // Assert
        let restored = editor.current_color().unwrap();
        assert_eq!(restored, original_color);
    }

    #[test]
    fn test_redo_reapplies_undone_color() {
        // Arrange
        let mut editor = create_test_editor();
        editor.cursor = RgbPos {
            row: 0,
            col: 0,
            half: Half::Left,
        };
        editor.set_key_color("RED");
        editor.undo();

        // Act
        editor.redo();

        // Assert
        let reapplied = editor.current_color().unwrap();
        assert_eq!(reapplied, "RED");
    }

    #[test]
    fn test_undo_with_empty_stack_returns_false() {
        // Arrange
        let mut editor = create_test_editor();

        // Act & Assert
        assert!(!editor.undo());
    }

    #[test]
    fn test_redo_with_empty_stack_returns_false() {
        // Arrange
        let mut editor = create_test_editor();

        // Act & Assert
        assert!(!editor.redo());
    }

    #[test]
    fn test_new_change_after_undo_clears_redo() {
        // Arrange
        let mut editor = create_test_editor();
        editor.cursor = RgbPos {
            row: 0,
            col: 0,
            half: Half::Left,
        };
        editor.set_key_color("RED");
        editor.undo();

        // Act
        editor.set_key_color("CYN");

        // Assert
        assert!(!editor.redo(), "redo should be cleared after a new change");
    }

    // --- Copy / Paste color ---

    #[test]
    fn test_yank_color_returns_current_color() {
        // Arrange
        let mut editor = create_test_editor();
        editor.cursor = RgbPos {
            row: 0,
            col: 0,
            half: Half::Left,
        };
        editor.set_key_color("RED");

        // Act
        let yanked = editor.yank_color();

        // Assert
        assert_eq!(yanked.as_deref(), Some("RED"));
        assert_eq!(editor.yanked_color.as_deref(), Some("RED"));
    }

    #[test]
    fn test_paste_color_applies_yanked_color() {
        // Arrange
        let mut editor = create_test_editor();
        editor.cursor = RgbPos {
            row: 0,
            col: 0,
            half: Half::Left,
        };
        editor.set_key_color("RED");
        editor.yank_color();
        editor.cursor.col = 1;

        // Act
        let pasted = editor.paste_color();

        // Assert
        assert_eq!(pasted.as_deref(), Some("RED"));
        let color = editor
            .current_layer()
            .unwrap()
            .get_color(&RgbPos {
                row: 0,
                col: 1,
                half: Half::Left,
            })
            .unwrap();
        assert_eq!(color, "RED");
    }

    #[test]
    fn test_paste_without_yank_returns_none() {
        // Arrange
        let mut editor = create_test_editor();
        assert!(editor.yanked_color.is_none());

        // Act
        let result = editor.paste_color();

        // Assert
        assert!(result.is_none());
    }

    // --- Clear color ---

    #[test]
    fn test_clear_key_color_sets_to_off() {
        // Arrange
        let mut editor = create_test_editor();
        editor.cursor = RgbPos {
            row: 0,
            col: 0,
            half: Half::Left,
        };
        editor.set_key_color("RED");

        // Act
        editor.clear_key_color();

        // Assert
        let cleared = editor.current_color().unwrap();
        assert_eq!(cleared, "___");
    }

    // --- Fade duration ---

    #[test]
    fn test_adjust_fade_positive() {
        // Arrange
        let mut editor = create_test_editor();
        let initial = editor.current_layer().unwrap().fade_delay;

        // Act
        let result = editor.adjust_fade(5);

        // Assert
        assert_eq!(result, Some(initial + 5));
        assert!(editor.modified);
    }

    #[test]
    fn test_adjust_fade_negative() {
        // Arrange
        let mut editor = create_test_editor();
        let initial = editor.current_layer().unwrap().fade_delay;

        // Act
        let result = editor.adjust_fade(-5);

        // Assert
        assert_eq!(result, Some(initial - 5));
    }

    #[test]
    fn test_adjust_fade_clamps_at_zero() {
        // Arrange
        let mut editor = create_test_editor();
        editor.current_layer_mut().unwrap().fade_delay = 3;

        // Act
        let result = editor.adjust_fade(-5);

        // Assert
        assert_eq!(result, Some(0));
    }

    // --- Layer navigation ---

    #[test]
    fn test_next_layer_wraps_around() {
        // Arrange
        let mut editor = create_test_editor();
        editor
            .config
            .layers
            .push(Layer::new("Second".to_string(), "LAYER_Second".to_string()));
        editor.current_layer = editor.config.layers.len() - 1;

        // Act
        editor.next_layer();

        // Assert
        assert_eq!(editor.current_layer, 0);
    }

    #[test]
    fn test_prev_layer_wraps_around() {
        // Arrange
        let mut editor = create_test_editor();
        editor
            .config
            .layers
            .push(Layer::new("Second".to_string(), "LAYER_Second".to_string()));
        editor.current_layer = 0;

        // Act
        editor.prev_layer();

        // Assert
        assert_eq!(editor.current_layer, editor.config.layers.len() - 1);
    }

    #[test]
    fn test_next_layer_with_empty_layers_does_nothing() {
        // Arrange
        let mut editor = create_test_editor();
        editor.config.layers.clear();

        // Act
        editor.next_layer();

        // Assert
        assert_eq!(editor.current_layer, 0);
    }

    #[test]
    fn test_prev_layer_with_empty_layers_does_nothing() {
        // Arrange
        let mut editor = create_test_editor();
        editor.config.layers.clear();

        // Act
        editor.prev_layer();

        // Assert
        assert_eq!(editor.current_layer, 0);
    }

    // --- Switch half ---

    #[test]
    fn test_switch_half_toggles_left_to_right() {
        // Arrange
        let mut editor = create_test_editor();
        editor.cursor.half = Half::Left;

        // Act
        editor.switch_half();

        // Assert
        assert_eq!(editor.cursor.half, Half::Right);
    }

    #[test]
    fn test_switch_half_clamps_column_on_thumb_row() {
        // Arrange
        let mut editor = create_test_editor();
        editor.cursor.half = Half::Left;
        editor.cursor.row = 0;
        editor.cursor.col = 5;
        editor.switch_half();
        editor.cursor.row = 4;

        // Act
        editor.switch_half();

        // Assert
        assert!(editor.cursor.col <= 2);
    }

    // --- Cursor boundary tests ---

    #[test]
    fn test_move_left_at_right_half_start_wraps() {
        // Arrange
        let mut editor = create_test_editor();
        editor.cursor = RgbPos {
            row: 0,
            col: 0,
            half: Half::Right,
        };

        // Act
        editor.move_cursor(Direction::Left);

        // Assert
        assert_eq!(editor.cursor.half, Half::Left);
        assert_eq!(editor.cursor.col, 5);
    }

    #[test]
    fn test_move_right_at_left_half_end_wraps() {
        // Arrange
        let mut editor = create_test_editor();
        editor.cursor = RgbPos {
            row: 0,
            col: 5,
            half: Half::Left,
        };

        // Act
        editor.move_cursor(Direction::Right);

        // Assert
        assert_eq!(editor.cursor.half, Half::Right);
        assert_eq!(editor.cursor.col, 0);
    }

    #[test]
    fn test_move_left_at_left_half_start_stays() {
        // Arrange
        let mut editor = create_test_editor();
        editor.cursor = RgbPos {
            row: 0,
            col: 0,
            half: Half::Left,
        };

        // Act
        editor.move_cursor(Direction::Left);

        // Assert
        assert_eq!(editor.cursor.half, Half::Left);
        assert_eq!(editor.cursor.col, 0);
    }

    #[test]
    fn test_move_right_at_right_half_end_stays() {
        // Arrange
        let mut editor = create_test_editor();
        editor.cursor = RgbPos {
            row: 0,
            col: 5,
            half: Half::Right,
        };

        // Act
        editor.move_cursor(Direction::Right);

        // Assert
        assert_eq!(editor.cursor.half, Half::Right);
        assert_eq!(editor.cursor.col, 5);
    }

    #[test]
    fn test_move_up_at_row0_stays() {
        // Arrange
        let mut editor = create_test_editor();
        editor.cursor.row = 0;
        editor.cursor.col = 3;

        // Act
        editor.move_cursor(Direction::Up);

        // Assert
        assert_eq!(editor.cursor.row, 0);
    }

    #[test]
    fn test_move_down_at_row5_stays() {
        // Arrange
        let mut editor = create_test_editor();
        editor.cursor.row = 5;
        editor.cursor.col = 0;

        // Act
        editor.move_cursor(Direction::Down);

        // Assert
        assert_eq!(editor.cursor.row, 5);
    }

    // --- Modified flag ---

    #[test]
    fn test_set_key_color_marks_modified() {
        // Arrange
        let mut editor = create_test_editor();
        editor.cursor = RgbPos {
            row: 0,
            col: 0,
            half: Half::Left,
        };
        assert!(!editor.modified);

        // Act
        editor.set_key_color("RED");

        // Assert
        assert!(editor.modified);
    }

    #[test]
    fn test_mark_saved_clears_modified() {
        // Arrange
        let mut editor = create_test_editor();
        editor.cursor = RgbPos {
            row: 0,
            col: 0,
            half: Half::Left,
        };
        editor.set_key_color("RED");
        assert!(editor.modified);

        // Act
        editor.mark_saved();

        // Assert
        assert!(!editor.modified);
    }

    // --- Add layer ---

    #[test]
    fn test_add_layer_inserts_and_selects() {
        // Arrange
        let mut editor = create_test_editor();
        assert_eq!(editor.config.layers.len(), 1);

        // Act
        let result = editor.add_layer("NewLayer");

        // Assert
        assert!(result.is_ok());
        assert_eq!(editor.config.layers.len(), 2);
        assert_eq!(editor.current_layer, 1);
        assert_eq!(editor.config.layers[1].name, "NewLayer");
        assert_eq!(editor.config.layers[1].macro_name, "LAYER_NewLayer");
        // All keys should be "___"
        assert_eq!(editor.config.layers[1].left_half[0][0], "___");
    }

    #[test]
    fn test_add_layer_empty_name_returns_error() {
        // Arrange
        let mut editor = create_test_editor();

        // Act
        let result = editor.add_layer("");

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[test]
    fn test_add_layer_duplicate_name_returns_error() {
        // Arrange
        let mut editor = create_test_editor();

        // Act
        let result = editor.add_layer("Test");

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));
    }

    #[test]
    fn test_add_layer_invalid_chars_returns_error() {
        // Arrange
        let mut editor = create_test_editor();

        // Act
        let result = editor.add_layer("My Layer!");

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("letters, digits, and underscores"));
    }

    #[test]
    fn test_add_layer_marks_modified_and_pushes_undo() {
        // Arrange
        let mut editor = create_test_editor();
        editor.modified = false;

        // Act
        editor.add_layer("NewLayer").unwrap();

        // Assert
        assert!(editor.modified);
        assert!(editor.undo());
        assert_eq!(editor.config.layers.len(), 1);
    }

    // --- Duplicate layer ---

    #[test]
    fn test_duplicate_layer_clones_colors_and_fade() {
        // Arrange
        let mut editor = create_test_editor();
        editor.cursor = RgbPos { row: 0, col: 0, half: Half::Left };
        editor.set_key_color("RED");
        editor.current_layer_mut().unwrap().fade_delay = 42;
        editor.modified = false;

        // Act
        let result = editor.duplicate_layer();

        // Assert
        assert!(result.is_ok());
        let new_layer = &editor.config.layers[editor.current_layer];
        assert_eq!(new_layer.left_half[0][0], "RED");
        assert_eq!(new_layer.fade_delay, 42);
    }

    #[test]
    fn test_duplicate_layer_generates_unique_name() {
        // Arrange
        let mut editor = create_test_editor();

        // Act
        let name1 = editor.duplicate_layer().unwrap();

        // Assert
        assert_eq!(name1, "Test_copy");

        // Act — duplicate again (should get _copy_2)
        editor.current_layer = 0; // go back to original
        let name2 = editor.duplicate_layer().unwrap();

        // Assert
        assert_eq!(name2, "Test_copy_2");
    }

    #[test]
    fn test_duplicate_layer_with_no_layers_returns_error() {
        // Arrange
        let mut editor = create_test_editor();
        editor.config.layers.clear();

        // Act
        let result = editor.duplicate_layer();

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No layers"));
    }

    #[test]
    fn test_duplicate_layer_marks_modified_and_pushes_undo() {
        // Arrange
        let mut editor = create_test_editor();
        editor.modified = false;

        // Act
        editor.duplicate_layer().unwrap();

        // Assert
        assert!(editor.modified);
        assert!(editor.undo());
        assert_eq!(editor.config.layers.len(), 1);
    }

    // --- Rename layer ---

    #[test]
    fn test_rename_layer_updates_name_and_macro() {
        // Arrange
        let mut editor = create_test_editor();

        // Act
        let result = editor.rename_layer("Renamed");

        // Assert
        assert!(result.is_ok());
        assert_eq!(editor.config.layers[0].name, "Renamed");
        assert_eq!(editor.config.layers[0].macro_name, "LAYER_Renamed");
    }

    #[test]
    fn test_rename_layer_to_existing_name_returns_error() {
        // Arrange
        let mut editor = create_test_editor();
        editor.add_layer("Other").unwrap();

        // Act
        let result = editor.rename_layer("Test");

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));
    }

    #[test]
    fn test_rename_layer_same_name_succeeds() {
        // Arrange
        let mut editor = create_test_editor();
        editor.current_layer = 0;

        // Act
        let result = editor.rename_layer("Test");

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_rename_layer_marks_modified_and_pushes_undo() {
        // Arrange
        let mut editor = create_test_editor();
        editor.modified = false;

        // Act
        editor.rename_layer("Renamed").unwrap();

        // Assert
        assert!(editor.modified);
        assert!(editor.undo());
        assert_eq!(editor.config.layers[0].name, "Test");
    }

    // --- Delete layer ---

    #[test]
    fn test_delete_layer_removes_and_adjusts_index() {
        // Arrange
        let mut editor = create_test_editor();
        editor.add_layer("Second").unwrap();
        editor.add_layer("Third").unwrap();
        editor.current_layer = 1; // "Second"

        // Act
        let result = editor.delete_layer();

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Second");
        assert_eq!(editor.config.layers.len(), 2);
        assert_eq!(editor.current_layer, 1);
        assert_eq!(editor.config.layers[1].name, "Third");
    }

    #[test]
    fn test_delete_layer_last_selected_moves_to_previous() {
        // Arrange
        let mut editor = create_test_editor();
        editor.add_layer("Second").unwrap();
        editor.current_layer = 1; // "Second" — the last layer

        // Act
        let result = editor.delete_layer();

        // Assert
        assert!(result.is_ok());
        assert_eq!(editor.current_layer, 0);
    }

    #[test]
    fn test_delete_last_remaining_layer_returns_error() {
        // Arrange
        let mut editor = create_test_editor();

        // Act
        let result = editor.delete_layer();

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("last remaining"));
    }

    #[test]
    fn test_delete_layer_marks_modified_and_pushes_undo() {
        // Arrange
        let mut editor = create_test_editor();
        editor.add_layer("Second").unwrap();
        editor.modified = false;
        editor.current_layer = 1;

        // Act
        editor.delete_layer().unwrap();

        // Assert
        assert!(editor.modified);
        assert!(editor.undo());
        assert_eq!(editor.config.layers.len(), 2);
    }

    // --- Undo integration with layers ---

    #[test]
    fn test_undo_after_add_layer_removes_it() {
        // Arrange
        let mut editor = create_test_editor();
        editor.add_layer("NewLayer").unwrap();
        assert_eq!(editor.config.layers.len(), 2);

        // Act
        editor.undo();

        // Assert
        assert_eq!(editor.config.layers.len(), 1);
        assert_eq!(editor.config.layers[0].name, "Test");
    }

    #[test]
    fn test_undo_after_delete_layer_restores_it() {
        // Arrange
        let mut editor = create_test_editor();
        editor.add_layer("Second").unwrap();
        editor.current_layer = 1;
        editor.delete_layer().unwrap();
        assert_eq!(editor.config.layers.len(), 1);

        // Act
        editor.undo();

        // Assert
        assert_eq!(editor.config.layers.len(), 2);
        assert_eq!(editor.config.layers[1].name, "Second");
    }
}
