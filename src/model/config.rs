use std::path::{Path, PathBuf};
use std::fs;

use super::{ColorPalette, Layer};

/// Complete configuration for the RGB editor
#[derive(Debug, Clone)]
pub struct Config {
    /// All layers in order
    pub layers: Vec<Layer>,
    /// Color palette with all defined colors
    pub palette: ColorPalette,
    /// Path to the config file
    pub file_path: PathBuf,
    /// Raw header content (everything before underglow-layer section)
    pub raw_header: String,
    /// Raw footer content (everything after underglow-layer section, including #undef)
    pub raw_footer: String,
}

impl Config {
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            layers: Vec::new(),
            palette: ColorPalette::new(),
            file_path,
            raw_header: String::new(),
            raw_footer: String::new(),
        }
    }

    /// Load config from a file path
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let mut config = crate::parser::parse_config(&content)?;
        config.file_path = path.to_path_buf();
        Ok(config)
    }

    /// Save config to its original file path
    pub fn save(&self) -> Result<(), String> {
        self.save_as(&self.file_path)
    }

    /// Save config to a specific path
    pub fn save_as(&self, path: &Path) -> Result<(), String> {
        let content = crate::parser::write_config(self);
        
        // Create backup of original file if it exists
        if path.exists() {
            let backup_path = path.with_extension("txt.bak");
            fs::copy(path, &backup_path)
                .map_err(|e| format!("Failed to create backup: {}", e))?;
        }
        
        fs::write(path, content)
            .map_err(|e| format!("Failed to write file: {}", e))?;
        
        Ok(())
    }

    /// Get the display name for the file
    pub fn file_name(&self) -> &str {
        self.file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
    }
}
