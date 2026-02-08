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
        // Initialize with 6 rows, appropriate columns per row
        let main_row = vec!["___".to_string(); 6];
        let thumb_row = vec!["___".to_string(); 3];

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

    /// Get the color at a specific position
    pub fn get_color(&self, row: usize, col: usize, is_left: bool) -> Option<&str> {
        let half = if is_left {
            &self.left_half
        } else {
            &self.right_half
        };
        half.get(row).and_then(|r| r.get(col)).map(|s| s.as_str())
    }

    /// Set the color at a specific position
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

    /// Get the number of columns for a given row
    #[allow(dead_code)]
    pub fn cols_for_row(row: usize) -> usize {
        if row < 4 {
            6 // Main rows
        } else {
            3 // Thumb rows
        }
    }

    /// Check if a position is valid
    #[allow(dead_code)]
    pub fn is_valid_pos(row: usize, col: usize) -> bool {
        if row < 4 {
            col < 6
        } else if row < 6 {
            col < 3
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
}
