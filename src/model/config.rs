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

    pub fn load(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let mut config = crate::parser::parse_config(&content)?;
        config.file_path = path.to_path_buf();
        Ok(config)
    }

    pub fn save(&self) -> Result<(), String> {
        self.save_as(&self.file_path)
    }

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

    pub fn file_name(&self) -> &str {
        self.file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_minimal_config_file(dir: &std::path::Path, filename: &str) -> PathBuf {
        let path = dir.join(filename);
        let minimal_content = "\
#define ___RGB 0x000000
#define ___ &ug ___RGB
// ==== PER-KEY-RGB <section begins> ====
  / {
    underglow-layer {
      compatible = \"zmk,underglow-layer\";

    };
  };
  // ==== PER-KEY-RGB <section ends> =====
";
        fs::write(&path, minimal_content).unwrap();
        path
    }

    #[test]
    fn test_load_parses_valid_config_file() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_minimal_config_file(temp_dir.path(), "test.txt");

        // Act
        let config = Config::load(&config_path).expect("should parse a valid config file");

        // Assert
        assert_eq!(
            config.file_path, config_path,
            "loaded config should store the file path it was loaded from"
        );
        assert!(
            config.palette.get("___").is_some(),
            "loaded config should have parsed the ___ color from the file"
        );
    }

    #[test]
    fn test_load_nonexistent_file_returns_error() {
        // Arrange
        let nonexistent_path = PathBuf::from("/nonexistent/path/config.txt");

        // Act
        let result = Config::load(&nonexistent_path);

        // Assert
        assert!(result.is_err(), "loading a nonexistent file should return an error");
        let error_message = result.unwrap_err();
        assert!(
            error_message.contains("Failed to read file"),
            "error should mention file read failure, got: '{}'", error_message
        );
    }

    #[test]
    fn test_save_writes_file_to_disk() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_minimal_config_file(temp_dir.path(), "test.txt");
        let config = Config::load(&config_path).unwrap();

        // Act
        config.save().expect("save should succeed");

        // Assert
        let saved_content = fs::read_to_string(&config_path).expect("should read saved file");
        assert!(
            saved_content.contains("PER-KEY-RGB"),
            "saved file should contain the config section markers"
        );
    }

    #[test]
    fn test_save_as_creates_backup_of_existing_file() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let target_path = temp_dir.path().join("target.txt");
        let original_content = "original content";
        fs::write(&target_path, original_content).unwrap();
        let config_path = create_minimal_config_file(temp_dir.path(), "source.txt");
        let config = Config::load(&config_path).unwrap();
        let expected_backup_path = target_path.with_extension("txt.bak");

        // Act
        config.save_as(&target_path).expect("save_as should succeed");

        // Assert
        assert!(
            expected_backup_path.exists(),
            "save_as should create a backup file at {:?}", expected_backup_path
        );
        let backup_content = fs::read_to_string(&expected_backup_path).unwrap();
        assert_eq!(
            backup_content, original_content,
            "backup should contain the original file content"
        );
    }

    #[test]
    fn test_save_as_to_new_file_does_not_create_backup() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_minimal_config_file(temp_dir.path(), "source.txt");
        let config = Config::load(&config_path).unwrap();
        let new_path = temp_dir.path().join("new_output.txt");
        let backup_path = new_path.with_extension("txt.bak");

        // Act
        config.save_as(&new_path).expect("save_as to new file should succeed");

        // Assert
        assert!(new_path.exists(), "new file should be created");
        assert!(
            !backup_path.exists(),
            "no backup should be created when saving to a new file"
        );
    }

    #[test]
    fn test_save_as_invalid_directory_returns_error() {
        // Arrange
        let config = Config::new(PathBuf::from("dummy.txt"));
        let invalid_path = PathBuf::from("/nonexistent/dir/output.txt");

        // Act
        let result = config.save_as(&invalid_path);

        // Assert
        assert!(result.is_err(), "save_as to an invalid directory should return an error");
        let error_message = result.unwrap_err();
        assert!(
            error_message.contains("Failed to write file"),
            "error should mention write failure, got: '{}'", error_message
        );
    }

    #[test]
    fn test_file_name_returns_filename_component() {
        // Arrange
        let config = Config::new(PathBuf::from("/some/path/my_config.txt"));

        // Act
        let display_name = config.file_name();

        // Assert
        assert_eq!(
            display_name, "my_config.txt",
            "file_name should return only the filename, not the full path"
        );
    }

    #[test]
    fn test_file_name_with_empty_path_returns_unknown() {
        // Arrange
        let config = Config::new(PathBuf::from(""));

        // Act
        let display_name = config.file_name();

        // Assert
        assert_eq!(
            display_name, "unknown",
            "file_name should return 'unknown' for an empty path"
        );
    }
}
