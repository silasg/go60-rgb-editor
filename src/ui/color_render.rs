use ratatui::{
    buffer::Buffer,
    style::{Color, Modifier, Style},
};

use crate::domain::{ColorPalette, RgbColor};

/// Convert an `RgbColor` to a ratatui `Color`.
fn to_ratatui_color(rgb: &RgbColor) -> Color {
    Color::Rgb(rgb.r, rgb.g, rgb.b)
}

/// Return a contrasting foreground color (black or white) for readability.
pub fn contrasting_fg(rgb: &RgbColor) -> Color {
    if rgb.luminance() > 128.0 {
        Color::Rgb(0, 0, 0)
    } else {
        Color::Rgb(255, 255, 255)
    }
}

/// Render a single color cell with selection pointers — shared by keyboard and color picker.
pub fn render_color_cell(
    buf: &mut Buffer, x: u16, y: u16, color_abbrev: &str, is_selected: bool, palette: &ColorPalette,
) {
    let style = if let Some(rgb) = palette.get_effective_rgb(color_abbrev) {
        Style::default()
            .bg(to_ratatui_color(rgb))
            .fg(contrasting_fg(rgb))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contrasting_fg_returns_true_black_and_white() {
        // Arrange
        let true_black = Color::Rgb(0, 0, 0);
        let true_white = Color::Rgb(255, 255, 255);

        // Act & Assert
        // Bright colors should get true black text
        let yellow = RgbColor::from_hex("FFFF00").unwrap();
        assert_eq!(contrasting_fg(&yellow), true_black);

        let white = RgbColor::from_hex("FFFFFF").unwrap();
        assert_eq!(contrasting_fg(&white), true_black);

        let cyan = RgbColor::from_hex("00FFFF").unwrap();
        assert_eq!(contrasting_fg(&cyan), true_black);

        // Dark colors should get true white text
        let black = RgbColor::from_hex("000000").unwrap();
        assert_eq!(contrasting_fg(&black), true_white);

        let dark_blue = RgbColor::from_hex("000080").unwrap();
        assert_eq!(contrasting_fg(&dark_blue), true_white);

        let purple = RgbColor::from_hex("7A00FF").unwrap();
        assert_eq!(contrasting_fg(&purple), true_white);
    }
}
