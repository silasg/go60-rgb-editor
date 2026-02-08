use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Widget},
};

use crate::app::Cursor;
use crate::model::{ColorPalette, Half, Layer};

const KEY_CELL_WIDTH: u16 = 4;
const HALF_GAP: u16 = 20;
const MIN_TERMINAL_WIDTH: u16 = 50;
const MIN_TERMINAL_HEIGHT: u16 = 8;

/// Widget for rendering the keyboard layout with colors
pub struct KeyboardWidget<'a> {
    layer: &'a Layer,
    palette: &'a ColorPalette,
    cursor: Cursor,
}

impl<'a> KeyboardWidget<'a> {
    pub fn new(layer: &'a Layer, palette: &'a ColorPalette, cursor: Cursor) -> Self {
        Self {
            layer,
            palette,
            cursor,
        }
    }

    fn render_half_row(
        &self, buf: &mut Buffer, half_keys: &[Vec<String>],
        row: usize, start_x: u16, y: u16, max_cols: usize, half: Half,
    ) {
        if row < half_keys.len() {
            for col in 0..half_keys[row].len().min(max_cols) {
                let x = start_x + col as u16 * KEY_CELL_WIDTH;
                let color = &half_keys[row][col];
                let is_selected = self.cursor.half == half
                    && self.cursor.row == row
                    && self.cursor.col == col;
                self.render_key(buf, x, y, color, is_selected);
            }
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

        let half_width = 6 * KEY_CELL_WIDTH;
        let left_start_x = inner.x + 2;
        let right_start_x = left_start_x + half_width + HALF_GAP;
        let start_y = inner.y + 1;
        let center_shift = 2 * KEY_CELL_WIDTH;

        // Main rows (0-3)
        for row in 0..4 {
            let y = start_y + row as u16;
            self.render_half_row(buf, &self.layer.left_half, row, left_start_x, y, 6, Half::Left);
            self.render_half_row(buf, &self.layer.right_half, row, right_start_x, y, 6, Half::Right);
        }

        // Row 4 (inner thumb keys)
        let row4_y = start_y + 4;
        let left_row4_x = left_start_x + center_shift;
        let right_row4_x = right_start_x + 3 * KEY_CELL_WIDTH - center_shift;
        self.render_half_row(buf, &self.layer.left_half, 4, left_row4_x, row4_y, 3, Half::Left);
        self.render_half_row(buf, &self.layer.right_half, 4, right_row4_x, row4_y, 3, Half::Right);

        // Row 5 (outer thumb keys)
        let thumb_y = start_y + 5;
        let left_thumb_x = left_row4_x + 3 * KEY_CELL_WIDTH;
        let right_thumb_x = right_start_x - center_shift;
        self.render_half_row(buf, &self.layer.left_half, 5, left_thumb_x, thumb_y, 3, Half::Left);
        self.render_half_row(buf, &self.layer.right_half, 5, right_thumb_x, thumb_y, 3, Half::Right);

        // Selection info
        if inner.height > 8 {
            self.render_selection_info(buf, inner.x + 2, start_y + 8);
        }
    }
}
