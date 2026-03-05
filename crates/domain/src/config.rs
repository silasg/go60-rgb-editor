use super::{ColorPalette, Layer};

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub layers: Vec<Layer>,
    pub palette: ColorPalette,
    /// Everything before the underglow-layer section
    pub raw_header: String,
    /// Everything after the underglow-layer section (including #undef)
    pub raw_footer: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            layers: Vec::new(),
            palette: ColorPalette::new(),
            raw_header: String::new(),
            raw_footer: String::new(),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }
}
