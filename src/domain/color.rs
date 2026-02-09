use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorKind {
    /// e.g., RED, CYN, ___
    Regular,
    /// Lock indicator with off/on states, e.g., BSL, BNL, BCL
    LockIndicator { off_color: String, on_color: String },
    /// Alias to another color, e.g., FST -> GOL
    Alias { target: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    #[cfg(test)]
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Parse "0xFF0000", "#FF0000", or "FF0000".
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

    pub fn to_ratatui_color(&self) -> ratatui::style::Color {
        ratatui::style::Color::Rgb(self.r, self.g, self.b)
    }

    pub fn luminance(&self) -> f64 {
        0.299 * self.r as f64 + 0.587 * self.g as f64 + 0.114 * self.b as f64
    }

    pub fn contrasting_fg(&self) -> ratatui::style::Color {
        if self.luminance() > 128.0 {
            ratatui::style::Color::Rgb(0, 0, 0)  // True black
        } else {
            ratatui::style::Color::Rgb(255, 255, 255)  // True white
        }
    }

    #[cfg(test)]
    pub fn to_hex(&self) -> String {
        format!("0x{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

#[derive(Debug, Clone)]
pub struct ColorDef {
    /// e.g., "RED", "CYN", "___"
    pub abbrev: String,
    pub rgb: RgbColor,
    pub comment: Option<String>,
    pub kind: ColorKind,
}

impl ColorDef {
    pub fn new(abbrev: String, rgb: RgbColor) -> Self {
        Self {
            abbrev,
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
}

#[derive(Debug, Clone, Default)]
pub struct ColorPalette {
    pub colors: Vec<ColorDef>,
    pub abbrev_to_index: HashMap<String, usize>,
}

impl ColorPalette {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, color: ColorDef) {
        let idx = self.colors.len();
        self.abbrev_to_index.insert(color.abbrev.clone(), idx);
        self.colors.push(color);
    }

    pub fn get(&self, abbrev: &str) -> Option<&ColorDef> {
        self.abbrev_to_index.get(abbrev).map(|&idx| &self.colors[idx])
    }

    /// Resolve aliases and lock indicators to their effective RGB value.
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

    /// Categorize palette colors by kind, returning indices into `self.colors`.
    pub fn categorize(&self) -> CategorizedColors {
        let mut categories = CategorizedColors::default();
        for (i, color) in self.colors.iter().enumerate() {
            match &color.kind {
                ColorKind::Regular => categories.regular.push(i),
                ColorKind::LockIndicator { .. } => categories.locks.push(i),
                ColorKind::Alias { .. } => categories.aliases.push(i),
            }
        }
        categories
    }

    #[cfg(test)]
    pub fn regular_colors(&self) -> Vec<&ColorDef> {
        self.colors
            .iter()
            .filter(|c| matches!(c.kind, ColorKind::Regular))
            .collect()
    }
}

/// Color indices grouped by kind.
#[derive(Default)]
pub struct CategorizedColors {
    pub regular: Vec<usize>,
    pub locks: Vec<usize>,
    pub aliases: Vec<usize>,
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
        let red_color = ColorDef::new("RED".to_string(), RgbColor::new(255, 0, 0));
        let green_color = ColorDef::new("GRN".to_string(), RgbColor::new(0, 255, 0));

        // Act
        palette.add(red_color);
        palette.add(green_color);

        // Assert
        assert!(palette.get("RED").is_some());
        assert!(palette.get("GRN").is_some());
        assert!(palette.get("BLU").is_none());
    }

    #[test]
    fn test_get_effective_rgb_for_regular_color() {
        // Arrange
        let mut palette = ColorPalette::new();
        let red_rgb = RgbColor::new(255, 0, 0);
        palette.add(ColorDef::new("RED".to_string(), red_rgb.clone()));

        // Act
        let effective_rgb = palette.get_effective_rgb("RED");

        // Assert
        assert_eq!(
            effective_rgb, Some(&red_rgb),
            "effective RGB for a regular color should be its own RGB value"
        );
    }

    #[test]
    fn test_get_effective_rgb_for_alias_resolves_to_target() {
        // Arrange
        let mut palette = ColorPalette::new();
        let gold_rgb = RgbColor::new(255, 215, 0);
        palette.add(ColorDef::new("GOL".to_string(), gold_rgb.clone()));
        let alias = ColorDef::new("FST".to_string(), RgbColor::default())
            .with_kind(ColorKind::Alias { target: "GOL".to_string() });
        palette.add(alias);

        // Act
        let effective_rgb = palette.get_effective_rgb("FST");

        // Assert
        assert_eq!(
            effective_rgb, Some(&gold_rgb),
            "effective RGB for an alias should resolve to the target's RGB value"
        );
    }

    #[test]
    fn test_get_effective_rgb_for_lock_indicator_returns_on_color() {
        // Arrange
        let mut palette = ColorPalette::new();
        let red_rgb = RgbColor::new(255, 0, 0);
        palette.add(ColorDef::new("RED".to_string(), red_rgb.clone()));
        let black_rgb = RgbColor::new(0, 0, 0);
        palette.add(ColorDef::new("BLK".to_string(), black_rgb));
        let lock_indicator = ColorDef::new("BSL".to_string(), red_rgb.clone())
            .with_kind(ColorKind::LockIndicator {
                off_color: "BLK".to_string(),
                on_color: "RED".to_string(),
            });
        palette.add(lock_indicator);

        // Act
        let effective_rgb = palette.get_effective_rgb("BSL");

        // Assert
        assert_eq!(
            effective_rgb, Some(&red_rgb),
            "effective RGB for a lock indicator should be the 'on' color's RGB"
        );
    }

    #[test]
    fn test_get_effective_rgb_for_unknown_abbreviation_returns_none() {
        // Arrange
        let palette = ColorPalette::new();

        // Act
        let effective_rgb = palette.get_effective_rgb("NONEXISTENT");

        // Assert
        assert_eq!(
            effective_rgb, None,
            "effective RGB for an unknown abbreviation should return None"
        );
    }

    #[test]
    fn test_regular_colors_excludes_special_colors() {
        // Arrange
        let mut palette = ColorPalette::new();
        let red = ColorDef::new("RED".to_string(), RgbColor::new(255, 0, 0));
        let alias = ColorDef::new("FST".to_string(), RgbColor::default())
            .with_kind(ColorKind::Alias { target: "RED".to_string() });
        let lock = ColorDef::new("BSL".to_string(), RgbColor::new(255, 0, 0))
            .with_kind(ColorKind::LockIndicator {
                off_color: "BLK".to_string(),
                on_color: "RED".to_string(),
            });
        palette.add(red);
        palette.add(alias);
        palette.add(lock);

        // Act
        let regular_colors = palette.regular_colors();

        // Assert
        assert_eq!(
            regular_colors.len(), 1,
            "regular_colors should only include regular colors, not aliases or lock indicators"
        );
        assert_eq!(regular_colors[0].abbrev, "RED");
    }

    #[test]
    fn test_rgb_from_hex_invalid_length_returns_error() {
        // Arrange
        let too_short_hex = "FF00";

        // Act
        let result = RgbColor::from_hex(too_short_hex);

        // Assert
        assert!(result.is_err(), "from_hex should fail for hex strings that are not 6 characters");
        let error_message = result.unwrap_err();
        assert!(
            error_message.contains("Invalid hex color length"),
            "error should mention invalid length, got: '{}'", error_message
        );
    }

    #[test]
    fn test_rgb_from_hex_invalid_characters_returns_error() {
        // Arrange
        let non_hex_input = "ZZZZZZ";

        // Act
        let result = RgbColor::from_hex(non_hex_input);

        // Assert
        assert!(result.is_err(), "from_hex should fail for non-hex characters");
    }

    #[test]
    fn test_color_def_with_comment() {
        // Arrange
        let color = ColorDef::new("RED".to_string(), RgbColor::new(255, 0, 0));

        // Act
        let color_with_comment = color.with_comment("Bright red".to_string());

        // Assert
        assert_eq!(
            color_with_comment.comment, Some("Bright red".to_string()),
            "with_comment should store the comment"
        );
    }
}
