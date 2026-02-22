use std::fs;
use std::path::Path;

use go60_rgb_editor_domain::Config;

pub fn load_config(path: &Path) -> Result<Config, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
    go60_rgb_editor_domain::parser::parse_config(&content)
}

pub fn save_config(config: &Config, path: &Path) -> Result<(), String> {
    let content = go60_rgb_editor_domain::parser::write_config(config);
    fs::write(path, content).map_err(|e| format!("Failed to write file: {}", e))?;
    Ok(())
}

pub fn save_config_with_backup(config: &Config, path: &Path) -> Result<(), String> {
    if path.exists() {
        let backup_path = path.with_extension("txt.bak");
        fs::copy(path, &backup_path)
            .map_err(|e| format!("Failed to create backup: {}", e))?;
    }
    save_config(config, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_minimal_config_file(dir: &Path, filename: &str) -> PathBuf {
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
        let config = load_config(&config_path).expect("should parse a valid config file");

        // Assert
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
        let result = load_config(&nonexistent_path);

        // Assert
        assert!(
            result.is_err(),
            "loading a nonexistent file should return an error"
        );
        let error_message = result.unwrap_err();
        assert!(
            error_message.contains("Failed to read file"),
            "error should mention file read failure, got: '{}'",
            error_message
        );
    }

    #[test]
    fn test_save_writes_file_to_disk() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_minimal_config_file(temp_dir.path(), "test.txt");
        let config = load_config(&config_path).unwrap();

        // Act
        save_config(&config, &config_path).expect("save should succeed");

        // Assert
        let saved_content = fs::read_to_string(&config_path).expect("should read saved file");
        assert!(
            saved_content.contains("PER-KEY-RGB"),
            "saved file should contain the config section markers"
        );
    }

    #[test]
    fn test_save_with_backup_creates_backup_of_existing_file() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let target_path = temp_dir.path().join("target.txt");
        let original_content = "original content";
        fs::write(&target_path, original_content).unwrap();
        let config_path = create_minimal_config_file(temp_dir.path(), "source.txt");
        let config = load_config(&config_path).unwrap();
        let expected_backup_path = target_path.with_extension("txt.bak");

        // Act
        save_config_with_backup(&config, &target_path).expect("save should succeed");

        // Assert
        assert!(
            expected_backup_path.exists(),
            "save should create a backup file at {:?}",
            expected_backup_path
        );
        let backup_content = fs::read_to_string(&expected_backup_path).unwrap();
        assert_eq!(
            backup_content, original_content,
            "backup should contain the original file content"
        );
    }

    #[test]
    fn test_save_to_new_file_does_not_create_backup() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_minimal_config_file(temp_dir.path(), "source.txt");
        let config = load_config(&config_path).unwrap();
        let new_path = temp_dir.path().join("new_output.txt");
        let backup_path = new_path.with_extension("txt.bak");

        // Act
        save_config_with_backup(&config, &new_path).expect("save to new file should succeed");

        // Assert
        assert!(new_path.exists(), "new file should be created");
        assert!(
            !backup_path.exists(),
            "no backup should be created when saving to a new file"
        );
    }

    #[test]
    fn test_save_to_invalid_directory_returns_error() {
        // Arrange
        let config = Config::new();
        let invalid_path = PathBuf::from("/nonexistent/dir/output.txt");

        // Act
        let result = save_config(&config, &invalid_path);

        // Assert
        assert!(
            result.is_err(),
            "save to an invalid directory should return an error"
        );
        let error_message = result.unwrap_err();
        assert!(
            error_message.contains("Failed to write file"),
            "error should mention write failure, got: '{}'",
            error_message
        );
    }
}
