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
    match app.mode {
        Mode::Help => {
            let help = HelpWidget::new();
            let help_area = centered_rect(62, 80, area);
            frame.render_widget(Clear, help_area);
            frame.render_widget(help, help_area);
        }
        Mode::ConfirmQuit => {
            let popup = Paragraph::new("You have unsaved changes.\n\nPress 'y' to quit, 'n' to cancel, 's' to save and quit")
                .block(Block::default().title("Confirm Quit").borders(Borders::ALL))
                .alignment(Alignment::Center);
            let popup_area = centered_rect(50, 20, area);
            frame.render_widget(Clear, popup_area);
            frame.render_widget(popup, popup_area);
        }
        _ => {}
    }
}

/// Create a centered rectangle within the given area
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
