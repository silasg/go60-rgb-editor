mod helpers;

use crossterm::event::{KeyCode, KeyModifiers};

#[test]
fn initial_state_shows_first_layer() {
    // Arrange
    let app = helpers::create_e2e_app();

    // Act
    let buf = helpers::render(&app, 120, 30);

    // Assert
    assert!(
        buffer_contains_layer_marker(&buf, "HRM_WinLinx"),
        "Buffer should contain the first layer name 'HRM_WinLinx'"
    );
    assert!(
        helpers::buffer_contains(&buf, "Layers"),
        "Buffer should contain the 'Layers' panel header"
    );
}

#[test]
fn add_layer_shows_new_layer_and_status() {
    // Arrange
    let mut app = helpers::create_e2e_app();

    // Act
    helpers::send_key(&mut app, KeyCode::Char('a'));
    helpers::type_str(&mut app, "Test_Layer");
    helpers::send_key(&mut app, KeyCode::Enter);
    let buf = helpers::render(&app, 120, 30);

    // Assert
    assert!(
        helpers::buffer_contains(&buf, "Test_Layer"),
        "Buffer should contain the newly added layer 'Test_Layer'"
    );
    assert!(
        helpers::buffer_contains(&buf, "Added"),
        "Status bar should contain 'Added'"
    );
}

#[test]
fn duplicate_layer_shows_copy_and_status() {
    // Arrange
    let mut app = helpers::create_e2e_app();

    // Act
    helpers::send_key(&mut app, KeyCode::Char('d'));
    let buf = helpers::render(&app, 120, 30);

    // Assert
    assert!(
        helpers::buffer_contains(&buf, "Duplicated"),
        "Status bar should contain 'Duplicated'"
    );
    assert!(
        helpers::buffer_contains(&buf, "HRM_WinLinx_copy"),
        "Buffer should contain the duplicated layer name 'HRM_WinLinx_copy'"
    );
}

#[test]
fn rename_layer_shows_new_name_and_status() {
    // Arrange
    let mut app = helpers::create_e2e_app();

    // Act — enter rename mode, clear prefilled name, type new name, confirm
    helpers::send_key(&mut app, KeyCode::Char('n'));
    helpers::send_key_modified(&mut app, KeyCode::Char('u'), KeyModifiers::CONTROL);
    helpers::type_str(&mut app, "Renamed");
    helpers::send_key(&mut app, KeyCode::Enter);
    let buf = helpers::render(&app, 120, 30);

    // Assert
    assert!(
        helpers::buffer_contains(&buf, "Renamed"),
        "Buffer should contain the renamed layer 'Renamed'"
    );
    assert!(
        helpers::buffer_contains(&buf, "Renamed"),
        "Status bar should contain 'Renamed'"
    );
}

#[test]
fn switch_layer_moves_selection_indicator() {
    // Arrange
    let mut app = helpers::create_e2e_app();
    let buf_before = helpers::render(&app, 120, 30);

    // Act
    helpers::send_key(&mut app, KeyCode::PageDown);
    let buf_after = helpers::render(&app, 120, 30);

    // Assert — the ▶ indicator should no longer be next to HRM_WinLinx
    assert!(
        buffer_contains_layer_marker(&buf_before, "HRM_WinLinx"),
        "Before: ▶ should be on HRM_WinLinx"
    );
    assert!(
        buffer_contains_layer_marker(&buf_after, "HRM_macOS"),
        "After PageDown: ▶ should move to HRM_macOS"
    );
}

#[test]
fn increase_fade_shows_status() {
    // Arrange
    let mut app = helpers::create_e2e_app();

    // Act
    helpers::send_key(&mut app, KeyCode::Char('f'));
    let buf = helpers::render(&app, 120, 30);

    // Assert
    assert!(
        helpers::buffer_contains(&buf, "Fade:"),
        "Status bar should contain 'Fade:' after increasing fade"
    );
}

#[test]
fn decrease_fade_shows_status() {
    // Arrange
    let mut app = helpers::create_e2e_app();
    // Increase first to ensure there's room to decrease
    helpers::send_key(&mut app, KeyCode::Char('f'));
    helpers::send_key(&mut app, KeyCode::Char('f'));

    // Act
    helpers::send_key(&mut app, KeyCode::Char('F'));
    let buf = helpers::render(&app, 120, 30);

    // Assert
    assert!(
        helpers::buffer_contains(&buf, "Fade:"),
        "Status bar should contain 'Fade:' after decreasing fade"
    );
}

#[test]
fn delete_layer_shows_deleted_status() {
    // Arrange — add a layer first so we can delete it
    let mut app = helpers::create_e2e_app();
    helpers::send_key(&mut app, KeyCode::Char('a'));
    helpers::type_str(&mut app, "ToDelete");
    helpers::send_key(&mut app, KeyCode::Enter);

    // Act — delete the current layer (confirm with 'y')
    helpers::send_key(&mut app, KeyCode::Char('x'));
    helpers::send_key(&mut app, KeyCode::Char('y'));
    let buf = helpers::render(&app, 120, 30);

    // Assert
    assert!(
        helpers::buffer_contains(&buf, "Deleted"),
        "Status bar should contain 'Deleted'"
    );
}

#[test]
fn cancel_delete_returns_to_normal() {
    // Arrange
    let mut app = helpers::create_e2e_app();

    // Act — start delete then cancel with Esc
    helpers::send_key(&mut app, KeyCode::Char('x'));
    helpers::send_key(&mut app, KeyCode::Esc);
    let buf = helpers::render(&app, 120, 30);

    // Assert — no "DELETE?" mode indicator in the status bar
    assert!(
        !helpers::buffer_contains(&buf, "DELETE?"),
        "Buffer should NOT contain 'DELETE?' after cancelling delete"
    );
}

/// Helper: checks that the buffer contains "▶ {name}" (the active layer marker).
fn buffer_contains_layer_marker(buf: &ratatui::buffer::Buffer, name: &str) -> bool {
    let marker = format!("▶ {}", name);
    helpers::buffer_contains(buf, &marker)
}
