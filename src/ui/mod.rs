mod keyboard;
mod layer_list;
mod color_picker;
mod status_bar;
mod help;

pub use keyboard::KeyboardWidget;
pub use layer_list::LayerListWidget;
pub use color_picker::{ColorPickerWidget, COLORS_PER_PICKER_ROW};
pub use status_bar::StatusBarWidget;
pub use help::HelpWidget;

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Clear},
};

use crate::app::{App, Mode};
use crate::domain::ColorPalette;

const MODAL_WIDTH: u16 = 65;
const MODAL_HEIGHT: u16 = 8;

enum ModalSize {
    FitText,
    Fixed,
}

struct ModalStyle<'a> {
    title: &'a str,
    border_color: Color,
    alignment: Alignment,
    size: ModalSize,
}

/// Render a single color cell with selection pointers — shared by keyboard and color picker.
fn render_color_cell(
    buf: &mut Buffer, x: u16, y: u16, color_abbrev: &str, is_selected: bool, palette: &ColorPalette,
) {
    let style = if let Some(rgb) = palette.get_effective_rgb(color_abbrev) {
        Style::default()
            .bg(rgb.to_ratatui_color())
            .fg(rgb.contrasting_fg())
    } else {
        Style::default()
            .bg(Color::DarkGray)
            .fg(Color::White)
    };

    let style = if is_selected {
        style.add_modifier(Modifier::BOLD)
    } else {
        style
    };

    let display = format!("{:^3}", &color_abbrev[..color_abbrev.len().min(3)]);
    buf.set_string(x, y, &display, style);

    if is_selected {
        let pointer_style = Style::default().fg(Color::Yellow);
        buf.set_string(x.saturating_sub(1), y, "▶", pointer_style);
        buf.set_string(x + 3, y, "◀", pointer_style);
    }
}

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20),
            Constraint::Min(60),
        ])
        .split(area);

    let layer_widget = LayerListWidget::new(&app.config.layers, app.current_layer);
    frame.render_widget(layer_widget, main_layout[0]);

    let content_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),
            Constraint::Length(10),
            Constraint::Length(1),
        ])
        .split(main_layout[1]);

    if let Some(layer) = app.config.layers.get(app.current_layer) {
        let keyboard_widget = KeyboardWidget::new(layer, &app.config.palette, app.cursor);
        frame.render_widget(keyboard_widget, content_layout[0]);
    }

    let selected_color_idx = if matches!(app.mode, Mode::ColorPick) {
        app.selected_color
    } else {
        app.get_current_color()
            .and_then(|color| app.config.palette.abbrev_to_index.get(color).copied())
            .unwrap_or(0)
    };

    let color_picker = ColorPickerWidget::new(
        &app.config.palette,
        selected_color_idx,
        matches!(app.mode, Mode::ColorPick),
    );
    frame.render_widget(color_picker, content_layout[1]);

    let status = StatusBarWidget::new(app);
    frame.render_widget(status, content_layout[2]);

    render_modals(frame, app, area);
}

fn render_modals(frame: &mut Frame, app: &App, area: Rect) {
    match app.mode {
        Mode::Help => {
            let help = HelpWidget::new();
            let help_area = centered_rect_chars(58, 28, area);
            frame.render_widget(Clear, help_area);
            frame.render_widget(help, help_area);
        }
        Mode::ConfirmQuit => {
            render_modal(frame, "You have unsaved changes.\n\nPress 'y' to quit, 'n' to cancel, 's' to save and quit", &ModalStyle {
                title: " Confirm Quit ", border_color: Color::Red, alignment: Alignment::Center, size: ModalSize::FitText,
            }, area);
        }
        Mode::ConfirmCopy => {
            render_modal(frame, "You have unsaved changes.\n\nPress 'y' to copy anyway, 'n' to cancel, 's' to save and copy", &ModalStyle {
                title: " Copy to Clipboard ", border_color: Color::Red, alignment: Alignment::Center, size: ModalSize::FitText,
            }, area);
        }
        Mode::SaveAs => {
            let text = format!(
                "Enter filename:\n\n{}▌\n\n[Enter] Save  [Esc] Cancel  [Ctrl+U] Clear",
                &app.filename_input
            );
            render_modal(frame, &text, &ModalStyle {
                title: " Save As ", border_color: Color::Yellow, alignment: Alignment::Left, size: ModalSize::Fixed,
            }, area);
        }
        Mode::SaveAsConfirm => {
            let text = format!(
                "File already exists:\n{}\n\nOverwrite? [y] Yes  [n] Back  [Esc] Cancel",
                &app.filename_input
            );
            render_modal(frame, &text, &ModalStyle {
                title: " Confirm Overwrite ", border_color: Color::Red, alignment: Alignment::Center, size: ModalSize::Fixed,
            }, area);
        }
        _ => {}
    }
}

fn render_modal(frame: &mut Frame, text: &str, style: &ModalStyle, area: Rect) {
    let popup = Paragraph::new(text)
        .block(Block::default()
            .title(style.title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(style.border_color)))
        .alignment(style.alignment);

    let popup_area = match style.size {
        ModalSize::FitText => centered_rect_for_text(text, area),
        ModalSize::Fixed => centered_rect_chars(MODAL_WIDTH, MODAL_HEIGHT, area),
    };
    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup, popup_area);
}

fn centered_rect_chars(width: u16, height: u16, area: Rect) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}

fn centered_rect_for_text(text: &str, area: Rect) -> Rect {
    let lines: Vec<&str> = text.lines().collect();
    let max_line_width = lines.iter().map(|l| l.len()).max().unwrap_or(20);
    let height = lines.len();

    let width = (max_line_width + 4).min(area.width as usize) as u16;
    let height = (height + 4).min(area.height as usize) as u16;

    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;

    Rect::new(x, y, width, height)
}
