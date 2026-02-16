use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

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

const LEFT_COLUMN: &str = r#"NAVIGATION
  h/←, j/↓, k/↑, l/→    Move cursor
  Tab                   Switch left/right half

COLORS
  Enter                 Open color picker
  0-9                   Quick select color
  Esc                   Cancel color selection

EDITING
  u/Ctrl+r              Undo/Redo
  y                     Copy color at cursor
  p                     Paste color at cursor
  f/F                   Increase/Decrease fade
  Del/Backspace         Clear color"#;

const RIGHT_COLUMN: &str = r#"LAYERS
  J/K, PgDn/PgUp       Next/Previous layer
  a                     Add new layer
  d                     Duplicate current layer
  n                     Rename current layer
  x                     Delete current layer

FILE
  s/S                   Save / Save as
  c                     Copy file to clipboard

OTHER
  q/Q                   Quit / Force quit
  ?                     Show this help
  Esc                   Close popup / Cancel"#;

impl Widget for HelpWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render outer block
        let block = Block::default()
            .title(" Help ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        Widget::render(block, area, buf);

        // Split inner area into two columns
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(inner);

        let left = Paragraph::new(LEFT_COLUMN)
            .wrap(Wrap { trim: false });
        Widget::render(left, columns[0], buf);

        let right = Paragraph::new(RIGHT_COLUMN)
            .wrap(Wrap { trim: false });
        Widget::render(right, columns[1], buf);
    }
}
