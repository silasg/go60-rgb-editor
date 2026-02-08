use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Widget},
};

use crate::model::{ColorPalette, Half, Layer, RgbPos, ROW_COUNT, MAIN_ROW_COLS, THUMB_ROW_COLS};

const KEY_CELL_WIDTH: u16 = 4;
const HALF_GAP: u16 = 20;
const MIN_TERMINAL_WIDTH: u16 = 50;
const MIN_TERMINAL_HEIGHT: u16 = 8;

struct HalfRowPos {
    half: Half,
    row: usize,
    max_cols: usize,
    x: u16,
    y: u16,
}

/// Widget for rendering the keyboard layout with colors
pub struct KeyboardWidget<'a> {
    layer: &'a Layer,
    palette: &'a ColorPalette,
    cursor: RgbPos,
}

impl<'a> KeyboardWidget<'a> {
    pub fn new(layer: &'a Layer, palette: &'a ColorPalette, cursor: RgbPos) -> Self {
        Self {
            layer,
            palette,
            cursor,
        }
    }

    fn render_half_row(
        &self, buf: &mut Buffer, half_keys: &[Vec<String>], pos: &HalfRowPos,
    ) {
        for (col, color) in half_keys[pos.row].iter().enumerate().take(pos.max_cols) {
            let x = pos.x + col as u16 * KEY_CELL_WIDTH;
            let is_selected = self.cursor.half == pos.half
                && self.cursor.row == pos.row
                && self.cursor.col == col;
            self.render_key(buf, x, pos.y, color, is_selected);
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

    fn render_key(&self, buf: &mut Buffer, x: u16, y: u16, color_abbrev: &str, is_selected: bool) {
        // Get the effective RGB color (resolving aliases)
        let style = if let Some(rgb) = self.palette.get_effective_rgb(color_abbrev) {
            Style::default()
                .bg(rgb.to_ratatui_color())
                .fg(rgb.contrasting_fg())
        } else {
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
        };

        // Add bold for selected key
        let style = if is_selected {
            style.add_modifier(Modifier::BOLD)
        } else {
            style
        };

        // Display the color abbreviation (no special markers - legend explains special types)
        let display = format!("{:^3}", color_abbrev);

        // Render the key (3 chars wide)
        buf.set_string(x, y, &display, style);

        // Draw selection pointers around selected key
        if is_selected {
            let pointer_style = Style::default().fg(Color::Yellow);
            buf.set_string(x.saturating_sub(1), y, "▶", pointer_style);
            buf.set_string(x + 3, y, "◀", pointer_style);
        }
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
        let right_base_x = left_base_x + MAIN_ROW_COLS as u16 * KEY_CELL_WIDTH + HALF_GAP;
        let start_y = inner.y + 1;
        let center_shift = 2 * KEY_CELL_WIDTH;
        let thumb_width = THUMB_ROW_COLS as u16 * KEY_CELL_WIDTH;

        // Precompute X position and column count per row
        // Rows 0-3: full-width main rows, flush left/right
        // Row 4: inner thumb keys, shifted toward center
        // Row 5: outer thumb keys, shifted further toward center
        let left_x = [
            left_base_x, left_base_x, left_base_x, left_base_x,
            left_base_x + center_shift,
            left_base_x + center_shift + thumb_width,
        ];
        let right_x = [
            right_base_x, right_base_x, right_base_x, right_base_x,
            right_base_x + thumb_width - center_shift,
            right_base_x - center_shift,
        ];
        let cols = [
            MAIN_ROW_COLS, MAIN_ROW_COLS, MAIN_ROW_COLS, MAIN_ROW_COLS,
            THUMB_ROW_COLS, THUMB_ROW_COLS,
        ];

        for row in 0..ROW_COUNT {
            let y = start_y + row as u16;
            self.render_half_row(buf, &self.layer.left_half, &HalfRowPos { half: Half::Left, row, max_cols: cols[row], x: left_x[row], y });
            self.render_half_row(buf, &self.layer.right_half, &HalfRowPos { half: Half::Right, row, max_cols: cols[row], x: right_x[row], y });
        }

        // Selection info
        if inner.height > 8 {
            self.render_selection_info(buf, inner.x + 2, start_y + 8);
        }
    }
}
