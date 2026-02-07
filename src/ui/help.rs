use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

/// Widget for the help popup
pub struct HelpWidget;

impl HelpWidget {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HelpWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for HelpWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let help_text = r#"
NAVIGATION
  h/←, j/↓, k/↑, l/→    Move cursor
  Tab                   Switch between left/right half
  J/K                   Next/Previous layer

COLORS
  Enter                 Open color picker
  0-9                   Quick select color (first 10)
  Esc                   Cancel color selection

EDITING
  u                     Undo
  Ctrl+r                Redo
  y                     Copy color at cursor
  p                     Paste color at cursor
  f/F                   Increase/Decrease fade duration

FILE
  s                     Save
  S                     Save as
  q                     Quit (prompts if unsaved)
  Q                     Force quit without saving

OTHER
  ?                     Show this help
  Esc                   Close popup / Cancel
"#;

        let paragraph = Paragraph::new(help_text.trim())
            .block(
                Block::default()
                    .title(" Help ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .wrap(Wrap { trim: false });

        Widget::render(paragraph, area, buf);
    }
}
