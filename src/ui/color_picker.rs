use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Widget},
};

use crate::model::ColorPalette;
use super::render_color_cell;

/// Number of colors per row in the color picker grid.
pub const COLORS_PER_PICKER_ROW: usize = 17;

const KEY_CELL_WIDTH: u16 = 4;
/// Maximum rows of regular colors shown before the lock/alias sections
const MAX_REGULAR_COLOR_ROWS: usize = 2;

struct LabeledSection<'a> {
    indices: &'a [usize],
    label: &'a str,
    label_width: u16,
    explanation: &'a str,
    style: Style,
}

pub struct ColorPickerWidget<'a> {
    palette: &'a ColorPalette,
    selected: usize,
    focused: bool,
}

impl<'a> ColorPickerWidget<'a> {
    pub fn new(palette: &'a ColorPalette, selected: usize, focused: bool) -> Self {
        Self { palette, selected, focused }
    }

    fn render_labeled_section(
        &self, buf: &mut Buffer, section: &LabeledSection, inner: Rect, y: u16,
    ) -> u16 {
        if section.indices.is_empty() || y >= inner.y + inner.height {
            return y;
        }
        buf.set_string(inner.x, y, section.label, section.style);
        let mut x = inner.x + section.label_width;
        for &idx in section.indices {
            let is_selected = idx == self.selected;
            let abbrev = &self.palette.colors[idx].abbrev;
            render_color_cell(buf, x, y, abbrev, is_selected, self.palette);
            x += KEY_CELL_WIDTH;
        }
        buf.set_string(x + 1, y, section.explanation, section.style);
        y + 1
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

        let color_start_x = inner.x + 1;
        let label_style = Style::default().fg(Color::DarkGray);

        let categories = self.palette.categorize();

        let mut current_y = inner.y;

        let max_cols = COLORS_PER_PICKER_ROW;
        for (i, &idx) in categories.regular.iter().enumerate() {
            let row = i / max_cols;
            let col = i % max_cols;

            if row >= MAX_REGULAR_COLOR_ROWS {
                break;
            }

            let x = color_start_x + col as u16 * KEY_CELL_WIDTH;
            let y = current_y + row as u16;
            let is_selected = idx == self.selected;
            let abbrev = &self.palette.colors[idx].abbrev;
            render_color_cell(buf, x, y, abbrev, is_selected, self.palette);
        }
        current_y += MAX_REGULAR_COLOR_ROWS as u16 + 1;

        current_y = self.render_labeled_section(
            buf,
            &LabeledSection {
                indices: &categories.locks,
                label: "Lock:",
                label_width: 7,
                explanation: "(CapsLock/NumLock/ScrollLock indicators)",
                style: label_style,
            },
            inner, current_y,
        );

        // Currently all aliases are mouse speed colors; update label if aliases expand.
        current_y = self.render_labeled_section(
            buf,
            &LabeledSection {
                indices: &categories.aliases,
                label: "Mouse:",
                label_width: 8,
                explanation: "(FST=Fast, WRP=Warp, SLO=Slow)",
                style: label_style,
            },
            inner, current_y,
        );

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
