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
        let layer = Layer::new("Test".to_string(), "LAYER_Test".to_string());
        assert_eq!(layer.left_half.len(), 6);
        assert_eq!(layer.right_half.len(), 6);
        assert_eq!(layer.left_half[0].len(), 6);
        assert_eq!(layer.left_half[4].len(), 3);
    }

    #[test]
    fn test_get_set_color() {
        let mut layer = Layer::new("Test".to_string(), "LAYER_Test".to_string());
        layer.set_color(0, 0, true, "RED".to_string());
        assert_eq!(layer.get_color(0, 0, true), Some("RED"));
        assert_eq!(layer.get_color(0, 0, false), Some("___"));
    }

    #[test]
    fn test_cols_for_row() {
        assert_eq!(Layer::cols_for_row(0), 6);
        assert_eq!(Layer::cols_for_row(3), 6);
        assert_eq!(Layer::cols_for_row(4), 3);
        assert_eq!(Layer::cols_for_row(5), 3);
    }
}
