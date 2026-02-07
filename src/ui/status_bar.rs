use ratatui::{
    prelude::*,
    widgets::Widget,
};

use crate::app::App;

/// Widget for the status bar
pub struct StatusBarWidget<'a> {
    app: &'a App,
}

impl<'a> StatusBarWidget<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }
}

impl<'a> Widget for StatusBarWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let modified = if self.app.modified { "[+]" } else { "" };
        let file_name = self.app.config.file_name();
        
        // Left side: file info
        let left = format!(" {} {} ", file_name, modified);
        
        // Right side: help hint (only show mode if not Normal)
        let mode_str = match self.app.mode {
            crate::app::Mode::Normal => "",
            crate::app::Mode::ColorPick => "COLOR  ",
            crate::app::Mode::Help => "HELP  ",
            crate::app::Mode::ConfirmQuit => "QUIT?  ",
            crate::app::Mode::ConfirmCopy => "COPY?  ",
        };
        
        let right = format!(" {}?:help  q:quit  s:save ", mode_str);
        
        // Status message if present
        let status_msg = self.app.status_message.as_ref().map(|(msg, _)| msg.as_str());
        
        // Render background
        let style = Style::default().bg(Color::DarkGray).fg(Color::White);
        for x in area.x..area.x + area.width {
            buf.set_string(x, area.y, " ", style);
        }
        
        // Render left side
        buf.set_string(area.x, area.y, &left, style);
        
        // Render status message in center if present
        if let Some(msg) = status_msg {
            let msg_x = area.x + area.width / 2 - msg.len() as u16 / 2;
            buf.set_string(
                msg_x,
                area.y,
                msg,
                Style::default().bg(Color::DarkGray).fg(Color::Yellow),
            );
        }
        
        // Render right side
        let right_x = area.x + area.width.saturating_sub(right.len() as u16);
        buf.set_string(right_x, area.y, &right, style);
    }
}
