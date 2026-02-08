use crate::model::{Half, Layer, RgbPos, ROW_COUNT};

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// Convert data column to visual column, accounting for thumb row offsets.
pub fn to_visual_col(half: Half, row: usize, col: usize) -> usize {
    if half.is_left() {
        match row {
            0..=3 => col,
            4 => col + 2,     // Row 4 shifted 2 keys toward center
            5 => col + 5,     // Row 5 (thumbs) shifted 5 keys toward center
            _ => col,
        }
    } else {
        match row {
            0..=3 => col,
            4 => col + 1,     // Row 4 shifted toward center
            5 => col.saturating_sub(2), // Row 5 (thumbs) toward center
            _ => col,
        }
    }
}

/// Convert visual column back to data column, clamped to the valid range.
pub fn visual_to_data_col(half: Half, row: usize, visual_col: usize) -> usize {
    let max_col = Layer::cols_for_row(row);

    let data_col = if half.is_left() {
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

pub fn move_cursor(cursor: &mut RgbPos, direction: Direction) {
    match direction {
        Direction::Up => {
            if cursor.row > 0 {
                let visual_col = to_visual_col(cursor.half, cursor.row, cursor.col);
                cursor.row -= 1;
                cursor.col = visual_to_data_col(cursor.half, cursor.row, visual_col);
            }
        }
        Direction::Down => {
            if cursor.row < ROW_COUNT - 1 {
                let visual_col = to_visual_col(cursor.half, cursor.row, cursor.col);
                cursor.row += 1;
                cursor.col = visual_to_data_col(cursor.half, cursor.row, visual_col);
            }
        }
        Direction::Left => {
            let max_col = Layer::cols_for_row(cursor.row);
            if cursor.col > 0 {
                cursor.col -= 1;
            } else if cursor.half == Half::Right {
                cursor.half = Half::Left;
                cursor.col = max_col - 1;
            }
        }
        Direction::Right => {
            let max_col = Layer::cols_for_row(cursor.row);
            if cursor.col < max_col - 1 {
                cursor.col += 1;
            } else if cursor.half == Half::Left {
                cursor.half = Half::Right;
                cursor.col = 0;
            }
        }
    }
}

pub fn switch_half(cursor: &mut RgbPos) {
    cursor.half = cursor.half.opposite();
    clamp_col(cursor);
}

fn clamp_col(cursor: &mut RgbPos) {
    let max_col = Layer::cols_for_row(cursor.row);
    if cursor.col >= max_col {
        cursor.col = max_col - 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visual_col_left_main_rows_no_offset() {
        assert_eq!(to_visual_col(Half::Left, 0, 0), 0);
        assert_eq!(to_visual_col(Half::Left, 3, 5), 5);
    }

    #[test]
    fn test_visual_col_left_row4_shifted_by_2() {
        assert_eq!(to_visual_col(Half::Left, 4, 0), 2);
        assert_eq!(to_visual_col(Half::Left, 4, 2), 4);
    }

    #[test]
    fn test_visual_col_left_row5_shifted_by_5() {
        assert_eq!(to_visual_col(Half::Left, 5, 0), 5);
        assert_eq!(to_visual_col(Half::Left, 5, 2), 7);
    }

    #[test]
    fn test_visual_col_right_row4_shifted_by_1() {
        assert_eq!(to_visual_col(Half::Right, 4, 0), 1);
        assert_eq!(to_visual_col(Half::Right, 4, 2), 3);
    }

    #[test]
    fn test_visual_to_data_col_roundtrip_left() {
        for row in 0..ROW_COUNT {
            let max = Layer::cols_for_row(row);
            for col in 0..max {
                let visual = to_visual_col(Half::Left, row, col);
                let back = visual_to_data_col(Half::Left, row, visual);
                assert_eq!(back, col, "roundtrip failed for left row={} col={}", row, col);
            }
        }
    }

    #[test]
    fn test_move_cursor_up_at_top_stays() {
        let mut cursor = RgbPos { row: 0, col: 3, half: Half::Left };
        move_cursor(&mut cursor, Direction::Up);
        assert_eq!(cursor.row, 0);
    }

    #[test]
    fn test_move_cursor_down_at_bottom_stays() {
        let mut cursor = RgbPos { row: 5, col: 0, half: Half::Left };
        move_cursor(&mut cursor, Direction::Down);
        assert_eq!(cursor.row, 5);
    }

    #[test]
    fn test_move_cursor_left_wraps_to_left_half() {
        let mut cursor = RgbPos { row: 0, col: 0, half: Half::Right };
        move_cursor(&mut cursor, Direction::Left);
        assert_eq!(cursor.half, Half::Left);
        assert_eq!(cursor.col, 5);
    }

    #[test]
    fn test_move_cursor_right_wraps_to_right_half() {
        let mut cursor = RgbPos { row: 0, col: 5, half: Half::Left };
        move_cursor(&mut cursor, Direction::Right);
        assert_eq!(cursor.half, Half::Right);
        assert_eq!(cursor.col, 0);
    }

    #[test]
    fn test_switch_half_clamps_col() {
        let mut cursor = RgbPos { row: 4, col: 5, half: Half::Left };
        // col 5 is out of range for row 4 (max 3 cols), switch should clamp
        switch_half(&mut cursor);
        assert_eq!(cursor.half, Half::Right);
        assert!(cursor.col <= 2);
    }
}
