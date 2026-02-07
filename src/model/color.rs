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
            ratatui::style::Color::Black
        } else {
            ratatui::style::Color::White
        }
    }

    /// Format as hex string for output
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
        let color = RgbColor::from_hex("0xFF0000").unwrap();
        assert_eq!(color, RgbColor::new(255, 0, 0));

        let color = RgbColor::from_hex("00FF00").unwrap();
        assert_eq!(color, RgbColor::new(0, 255, 0));

        let color = RgbColor::from_hex("#0000FF").unwrap();
        assert_eq!(color, RgbColor::new(0, 0, 255));
    }

    #[test]
    fn test_rgb_to_hex() {
        let color = RgbColor::new(255, 128, 0);
        assert_eq!(color.to_hex(), "0xFF8000");
    }

    #[test]
    fn test_luminance() {
        let white = RgbColor::new(255, 255, 255);
        let black = RgbColor::new(0, 0, 0);
        assert!(white.luminance() > 200.0);
        assert!(black.luminance() < 1.0);
    }

    #[test]
    fn test_palette() {
        let mut palette = ColorPalette::new();
        palette.add(ColorDef::new(
            "RED".to_string(),
            "RED_RGB".to_string(),
            RgbColor::new(255, 0, 0),
        ));
        palette.add(ColorDef::new(
            "GRN".to_string(),
            "GRN_RGB".to_string(),
            RgbColor::new(0, 255, 0),
        ));

        assert!(palette.get("RED").is_some());
        assert!(palette.get("GRN").is_some());
        assert!(palette.get("BLU").is_none());
    }
}
