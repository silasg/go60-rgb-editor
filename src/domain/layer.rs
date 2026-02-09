use super::geometry::{MAIN_ROW_COLS, THUMB_ROW_COLS};

/// A keyboard layer with per-key RGB color assignments.
///
/// Each half has 6 rows: rows 0-3 are main keys (6 cols), rows 4-5 are thumb keys (3 cols).
/// Colors are stored as abbreviations (e.g., "RED", "CYN", "___" for off).
#[derive(Debug, Clone)]
pub struct Layer {
    /// e.g., "Cursor", "Symbol"
    pub name: String,
    /// #ifdef guard name, e.g., "LAYER_Cursor"
    pub macro_name: String,
    pub fade_delay: u16,
    /// [row][col] = color abbreviation
    pub left_half: Vec<Vec<String>>,
    /// [row][col] = color abbreviation
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

    pub fn get_color(&self, pos: &super::RgbPos) -> Option<&str> {
        let half = match pos.half {
            super::Half::Left => &self.left_half,
            super::Half::Right => &self.right_half,
        };
        half.get(pos.row).and_then(|r| r.get(pos.col)).map(|s| s.as_str())
    }

    pub fn set_color(&mut self, pos: &super::RgbPos, color: String) {
        let half = match pos.half {
            super::Half::Left => &mut self.left_half,
            super::Half::Right => &mut self.right_half,
        };
        if let Some(row_vec) = half.get_mut(pos.row) {
            if let Some(cell) = row_vec.get_mut(pos.col) {
                *cell = color;
            }
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::geometry;

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
        use crate::domain::{Half, RgbPos};

        // Arrange
        let mut layer = Layer::new("Test".to_string(), "LAYER_Test".to_string());
        let pos = RgbPos { row: 0, col: 0, half: Half::Left };
        let default_color = "___";

        // Act
        layer.set_color(&pos, "RED".to_string());

        // Assert
        assert_eq!(layer.get_color(&pos), Some("RED"));
        let right_pos = RgbPos { half: Half::Right, ..pos };
        assert_eq!(layer.get_color(&right_pos), Some(default_color));
    }

    #[test]
    fn test_cols_for_row() {
        let main_row_col_count = 6;
        let thumb_row_col_count = 3;

        // Act
        let first_main_row_cols = geometry::cols_for_row(0);
        let last_main_row_cols = geometry::cols_for_row(3);
        let inner_thumb_row_cols = geometry::cols_for_row(4);
        let outer_thumb_row_cols = geometry::cols_for_row(5);

        // Assert
        assert_eq!(first_main_row_cols, main_row_col_count);
        assert_eq!(last_main_row_cols, main_row_col_count);
        assert_eq!(inner_thumb_row_cols, thumb_row_col_count);
        assert_eq!(outer_thumb_row_cols, thumb_row_col_count);
    }

    #[test]
    fn test_is_valid_pos_for_main_rows() {
        // Act & Assert
        assert!(geometry::is_valid_pos(0, 0), "row 0 col 0 should be valid");
        assert!(geometry::is_valid_pos(0, 5), "row 0 col 5 should be valid");
        assert!(!geometry::is_valid_pos(0, 6), "row 0 col 6 should be out of bounds");
        assert!(geometry::is_valid_pos(3, 5), "row 3 col 5 should be valid");
        assert!(!geometry::is_valid_pos(3, 6), "row 3 col 6 should be out of bounds");
    }

    #[test]
    fn test_is_valid_pos_for_thumb_rows() {
        // Act & Assert
        assert!(geometry::is_valid_pos(4, 0), "row 4 col 0 should be valid");
        assert!(geometry::is_valid_pos(4, 2), "row 4 col 2 should be valid");
        assert!(!geometry::is_valid_pos(4, 3), "row 4 col 3 should be out of bounds");
        assert!(geometry::is_valid_pos(5, 2), "row 5 col 2 should be valid");
        assert!(!geometry::is_valid_pos(5, 3), "row 5 col 3 should be out of bounds");
    }

    #[test]
    fn test_is_valid_pos_for_out_of_bounds_row() {
        // Act & Assert
        assert!(!geometry::is_valid_pos(6, 0), "row 6 should be out of bounds");
        assert!(!geometry::is_valid_pos(100, 0), "row 100 should be out of bounds");
    }

    #[test]
    fn test_get_color_out_of_bounds_returns_none() {
        use crate::domain::{Half, RgbPos};

        // Arrange
        let layer = Layer::new("Test".to_string(), "LAYER_Test".to_string());

        // Act & Assert
        assert_eq!(
            layer.get_color(&RgbPos { row: 10, col: 0, half: Half::Left }), None,
            "getting color at an out-of-bounds row should return None"
        );
        assert_eq!(
            layer.get_color(&RgbPos { row: 0, col: 20, half: Half::Left }), None,
            "getting color at an out-of-bounds column should return None"
        );
    }

    #[test]
    fn test_set_color_out_of_bounds_does_not_panic() {
        use crate::domain::{Half, RgbPos};

        // Arrange
        let mut layer = Layer::new("Test".to_string(), "LAYER_Test".to_string());

        // Act & Assert (should not panic)
        layer.set_color(&RgbPos { row: 10, col: 0, half: Half::Left }, "RED".to_string());
        layer.set_color(&RgbPos { row: 0, col: 20, half: Half::Left }, "RED".to_string());
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
