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

        if inner.width < 20 || inner.height < 2 {
            return;
        }

        let key_width: u16 = 4;
        let cols = (inner.width / key_width) as usize;
        let max_cols = cols.min(14); // Limit to reasonable width

        for (i, color) in self.palette.colors.iter().enumerate() {
            let row = i / max_cols;
            let col = i % max_cols;

            if row as u16 >= inner.height {
                break;
            }

            let x = inner.x + col as u16 * key_width;
            let y = inner.y + row as u16;

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

            // Highlight selected color
            let style = if i == self.selected {
                style.add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                style
            };

            // Show special indicator for non-regular colors
            let display = match &color.kind {
                ColorKind::Regular => format!("{:^3}", color.abbrev),
                ColorKind::LockIndicator { .. } => format!("*{}", &color.abbrev[..2.min(color.abbrev.len())]),
                ColorKind::Alias { target } => format!("→{}", &target[..2.min(target.len())]),
            };

            buf.set_string(x, y, &display, style);
        }

        // Show quick select numbers for first 10 colors
        if inner.height > 3 {
            let hint_y = inner.y + inner.height - 1;
            let hint = "0-9: quick select";
            buf.set_string(inner.x, hint_y, hint, Style::default().fg(Color::DarkGray));
        }
    }
}
