use crate::geometry::{self, ROW_COUNT};
use crate::model::{Half, RgbPos};

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub fn move_cursor(cursor: &mut RgbPos, direction: Direction) {
    match direction {
        Direction::Up => {
            if cursor.row > 0 {
                let visual_col = geometry::to_visual_col(cursor.half, cursor.row, cursor.col);
                cursor.row -= 1;
                cursor.col = geometry::visual_to_data_col(cursor.half, cursor.row, visual_col);
            }
        }
        Direction::Down => {
            if cursor.row < ROW_COUNT - 1 {
                let visual_col = geometry::to_visual_col(cursor.half, cursor.row, cursor.col);
                cursor.row += 1;
                cursor.col = geometry::visual_to_data_col(cursor.half, cursor.row, visual_col);
            }
        }
        Direction::Left => {
            let max_col = geometry::cols_for_row(cursor.row);
            if cursor.col > 0 {
                cursor.col -= 1;
            } else if cursor.half == Half::Right {
                cursor.half = Half::Left;
                cursor.col = max_col - 1;
            }
        }
        Direction::Right => {
            let max_col = geometry::cols_for_row(cursor.row);
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
    let max_col = geometry::cols_for_row(cursor.row);
    if cursor.col >= max_col {
        cursor.col = max_col - 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
