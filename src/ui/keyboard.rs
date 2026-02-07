use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Widget},
};

use crate::app::Cursor;
use crate::model::{ColorPalette, Layer};

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

        if inner.width < 50 || inner.height < 8 {
            buf.set_string(inner.x, inner.y, "Terminal too small", Style::default());
            return;
        }

        let key_width: u16 = 4;  // 3 chars + 1 space
        let half_width = 6 * key_width;
        let gap = 20;  // Gap between halves (wider to accommodate last row shift)

        // Calculate starting positions
        let left_start_x = inner.x + 2;
        let right_start_x = left_start_x + half_width + gap;
        let start_y = inner.y + 1;

        // Render main rows (0-3)
        for row in 0..4 {
            let y = start_y + row as u16;

            // Left half
            if row < self.layer.left_half.len() {
                for col in 0..self.layer.left_half[row].len().min(6) {
                    let x = left_start_x + col as u16 * key_width;
                    let color = &self.layer.left_half[row][col];
                    let is_selected = self.cursor.is_left
                        && self.cursor.row == row
                        && self.cursor.col == col;
                    self.render_key(buf, x, y, color, is_selected);
                }
            }

            // Right half
            if row < self.layer.right_half.len() {
                for col in 0..self.layer.right_half[row].len().min(6) {
                    let x = right_start_x + col as u16 * key_width;
                    let color = &self.layer.right_half[row][col];
                    let is_selected = !self.cursor.is_left
                        && self.cursor.row == row
                        && self.cursor.col == col;
                    self.render_key(buf, x, y, color, is_selected);
                }
            }
        }

        // Render row 4 (outer 3 keys) - directly below main rows
        let row4_y = start_y + 4;
        let center_shift = 2 * key_width;  // Shift toward center
        
        // Left half row 4: shifted right toward center
        let left_row4_x = left_start_x + center_shift;

        if self.layer.left_half.len() > 4 {
            for col in 0..self.layer.left_half[4].len().min(3) {
                let x = left_row4_x + col as u16 * key_width;
                let color = &self.layer.left_half[4][col];
                let is_selected = self.cursor.is_left && self.cursor.row == 4 && self.cursor.col == col;
                self.render_key(buf, x, row4_y, color, is_selected);
            }
        }

        // Right half row 4: shifted left toward center (symmetric)
        let right_row4_x = right_start_x + 3 * key_width - center_shift;

        if self.layer.right_half.len() > 4 {
            for col in 0..self.layer.right_half[4].len().min(3) {
                let x = right_row4_x + col as u16 * key_width;
                let color = &self.layer.right_half[4][col];
                let is_selected = !self.cursor.is_left && self.cursor.row == 4 && self.cursor.col == col;
                self.render_key(buf, x, row4_y, color, is_selected);
            }
        }

        // Render row 5 (thumbs / inner 3 keys) - one line lower
        let thumb_y = start_y + 5;
        
        // Left half thumbs: after row4 position
        let left_thumb_x = left_row4_x + 3 * key_width;

        if self.layer.left_half.len() > 5 {
            for col in 0..self.layer.left_half[5].len().min(3) {
                let x = left_thumb_x + col as u16 * key_width;
                let color = &self.layer.left_half[5][col];
                let is_selected = self.cursor.is_left && self.cursor.row == 5 && self.cursor.col == col;
                self.render_key(buf, x, thumb_y, color, is_selected);
            }
        }

        // Right half thumbs: before row4 position (symmetric)
        let right_thumb_x = right_start_x - center_shift;

        if self.layer.right_half.len() > 5 {
            for col in 0..self.layer.right_half[5].len().min(3) {
                let x = right_thumb_x + col as u16 * key_width;
                let color = &self.layer.right_half[5][col];
                let is_selected = !self.cursor.is_left && self.cursor.row == 5 && self.cursor.col == col;
                self.render_key(buf, x, thumb_y, color, is_selected);
            }
        }

        // Render current selection info
        if inner.height > 8 {
            let info_y = start_y + 8;
            let selected_color = if self.cursor.is_left {
                self.layer.left_half.get(self.cursor.row).and_then(|r| r.get(self.cursor.col))
            } else {
                self.layer.right_half.get(self.cursor.row).and_then(|r| r.get(self.cursor.col))
            };

            if let Some(color) = selected_color {
                let half = if self.cursor.is_left { "L" } else { "R" };
                let info = format!(
                    "Selected: {} @ {}{},{} ",
                    color, half, self.cursor.row, self.cursor.col
                );
                buf.set_string(inner.x + 2, info_y, info, Style::default().fg(Color::Cyan));
            }
        }
    }
}
