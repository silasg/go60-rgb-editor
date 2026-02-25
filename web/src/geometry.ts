// Go60 split keyboard physical layout — mirrors domain/src/geometry.rs

export const MAIN_ROW_COLS = 6;
export const THUMB_ROW_COLS = 3;
export const ROW_COUNT = 6;
export const THUMB_START_ROW = 4;

/** Number of data columns for the given row. */
export function colsForRow(row: number): number {
  return row < THUMB_START_ROW ? MAIN_ROW_COLS : THUMB_ROW_COLS;
}

/**
 * Visual column offsets per row (from domain geometry.rs).
 * Thumb rows are shifted toward the center of the split keyboard.
 */
const LEFT_OFFSETS = [0, 0, 0, 0, 2, 5];
const RIGHT_OFFSETS = [0, 0, 0, 0, 1, -2];

/**
 * CSS grid-column (1-indexed) for a key at (row, dataCol) in the given half.
 *
 * Layout uses 8 CSS grid columns per half.
 * - Left:  visual = dataCol + LEFT_OFFSETS[row];  gridCol = visual + 1
 * - Right: visual = dataCol + RIGHT_OFFSETS[row]; gridCol = visual + 3 (shift +2 to normalize negative offsets)
 */
export function gridColumn(half: 'left' | 'right', row: number, dataCol: number): number {
  if (half === 'left') {
    const visual = dataCol + LEFT_OFFSETS[row];
    return visual + 1;
  } else {
    const visual = dataCol + RIGHT_OFFSETS[row];
    return visual + 3; // +2 normalization, +1 for CSS 1-indexing
  }
}

/** CSS grid-row (1-indexed) for a key row. */
export function gridRow(row: number): number {
  return row + 1;
}

/** Total CSS grid columns per half. */
export const GRID_COLS = 8;
