use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Widget},
};

use crate::domain::geometry::{self, ROW_COUNT};
use crate::domain::{ColorPalette, Half, RgbPos};
use super::render_color_cell;

const KEY_CELL_WIDTH: u16 = 4;
const HALF_GAP: u16 = 20;
const MIN_TERMINAL_WIDTH: u16 = 50;
const MIN_TERMINAL_HEIGHT: u16 = 8;
/// Vertical space between the first key row and the selection info line.
const SELECTION_INFO_Y_OFFSET: u16 = ROW_COUNT as u16 + 2;

struct RowRenderContext {
    half: Half,
    row: usize,
    max_cols: usize,
    x: u16,
    y: u16,
}

pub struct KeyboardWidget<'a> {
    layer: &'a crate::domain::Layer,
    palette: &'a ColorPalette,
    cursor: RgbPos,
}

impl<'a> KeyboardWidget<'a> {
    pub fn new(layer: &'a crate::domain::Layer, palette: &'a ColorPalette, cursor: RgbPos) -> Self {
        Self { layer, palette, cursor }
    }

    fn render_half_row(&self, buf: &mut Buffer, half_keys: &[Vec<String>], ctx: &RowRenderContext) {
        for (col, color) in half_keys[ctx.row].iter().enumerate().take(ctx.max_cols) {
            let x = ctx.x + col as u16 * KEY_CELL_WIDTH;
            let is_selected = self.cursor.half == ctx.half
                && self.cursor.row == ctx.row
                && self.cursor.col == col;
            render_color_cell(buf, x, ctx.y, color, is_selected, self.palette);
        }
    }

    fn render_selection_info(&self, buf: &mut Buffer, x: u16, y: u16) {
        let half_keys = match self.cursor.half {
            Half::Left => &self.layer.left_half,
            Half::Right => &self.layer.right_half,
        };
        let selected_color = half_keys.get(self.cursor.row).and_then(|r| r.get(self.cursor.col));

        if let Some(color) = selected_color {
            let half_label = match self.cursor.half {
                Half::Left => "L",
                Half::Right => "R",
            };
            let info = format!(
                "Selected: {} @ {}{},{} ",
                color, half_label, self.cursor.row, self.cursor.col
            );
            buf.set_string(x, y, info, Style::default().fg(Color::Cyan));
        }
    }
}

fn row_x_position(base_x: u16, half: Half, row: usize) -> u16 {
    let offset = geometry::row_offset(half, row);
    if offset >= 0 {
        base_x + offset as u16 * KEY_CELL_WIDTH
    } else {
        base_x.saturating_sub((-offset) as u16 * KEY_CELL_WIDTH)
    }
}

impl<'a> Widget for KeyboardWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(format!(" {} (fade: {}ms) ", self.layer.name, self.layer.fade_delay))
            .borders(Borders::ALL);
        let inner = block.inner(area);
        block.render(area, buf);

        if inner.width < MIN_TERMINAL_WIDTH || inner.height < MIN_TERMINAL_HEIGHT {
            buf.set_string(inner.x, inner.y, "Terminal too small", Style::default());
            return;
        }

        let left_base_x = inner.x + 2;
        let right_base_x = left_base_x + geometry::MAIN_ROW_COLS as u16 * KEY_CELL_WIDTH + HALF_GAP;
        let start_y = inner.y + 1;

        for row in 0..ROW_COUNT {
            let y = start_y + row as u16;
            let max_cols = geometry::cols_for_row(row);

            let left_x = row_x_position(left_base_x, Half::Left, row);
            let right_x = row_x_position(right_base_x, Half::Right, row);

            self.render_half_row(buf, &self.layer.left_half, &RowRenderContext { half: Half::Left, row, max_cols, x: left_x, y });
            self.render_half_row(buf, &self.layer.right_half, &RowRenderContext { half: Half::Right, row, max_cols, x: right_x, y });
        }

        if inner.height > SELECTION_INFO_Y_OFFSET {
            self.render_selection_info(buf, inner.x + 2, start_y + SELECTION_INFO_Y_OFFSET);
        }
    }
}
