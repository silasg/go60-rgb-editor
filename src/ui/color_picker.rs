use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
};

use go60_rgb_editor_domain::ColorPalette;
use go60_rgb_editor_domain::cursor::Direction;
use super::render_color_cell;

/// Number of colors per row in the color picker grid.
pub const COLORS_PER_PICKER_ROW: usize = 17;

#[derive(Default)]
pub struct ColorPickerState {
    pub selected: usize,
}

impl ColorPickerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn move_selection(&mut self, direction: Direction, palette: &ColorPalette) {
        let categories = palette.categorize();
        let current = self.selected;
        let cols = COLORS_PER_PICKER_ROW;

        let sections = [&categories.regular, &categories.locks, &categories.aliases];

        let (section_idx, pos) = match sections
            .iter()
            .enumerate()
            .find_map(|(i, s)| s.iter().position(|&x| x == current).map(|p| (i, p)))
        {
            Some(found) => found,
            None => return,
        };
        let section = sections[section_idx];

        match direction {
            Direction::Left => {
                self.selected = move_within_section(section, pos, -1);
            }
            Direction::Right => {
                self.selected = move_within_section(section, pos, 1);
            }
            Direction::Up => {
                self.selected =
                    jump_to_prev_section(current, sections, section_idx, pos, cols);
            }
            Direction::Down => {
                self.selected =
                    jump_to_next_section(current, sections, section_idx, pos, cols);
            }
        }
    }
}

fn move_within_section(section: &[usize], pos: usize, delta: isize) -> usize {
    let new_pos = pos as isize + delta;
    if new_pos >= 0 && (new_pos as usize) < section.len() {
        section[new_pos as usize]
    } else {
        section[pos]
    }
}

fn jump_to_prev_section(
    current: usize,
    sections: [&Vec<usize>; 3],
    section_idx: usize,
    pos: usize,
    cols: usize,
) -> usize {
    if section_idx == 0 {
        if pos >= cols {
            return sections[0][pos - cols];
        }
        return current;
    }
    let target = sections[section_idx - 1];
    if target.is_empty() {
        return current;
    }

    let target_pos = if section_idx - 1 == 0 {
        let last_row_start = (target.len() - 1) / cols * cols;
        last_row_start + pos
    } else {
        pos
    };
    target[target_pos.min(target.len() - 1)]
}

fn jump_to_next_section(
    current: usize,
    sections: [&Vec<usize>; 3],
    section_idx: usize,
    pos: usize,
    cols: usize,
) -> usize {
    if section_idx == 0 && pos + cols < sections[0].len() {
        return sections[0][pos + cols];
    }
    if section_idx + 1 >= sections.len() {
        return current;
    }

    let target = sections[section_idx + 1];
    if target.is_empty() {
        return current;
    }

    let target_pos = if section_idx == 0 { pos % cols } else { pos };
    target[target_pos.min(target.len() - 1)]
}

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

impl<'a> ColorPickerWidget<'a> {
    fn render_regular_colors(&self, buf: &mut Buffer, indices: &[usize], x: u16, y: u16) {
        for (i, &idx) in indices.iter().enumerate() {
            let row = i / COLORS_PER_PICKER_ROW;
            let col = i % COLORS_PER_PICKER_ROW;

            if row >= MAX_REGULAR_COLOR_ROWS {
                break;
            }

            let cell_x = x + col as u16 * KEY_CELL_WIDTH;
            let cell_y = y + row as u16;
            let abbrev = &self.palette.colors[idx].abbrev;
            render_color_cell(buf, cell_x, cell_y, abbrev, idx == self.selected, self.palette);
        }
    }
}

impl<'a> ColorPickerWidget<'a> {
    fn render_block(&self, area: Rect, buf: &mut Buffer) -> Option<Rect> {
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

        (inner.width >= 20 && inner.height >= 4).then_some(inner)
    }
}

impl<'a> Widget for ColorPickerWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let Some(inner) = self.render_block(area, buf) else {
            return;
        };

        let label_style = Style::default().fg(Color::DarkGray);
        let categories = self.palette.categorize();

        self.render_regular_colors(buf, &categories.regular, inner.x + 1, inner.y);
        let mut current_y = inner.y + MAX_REGULAR_COLOR_ROWS as u16 + 1;

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
