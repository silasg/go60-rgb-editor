mod keyboard;
mod layer_list;
mod color_picker;
mod status_bar;
mod help;

pub use keyboard::KeyboardWidget;
pub use layer_list::LayerListWidget;
pub use color_picker::ColorPickerWidget;
pub use status_bar::StatusBarWidget;
pub use help::HelpWidget;

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Clear},
};

use crate::app::{App, Mode};

/// Main UI drawing function
pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Main layout: sidebar (layers) | main content (keyboard + color picker)
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20), // Sidebar
            Constraint::Min(60),     // Main content
        ])
        .split(area);

    // Sidebar: layers list
    let layer_widget = LayerListWidget::new(&app.config.layers, app.current_layer);
    frame.render_widget(layer_widget, main_layout[0]);

    // Main content layout: keyboard + color picker + status
    let content_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),     // Keyboard (reduced height)
            Constraint::Length(10), // Color picker (more room for legend sections)
            Constraint::Length(1),  // Status bar
        ])
        .split(main_layout[1]);

    // Keyboard widget
    if let Some(layer) = app.config.layers.get(app.current_layer) {
        let keyboard_widget = KeyboardWidget::new(layer, &app.config.palette, app.cursor);
        frame.render_widget(keyboard_widget, content_layout[0]);
    }

    // Color picker - use app.selected_color when in ColorPick mode, 
    // otherwise show current key's color
    let selected_color_idx = if matches!(app.mode, Mode::ColorPick) {
        app.selected_color
    } else {
        app.get_current_color()
            .and_then(|color| app.config.palette.by_abbrev.get(color).copied())
            .unwrap_or(0)
    };
    
    let color_picker = ColorPickerWidget::new(
        &app.config.palette,
        selected_color_idx,
        matches!(app.mode, Mode::ColorPick),
    );
    frame.render_widget(color_picker, content_layout[1]);

    // Status bar
    let status = StatusBarWidget::new(app);
    frame.render_widget(status, content_layout[2]);

    // Modal overlays
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
            let text = "You have unsaved changes.\n\nPress 'y' to quit, 'n' to cancel, 's' to save and quit";
            render_modal(frame, text, " Confirm Quit ", None, area);
        }
        Mode::ConfirmCopy => {
            let text = "You have unsaved changes.\n\nPress 'y' to copy anyway, 'n' to cancel, 's' to save and copy";
            render_modal(frame, text, " Copy to Clipboard ", None, area);
        }
        Mode::SaveAs => {
            let text = format!(
                "Enter filename:\n\n{}▌\n\n[Enter] Save  [Esc] Cancel  [Ctrl+U] Clear",
                &app.filename_input
            );
            render_fixed_modal(frame, &text, " Save As ", Color::Yellow, Alignment::Left, 65, 8, area);
        }
        Mode::SaveAsConfirm => {
            let text = format!(
                "File already exists:\n{}\n\nOverwrite? [y] Yes  [n] Back  [Esc] Cancel",
                &app.filename_input
            );
            render_fixed_modal(frame, &text, " Confirm Overwrite ", Color::Red, Alignment::Center, 65, 8, area);
        }
        _ => {}
    }
}

fn render_modal(frame: &mut Frame, text: &str, title: &str, border_color: Option<Color>, area: Rect) {
    let mut block = Block::default().title(title).borders(Borders::ALL);
    if let Some(color) = border_color {
        block = block.border_style(Style::default().fg(color));
    }
    let popup = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);
    let popup_area = centered_rect_for_text(text, area);
    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup, popup_area);
}

#[allow(clippy::too_many_arguments)]
fn render_fixed_modal(
    frame: &mut Frame, text: &str, title: &str,
    border_color: Color, alignment: Alignment,
    width: u16, height: u16, area: Rect,
) {
    let popup = Paragraph::new(text)
        .block(Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)))
        .alignment(alignment);
    let popup_area = centered_rect_chars(width, height, area);
    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup, popup_area);
}

/// Create a centered rectangle with fixed character dimensions
fn centered_rect_chars(width: u16, height: u16, area: Rect) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}

/// Create a centered rectangle sized to fit the text content
fn centered_rect_for_text(text: &str, area: Rect) -> Rect {
    let lines: Vec<&str> = text.lines().collect();
    let max_line_width = lines.iter().map(|l| l.len()).max().unwrap_or(20);
    let height = lines.len();
    
    // Add padding for borders (2) and some margin (2)
    let width = (max_line_width + 4).min(area.width as usize) as u16;
    let height = (height + 4).min(area.height as usize) as u16;
    
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    
    Rect::new(x, y, width, height)
}


