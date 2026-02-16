use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{App, Mode};
use super::{
    ColorPickerWidget, HelpWidget, KeyboardWidget, LayerListWidget, StatusBarWidget,
};

const MODAL_WIDTH: u16 = 65;
const MODAL_HEIGHT: u16 = 8;

enum ModalSize {
    FitText,
    Fixed,
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

    let layer_widget = LayerListWidget::new(&app.editor.config.layers, app.editor.current_layer);
    frame.render_widget(layer_widget, main_layout[0]);

    let content_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),
            Constraint::Length(10),
            Constraint::Length(1),
        ])
        .split(main_layout[1]);

    if let Some(layer) = app.editor.config.layers.get(app.editor.current_layer) {
        let keyboard_widget = KeyboardWidget::new(layer, &app.editor.config.palette, app.editor.cursor);
        frame.render_widget(keyboard_widget, content_layout[0]);
    }

    let selected_color_idx = if matches!(app.mode, Mode::ColorPick) {
        app.color_picker.selected
    } else {
        app.get_current_color()
            .and_then(|color| app.editor.config.palette.abbrev_to_index.get(color).copied())
            .unwrap_or(0)
    };

    let color_picker = ColorPickerWidget::new(
        &app.editor.config.palette,
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
            let help_area = centered_rect_chars(100, 22, area);
            frame.render_widget(Clear, help_area);
            frame.render_widget(help, help_area);
        }
        Mode::ConfirmQuit => render_modal(frame, area,
            "You have unsaved changes.\n\nPress 'y' to quit, 'n' to cancel, 's' to save and quit",
            " Confirm Quit ", Color::Red, Alignment::Center, ModalSize::FitText,
        ),
        Mode::ConfirmCopy => render_modal(frame, area,
            "You have unsaved changes.\n\nPress 'y' to copy anyway, 'n' to cancel, 's' to save and copy",
            " Copy to Clipboard ", Color::Red, Alignment::Center, ModalSize::FitText,
        ),
        Mode::SaveAs => render_modal(frame, area,
            &format!("Enter filename:\n\n{}▌\n\n[Enter] Save  [Esc] Cancel  [Ctrl+U] Clear", &app.filename_input),
            " Save As ", Color::Yellow, Alignment::Left, ModalSize::Fixed,
        ),
        Mode::SaveAsConfirm => render_modal(frame, area,
            &format!("File already exists:\n{}\n\nOverwrite? [y] Yes  [n] Back  [Esc] Cancel", &app.filename_input),
            " Confirm Overwrite ", Color::Red, Alignment::Center, ModalSize::Fixed,
        ),
        Mode::AddLayer => render_modal(frame, area,
            &format!("Enter layer name:\n\n{}▌\n\n[Enter] Add  [Esc] Cancel  [Ctrl+U] Clear", &app.layer_name_input),
            " Add Layer ", Color::Yellow, Alignment::Left, ModalSize::Fixed,
        ),
        Mode::RenameLayer => render_modal(frame, area,
            &format!("Rename layer:\n\n{}▌\n\n[Enter] Rename  [Esc] Cancel  [Ctrl+U] Clear", &app.layer_name_input),
            " Rename Layer ", Color::Yellow, Alignment::Left, ModalSize::Fixed,
        ),
        Mode::ConfirmDelete => {
            let name = app.editor.current_layer().map(|l| l.name.as_str()).unwrap_or("?");
            render_modal(frame, area,
                &format!("Delete layer '{name}'?\n\n[y] Yes  [n] No"),
                " Delete Layer ", Color::Red, Alignment::Center, ModalSize::FitText,
            );
        }
        _ => {}
    }
}

fn render_modal(
    frame: &mut Frame, area: Rect, text: &str,
    title: &str, border_color: Color, alignment: Alignment, size: ModalSize,
) {
    let popup = Paragraph::new(text)
        .block(Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)))
        .alignment(alignment);

    let popup_area = match size {
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
