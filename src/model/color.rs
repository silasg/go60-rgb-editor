use std::collections::HashMap;

/// The kind of color definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorKind {
    /// Regular RGB color (e.g., RED, CYN, ___)
    Regular,
    /// Lock indicator color (e.g., BSL, BNL, BCL) - has off/on states
    LockIndicator { off_color: String, on_color: String },
    /// Alias to another color (e.g., FST -> GOL)
    Alias { target: String },
}

/// RGB color value
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    #[allow(dead_code)]
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Parse hex color string like "0xFF0000" or "FF0000"
    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let hex = hex.trim_start_matches("0x").trim_start_matches('#');
        if hex.len() != 6 {
            return Err(format!("Invalid hex color length: {}", hex));
        }

        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|e| format!("Invalid red component: {}", e))?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|e| format!("Invalid green component: {}", e))?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|e| format!("Invalid blue component: {}", e))?;

        Ok(Self { r, g, b })
    }

    /// Convert to ratatui Color
    pub fn to_ratatui_color(&self) -> ratatui::style::Color {
        ratatui::style::Color::Rgb(self.r, self.g, self.b)
    }

    /// Calculate perceived luminance (0-255)
    pub fn luminance(&self) -> f64 {
        0.299 * self.r as f64 + 0.587 * self.g as f64 + 0.114 * self.b as f64
    }

    /// Get a contrasting foreground color (black or white)
    pub fn contrasting_fg(&self) -> ratatui::style::Color {
        if self.luminance() > 128.0 {
            ratatui::style::Color::Rgb(0, 0, 0)  // True black
        } else {
            ratatui::style::Color::Rgb(255, 255, 255)  // True white
        }
    }

    /// Format as hex string for output
    #[allow(dead_code)]
    pub fn to_hex(&self) -> String {
        format!("0x{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

/// A color definition with abbreviation, name, and RGB value
#[derive(Debug, Clone)]
pub struct ColorDef {
    /// Short abbreviation (e.g., "RED", "CYN", "___")
    pub abbrev: String,
    /// RGB suffix name (e.g., "RED_RGB")
    #[allow(dead_code)]
    pub rgb_name: String,
    /// The RGB color value
    pub rgb: RgbColor,
    /// Optional comment from the original file
    pub comment: Option<String>,
    /// The kind of color (regular, lock indicator, or alias)
    pub kind: ColorKind,
}

impl ColorDef {
    pub fn new(abbrev: String, rgb_name: String, rgb: RgbColor) -> Self {
        Self {
            abbrev,
            rgb_name,
            rgb,
            comment: None,
            kind: ColorKind::Regular,
        }
    }

    pub fn with_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }

    pub fn with_kind(mut self, kind: ColorKind) -> Self {
        self.kind = kind;
        self
    }

    /// Check if this is a special color (lock indicator or alias)
    #[allow(dead_code)]
    pub fn is_special(&self) -> bool {
        !matches!(self.kind, ColorKind::Regular)
    }
}

/// Collection of color definitions with lookup by abbreviation
#[derive(Debug, Clone, Default)]
pub struct ColorPalette {
    /// All color definitions in order
    pub colors: Vec<ColorDef>,
    /// Lookup index by abbreviation
    pub by_abbrev: HashMap<String, usize>,
}

impl ColorPalette {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, color: ColorDef) {
        let idx = self.colors.len();
        self.by_abbrev.insert(color.abbrev.clone(), idx);
        self.colors.push(color);
    }

    pub fn get(&self, abbrev: &str) -> Option<&ColorDef> {
        self.by_abbrev.get(abbrev).map(|&idx| &self.colors[idx])
    }

    /// Get the effective RGB color for an abbreviation, resolving aliases
    pub fn get_effective_rgb(&self, abbrev: &str) -> Option<&RgbColor> {
        let color = self.get(abbrev)?;
        match &color.kind {
            ColorKind::Regular => Some(&color.rgb),
            ColorKind::LockIndicator { on_color, .. } => {
                // Show the "on" color for lock indicators
                self.get(on_color).map(|c| &c.rgb)
            }
            ColorKind::Alias { target } => self.get_effective_rgb(target),
        }
    }

    /// Get all regular (non-special) colors for the picker
    #[allow(dead_code)]
    pub fn regular_colors(&self) -> Vec<&ColorDef> {
        self.colors
            .iter()
            .filter(|c| matches!(c.kind, ColorKind::Regular))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_from_hex() {
        // Arrange
        let hex_with_0x_prefix = "0xFF0000";
        let hex_without_prefix = "00FF00";
        let hex_with_hash_prefix = "#0000FF";

        // Act
        let red_from_0x = RgbColor::from_hex(hex_with_0x_prefix).unwrap();
        let green_from_plain = RgbColor::from_hex(hex_without_prefix).unwrap();
        let blue_from_hash = RgbColor::from_hex(hex_with_hash_prefix).unwrap();

        // Assert
        assert_eq!(red_from_0x, RgbColor::new(255, 0, 0));
        assert_eq!(green_from_plain, RgbColor::new(0, 255, 0));
        assert_eq!(blue_from_hash, RgbColor::new(0, 0, 255));
    }

    #[test]
    fn test_rgb_to_hex() {
        // Arrange
        let orange_color = RgbColor::new(255, 128, 0);

        // Act
        let hex_string = orange_color.to_hex();

        // Assert
        assert_eq!(hex_string, "0xFF8000");
    }

    #[test]
    fn test_luminance() {
        // Arrange
        let white = RgbColor::new(255, 255, 255);
        let black = RgbColor::new(0, 0, 0);
        let high_luminance_threshold = 200.0;
        let low_luminance_threshold = 1.0;

        // Act
        let white_luminance = white.luminance();
        let black_luminance = black.luminance();

        // Assert
        assert!(white_luminance > high_luminance_threshold);
        assert!(black_luminance < low_luminance_threshold);
    }

    #[test]
    fn test_contrasting_fg_returns_true_black_and_white() {
        use ratatui::style::Color;
        let true_black = Color::Rgb(0, 0, 0);
        let true_white = Color::Rgb(255, 255, 255);

        // Bright colors should get true black text
        // Arrange
        let yellow = RgbColor::new(255, 255, 0);
        // Act
        let yellow_fg = yellow.contrasting_fg();
        // Assert
        assert_eq!(yellow_fg, true_black);

        // Arrange
        let white = RgbColor::new(255, 255, 255);
        // Act
        let white_fg = white.contrasting_fg();
        // Assert
        assert_eq!(white_fg, true_black);

        // Arrange
        let cyan = RgbColor::new(0, 255, 255);
        // Act
        let cyan_fg = cyan.contrasting_fg();
        // Assert
        assert_eq!(cyan_fg, true_black);

        // Dark colors should get true white text
        // Arrange
        let black = RgbColor::new(0, 0, 0);
        // Act
        let black_fg = black.contrasting_fg();
        // Assert
        assert_eq!(black_fg, true_white);

        // Arrange
        let dark_blue = RgbColor::new(0, 0, 128);
        // Act
        let dark_blue_fg = dark_blue.contrasting_fg();
        // Assert
        assert_eq!(dark_blue_fg, true_white);

        // Arrange
        let purple = RgbColor::new(122, 0, 255);
        // Act
        let purple_fg = purple.contrasting_fg();
        // Assert
        assert_eq!(purple_fg, true_white);
    }

    #[test]
    fn test_palette() {
        // Arrange
        let mut palette = ColorPalette::new();
        let red_color = ColorDef::new("RED".to_string(), "RED_RGB".to_string(), RgbColor::new(255, 0, 0));
        let green_color = ColorDef::new("GRN".to_string(), "GRN_RGB".to_string(), RgbColor::new(0, 255, 0));

        // Act
        palette.add(red_color);
        palette.add(green_color);

        // Assert
        assert!(palette.get("RED").is_some());
        assert!(palette.get("GRN").is_some());
        assert!(palette.get("BLU").is_none());
    }
}
