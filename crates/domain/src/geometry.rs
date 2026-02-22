//! Physical layout of the Go60 split keyboard.
//!
//! Each half has 6 rows: 4 main rows (6 keys each) and 2 thumb rows (3 keys each).
//! The thumb rows are visually offset toward the center of the keyboard.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Half {
    #[default]
    Left,
    Right,
}

impl Half {
    pub fn opposite(self) -> Self {
        match self {
            Half::Left => Half::Right,
            Half::Right => Half::Left,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RgbPos {
    pub row: usize,
    pub col: usize,
    pub half: Half,
}

/// Columns in main key rows (rows 0–3).
pub const MAIN_ROW_COLS: usize = 6;
/// Columns in thumb rows (rows 4–5).
pub const THUMB_ROW_COLS: usize = 3;
/// Total rows per half (4 main + 2 thumb).
pub const ROW_COUNT: usize = 6;
/// Index of the first thumb row.
pub const THUMB_START_ROW: usize = 4;

/// Visual column offsets per row.
/// Thumb rows are shifted toward the center of the split keyboard.
const LEFT_OFFSETS: [isize; ROW_COUNT] = [0, 0, 0, 0, 2, 5];
const RIGHT_OFFSETS: [isize; ROW_COUNT] = [0, 0, 0, 0, 1, -2];

/// Number of data columns for the given row.
pub fn cols_for_row(row: usize) -> usize {
    if row < THUMB_START_ROW {
        MAIN_ROW_COLS
    } else {
        THUMB_ROW_COLS
    }
}

/// Whether the given (row, col) is within the keyboard bounds.
#[cfg(test)]
pub fn is_valid_pos(row: usize, col: usize) -> bool {
    row < ROW_COUNT && col < cols_for_row(row)
}

/// The visual column offset for a given half and row.
/// Positive = shifted toward center, negative = shifted outward.
pub fn row_offset(half: Half, row: usize) -> isize {
    if row >= ROW_COUNT {
        return 0;
    }
    match half {
        Half::Left => LEFT_OFFSETS[row],
        Half::Right => RIGHT_OFFSETS[row],
    }
}

/// Map a data column to its visual column (for consistent vertical cursor movement).
pub fn to_visual_col(half: Half, row: usize, col: usize) -> usize {
    apply_offset(col, row_offset(half, row))
}

/// Map a visual column back to a data column, clamped to the row's valid range.
pub fn visual_to_data_col(half: Half, row: usize, visual_col: usize) -> usize {
    let data_col = apply_offset(visual_col, -row_offset(half, row));
    data_col.min(cols_for_row(row) - 1)
}

fn apply_offset(value: usize, offset: isize) -> usize {
    if offset >= 0 {
        value + offset as usize
    } else {
        value.saturating_sub((-offset) as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cols_for_row() {
        // Act & Assert
        assert_eq!(cols_for_row(0), MAIN_ROW_COLS);
        assert_eq!(cols_for_row(3), MAIN_ROW_COLS);
        assert_eq!(cols_for_row(4), THUMB_ROW_COLS);
        assert_eq!(cols_for_row(5), THUMB_ROW_COLS);
    }

    #[test]
    fn test_visual_col_left_main_rows_no_offset() {
        // Act & Assert
        assert_eq!(to_visual_col(Half::Left, 0, 0), 0);
        assert_eq!(to_visual_col(Half::Left, 3, 5), 5);
    }

    #[test]
    fn test_visual_col_left_row4_shifted_by_2() {
        // Act & Assert
        assert_eq!(to_visual_col(Half::Left, 4, 0), 2);
        assert_eq!(to_visual_col(Half::Left, 4, 2), 4);
    }

    #[test]
    fn test_visual_col_left_row5_shifted_by_5() {
        // Act & Assert
        assert_eq!(to_visual_col(Half::Left, 5, 0), 5);
        assert_eq!(to_visual_col(Half::Left, 5, 2), 7);
    }

    #[test]
    fn test_visual_col_right_row4_shifted_by_1() {
        // Act & Assert
        assert_eq!(to_visual_col(Half::Right, 4, 0), 1);
        assert_eq!(to_visual_col(Half::Right, 4, 2), 3);
    }

    #[test]
    fn test_visual_col_roundtrip_left() {
        // Act & Assert
        for row in 0..ROW_COUNT {
            let max = cols_for_row(row);
            for col in 0..max {
                let visual = to_visual_col(Half::Left, row, col);
                let back = visual_to_data_col(Half::Left, row, visual);
                assert_eq!(back, col, "roundtrip failed for left row={row} col={col}");
            }
        }
    }

    #[test]
    fn test_main_rows_have_no_offset() {
        // Act & Assert
        for row in 0..4 {
            assert_eq!(row_offset(Half::Left, row), 0);
            assert_eq!(row_offset(Half::Right, row), 0);
        }
    }

    #[test]
    fn test_out_of_range_row_returns_zero_offset() {
        // Act & Assert
        assert_eq!(row_offset(Half::Left, 99), 0);
        assert_eq!(row_offset(Half::Right, 99), 0);
    }
}
