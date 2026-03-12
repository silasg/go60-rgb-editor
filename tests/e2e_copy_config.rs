mod helpers;

use crossterm::event::KeyCode;

#[test]
fn initial_state_shows_layers_and_normal_mode() {
    // Arrange
    let app = helpers::create_e2e_app();

    // Act
    let buffer = helpers::render(&app, 120, 30);

    // Assert
    assert!(buffer_contains(&buffer, "Layer"), "should show layer names");
    assert!(!buffer_contains(&buffer, "COPY?"), "should not show COPY? in status bar");
}

#[test]
fn copy_when_modified_shows_confirm_modal() {
    // Arrange
    let mut app = helpers::create_e2e_app();

    // Act
    helpers::send_key(&mut app, KeyCode::Char('0'));
    helpers::send_key(&mut app, KeyCode::Char('c'));
    let buffer = helpers::render(&app, 120, 30);

    // Assert
    assert!(
        buffer_contains(&buffer, "unsaved changes"),
        "should show ConfirmCopy modal with unsaved changes text"
    );
    assert!(
        buffer_contains(&buffer, "COPY?"),
        "should show COPY? mode indicator in status bar"
    );
}

#[test]
fn confirm_copy_with_y_returns_to_normal() {
    // Arrange
    let mut app = helpers::create_e2e_app();
    helpers::send_key(&mut app, KeyCode::Char('0'));
    helpers::send_key(&mut app, KeyCode::Char('c'));

    // Act
    helpers::send_key(&mut app, KeyCode::Char('y'));
    let buffer = helpers::render(&app, 120, 30);

    // Assert
    assert!(
        !buffer_contains(&buffer, "unsaved changes"),
        "should dismiss ConfirmCopy modal after confirming"
    );
    assert!(
        !buffer_contains(&buffer, "COPY?"),
        "should return to Normal mode (no COPY? in status bar)"
    );
}

#[test]
fn cancel_copy_with_n_returns_to_normal() {
    // Arrange
    let mut app = helpers::create_e2e_app();
    helpers::send_key(&mut app, KeyCode::Char('0'));
    helpers::send_key(&mut app, KeyCode::Char('c'));

    // Act
    helpers::send_key(&mut app, KeyCode::Char('n'));
    let buffer = helpers::render(&app, 120, 30);

    // Assert
    assert!(
        !buffer_contains(&buffer, "unsaved changes"),
        "should dismiss ConfirmCopy modal after cancelling with n"
    );
}

#[test]
fn cancel_copy_with_escape_returns_to_normal() {
    // Arrange
    let mut app = helpers::create_e2e_app();
    helpers::send_key(&mut app, KeyCode::Char('0'));
    helpers::send_key(&mut app, KeyCode::Char('c'));

    // Act
    helpers::send_key(&mut app, KeyCode::Esc);
    let buffer = helpers::render(&app, 120, 30);

    // Assert
    assert!(
        !buffer_contains(&buffer, "unsaved changes"),
        "should dismiss ConfirmCopy modal after cancelling with Escape"
    );
}

fn buffer_contains(buffer: &ratatui::buffer::Buffer, text: &str) -> bool {
    helpers::buffer_contains(buffer, text)
}
