/// Number of columns in the main key rows (rows 0–3)
pub const MAIN_ROW_COLS: usize = 6;
/// Number of columns in the thumb rows (rows 4–5)
pub const THUMB_ROW_COLS: usize = 3;
/// Total number of key rows per half (4 main + 2 thumb)
pub const ROW_COUNT: usize = 6;
/// First thumb row index
pub const THUMB_START_ROW: usize = 4;

/// Represents a single keyboard layer with RGB color assignments
#[derive(Debug, Clone)]
pub struct Layer {
    /// Display name (e.g., "Cursor", "Symbol")
    pub name: String,
    /// Macro name used in #ifdef (e.g., "LAYER_Cursor")
    pub macro_name: String,
    /// Fade delay in ms
    pub fade_delay: u16,
    /// Left half colors: [row][col] = color abbreviation
    /// Rows 0-3: main keys (6 cols each)
    /// Row 4: inner thumb keys (3 keys, indices 0-2)
    /// Row 5: outer thumb keys (3 keys, indices 0-2)
    pub left_half: Vec<Vec<String>>,
    /// Right half colors: same structure as left
    pub right_half: Vec<Vec<String>>,
}

impl Layer {
    pub fn new(name: String, macro_name: String) -> Self {
        let main_row = vec!["___".to_string(); MAIN_ROW_COLS];
        let thumb_row = vec!["___".to_string(); THUMB_ROW_COLS];

        Self {
            name,
            macro_name,
            fade_delay: 30,
            left_half: vec![
                main_row.clone(),
                main_row.clone(),
                main_row.clone(),
                main_row.clone(),
                thumb_row.clone(),
                thumb_row.clone(),
            ],
            right_half: vec![
                main_row.clone(),
                main_row.clone(),
                main_row.clone(),
                main_row,
                thumb_row.clone(),
                thumb_row,
            ],
        }
    }

    pub fn get_color(&self, row: usize, col: usize, is_left: bool) -> Option<&str> {
        let half = if is_left {
            &self.left_half
        } else {
            &self.right_half
        };
        half.get(row).and_then(|r| r.get(col)).map(|s| s.as_str())
    }

    pub fn set_color(&mut self, row: usize, col: usize, is_left: bool, color: String) {
        let half = if is_left {
            &mut self.left_half
        } else {
            &mut self.right_half
        };
        if let Some(row_vec) = half.get_mut(row) {
            if let Some(cell) = row_vec.get_mut(col) {
                *cell = color;
            }
        }
    }

    pub fn cols_for_row(row: usize) -> usize {
        if row < THUMB_START_ROW {
            MAIN_ROW_COLS
        } else {
            THUMB_ROW_COLS
        }
    }

    #[cfg(test)]
    pub fn is_valid_pos(row: usize, col: usize) -> bool {
        if row < THUMB_START_ROW {
            col < MAIN_ROW_COLS
        } else if row < ROW_COUNT {
            col < THUMB_ROW_COLS
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_new() {
        // Act
        let layer = Layer::new("Test".to_string(), "LAYER_Test".to_string());

        // Assert
        let expected_row_count = 6;
        let expected_main_row_cols = 6;
        let expected_thumb_row_cols = 3;
        assert_eq!(layer.left_half.len(), expected_row_count);
        assert_eq!(layer.right_half.len(), expected_row_count);
        assert_eq!(layer.left_half[0].len(), expected_main_row_cols);
        assert_eq!(layer.left_half[4].len(), expected_thumb_row_cols);
    }

    #[test]
    fn test_get_set_color() {
        // Arrange
        let mut layer = Layer::new("Test".to_string(), "LAYER_Test".to_string());
        let row = 0;
        let col = 0;
        let is_left_half = true;
        let default_color = "___";

        // Act
        layer.set_color(row, col, is_left_half, "RED".to_string());

        // Assert
        assert_eq!(layer.get_color(row, col, is_left_half), Some("RED"));
        assert_eq!(layer.get_color(row, col, !is_left_half), Some(default_color));
    }

    #[test]
    fn test_cols_for_row() {
        let main_row_col_count = 6;
        let thumb_row_col_count = 3;

        // Act
        let first_main_row_cols = Layer::cols_for_row(0);
        let last_main_row_cols = Layer::cols_for_row(3);
        let inner_thumb_row_cols = Layer::cols_for_row(4);
        let outer_thumb_row_cols = Layer::cols_for_row(5);

        // Assert
        assert_eq!(first_main_row_cols, main_row_col_count);
        assert_eq!(last_main_row_cols, main_row_col_count);
        assert_eq!(inner_thumb_row_cols, thumb_row_col_count);
        assert_eq!(outer_thumb_row_cols, thumb_row_col_count);
    }

    #[test]
    fn test_is_valid_pos_for_main_rows() {
        // Act & Assert
        assert!(Layer::is_valid_pos(0, 0), "row 0 col 0 should be valid");
        assert!(Layer::is_valid_pos(0, 5), "row 0 col 5 should be valid");
        assert!(!Layer::is_valid_pos(0, 6), "row 0 col 6 should be out of bounds");
        assert!(Layer::is_valid_pos(3, 5), "row 3 col 5 should be valid");
        assert!(!Layer::is_valid_pos(3, 6), "row 3 col 6 should be out of bounds");
    }

    #[test]
    fn test_is_valid_pos_for_thumb_rows() {
        // Act & Assert
        assert!(Layer::is_valid_pos(4, 0), "row 4 col 0 should be valid");
        assert!(Layer::is_valid_pos(4, 2), "row 4 col 2 should be valid");
        assert!(!Layer::is_valid_pos(4, 3), "row 4 col 3 should be out of bounds");
        assert!(Layer::is_valid_pos(5, 2), "row 5 col 2 should be valid");
        assert!(!Layer::is_valid_pos(5, 3), "row 5 col 3 should be out of bounds");
    }

    #[test]
    fn test_is_valid_pos_for_out_of_bounds_row() {
        // Act & Assert
        assert!(!Layer::is_valid_pos(6, 0), "row 6 should be out of bounds");
        assert!(!Layer::is_valid_pos(100, 0), "row 100 should be out of bounds");
    }

    #[test]
    fn test_get_color_out_of_bounds_returns_none() {
        // Arrange
        let layer = Layer::new("Test".to_string(), "LAYER_Test".to_string());

        // Act & Assert
        assert_eq!(
            layer.get_color(10, 0, true), None,
            "getting color at an out-of-bounds row should return None"
        );
        assert_eq!(
            layer.get_color(0, 20, true), None,
            "getting color at an out-of-bounds column should return None"
        );
    }

    #[test]
    fn test_set_color_out_of_bounds_does_not_panic() {
        // Arrange
        let mut layer = Layer::new("Test".to_string(), "LAYER_Test".to_string());

        // Act & Assert (should not panic)
        layer.set_color(10, 0, true, "RED".to_string());
        layer.set_color(0, 20, true, "RED".to_string());
    }

    #[test]
    fn test_default_fade_delay() {
        // Act
        let layer = Layer::new("Test".to_string(), "LAYER_Test".to_string());

        // Assert
        assert_eq!(
            layer.fade_delay, 30,
            "default fade delay should be 30ms"
        );
    }
}
