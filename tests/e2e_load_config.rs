mod helpers;

use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyModifiers};
use tempfile::TempDir;

use go60_rgb_editor::app::App;

fn create_app_with_tempfile() -> (App, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path().join("test_config.txt");
    let fixture_content = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sample_config.txt"),
    )
    .unwrap();
    std::fs::write(&temp_path, &fixture_content).unwrap();
    let config = go60_rgb_editor_domain::parser::parse_config(&fixture_content).unwrap();
    let app = App::new(config, temp_path);
    (app, temp_dir)
}

#[test]
fn load_valid_config_shows_layers_and_filename() {
    // Arrange
    let app = helpers::create_e2e_app();

    // Act
    let buffer = helpers::render(&app, 120, 30);

    // Assert
    assert!(
        helpers::buffer_contains(&buffer, "HRM_WinLinx"),
        "buffer should contain layer name from fixture"
    );
    assert!(
        helpers::buffer_contains(&buffer, "sample_config.txt"),
        "status bar should show the fixture filename"
    );
}

#[test]
fn modify_and_save_shows_saved_status() {
    // Arrange
    let (mut app, _temp_dir) = create_app_with_tempfile();

    // Act
    helpers::send_key(&mut app, KeyCode::Char('0'));
    helpers::send_key(&mut app, KeyCode::Char('s'));
    let buffer = helpers::render(&app, 120, 30);

    // Assert
    assert!(
        helpers::buffer_contains(&buffer, "Saved!"),
        "status bar should show Saved! after saving"
    );
}

#[test]
fn save_as_flow_writes_to_new_file() {
    // Arrange
    let (mut app, temp_dir) = create_app_with_tempfile();
    let new_path = temp_dir.path().join("new_config.txt");
    let new_path_str = new_path.to_string_lossy().to_string();

    // Act
    helpers::send_key(&mut app, KeyCode::Char('S'));
    helpers::send_key_modified(&mut app, KeyCode::Char('u'), KeyModifiers::CONTROL);
    helpers::type_str(&mut app, &new_path_str);
    helpers::send_key(&mut app, KeyCode::Enter);
    let buffer = helpers::render(&app, 120, 30);

    // Assert
    assert!(
        helpers::buffer_contains(&buffer, "Saved!"),
        "status bar should show Saved! after save-as"
    );
}

#[test]
fn cancel_save_as_returns_to_normal() {
    // Arrange
    let mut app = helpers::create_e2e_app();
    helpers::send_key(&mut app, KeyCode::Char('S'));

    // Act
    helpers::send_key(&mut app, KeyCode::Esc);
    let buffer = helpers::render(&app, 120, 30);

    // Assert
    assert!(
        !helpers::buffer_contains(&buffer, "SAVE AS"),
        "buffer should not contain SAVE AS mode indicator after cancelling"
    );
}
