use super::config::Config;
use super::cursor::{self, Direction};
use super::undo::UndoHistory;
use super::{Layer, RgbPos};

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

    /// Mark the editor state as saved (not modified).
    pub fn mark_saved(&mut self) {
        self.modified = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ColorPalette, Config, Half, Layer, RgbPos};

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
}
