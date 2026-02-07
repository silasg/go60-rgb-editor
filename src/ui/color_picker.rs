use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Widget},
};

use crate::model::{ColorKind, ColorPalette};

/// Widget for the color palette picker
pub struct ColorPickerWidget<'a> {
    palette: &'a ColorPalette,
    selected: usize,
    focused: bool,
}

impl<'a> ColorPickerWidget<'a> {
    pub fn new(palette: &'a ColorPalette, selected: usize, focused: bool) -> Self {
        Self {
            palette,
            selected,
            focused,
        }
    }

    fn render_color(&self, buf: &mut Buffer, x: u16, y: u16, idx: usize) {
        let color = &self.palette.colors[idx];

        // Get the effective RGB (resolving aliases)
        let effective_rgb = self.palette.get_effective_rgb(&color.abbrev);

        let style = if let Some(rgb) = effective_rgb {
            Style::default()
                .bg(rgb.to_ratatui_color())
                .fg(rgb.contrasting_fg())
        } else {
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
        };

        // Add bold for selected color
        let style = if idx == self.selected {
            style.add_modifier(Modifier::BOLD)
        } else {
            style
        };

        // Always show the abbreviation (max 3 chars)
        let display = format!("{:^3}", &color.abbrev[..color.abbrev.len().min(3)]);
        buf.set_string(x, y, &display, style);

        // Draw selection pointers around selected color
        if idx == self.selected {
            let pointer_style = Style::default().fg(Color::Yellow);
            buf.set_string(x.saturating_sub(1), y, "▶", pointer_style);
            buf.set_string(x + 3, y, "◀", pointer_style);
        }
    }
}

impl<'a> Widget for ColorPickerWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = if self.focused {
            " Colors [Enter to apply, Esc to cancel] "
        } else {
            " Colors [Enter to pick, 0-9 quick select] "
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(if self.focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            });

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.width < 20 || inner.height < 4 {
            return;
        }

        let key_width: u16 = 4;
        let color_start_x = inner.x + 1;  // Leading space for pointer
        let label_style = Style::default().fg(Color::DarkGray);

        // Separate colors by type
        let mut regular_colors: Vec<usize> = Vec::new();
        let mut lock_indicators: Vec<usize> = Vec::new();
        let mut aliases: Vec<usize> = Vec::new();

        for (i, color) in self.palette.colors.iter().enumerate() {
            match &color.kind {
                ColorKind::Regular => regular_colors.push(i),
                ColorKind::LockIndicator { .. } => lock_indicators.push(i),
                ColorKind::Alias { .. } => aliases.push(i),
            }
        }

        let mut current_y = inner.y;

        // Row 1-2: Regular colors (17 per row: RED to PNK, then WHT to LAC)
        let max_cols = 17;
        for (i, &idx) in regular_colors.iter().enumerate() {
            let row = i / max_cols;
            let col = i % max_cols;

            if row > 1 {
                break; // Limit to 2 rows of regular colors
            }

            let x = color_start_x + col as u16 * key_width;
            let y = current_y + row as u16;
            self.render_color(buf, x, y, idx);
        }
        current_y += 3; // 2 rows + 1 empty line

        // Lock indicators with label
        if !lock_indicators.is_empty() && current_y < inner.y + inner.height {
            buf.set_string(inner.x, current_y, "Lock:", label_style);
            let mut x = inner.x + 7;  // Extra space for pointer
            for &idx in &lock_indicators {
                self.render_color(buf, x, current_y, idx);
                x += key_width;
            }
            // Add explanation
            let explain_x = x + 1;
            buf.set_string(
                explain_x,
                current_y,
                "(CapsLock/NumLock/ScrollLock indicators)",
                label_style,
            );
            current_y += 1;
        }

        // Mouse speed aliases with label
        if !aliases.is_empty() && current_y < inner.y + inner.height {
            buf.set_string(inner.x, current_y, "Mouse:", label_style);
            let mut x = inner.x + 8;  // Extra space for pointer
            for &idx in &aliases {
                self.render_color(buf, x, current_y, idx);
                x += key_width;
            }
            // Add explanation
            let explain_x = x + 1;
            buf.set_string(
                explain_x,
                current_y,
                "(FST=Fast, WRP=Warp, SLO=Slow)",
                label_style,
            );
            current_y += 1;
        }

        // Bottom: quick select hint
        if current_y < inner.y + inner.height {
            buf.set_string(
                inner.x,
                inner.y + inner.height - 1,
                "0-9: quick select regular colors",
                label_style,
            );
        }
    }
}
