mod helpers;

use crossterm::event::KeyCode;

const WIDTH: u16 = 120;
const HEIGHT: u16 = 30;

// ============================================================================
// Journey 4a: Navigation & Palette Picker
// ============================================================================

/// 4a-1: Initial cursor position shows L0,0.
#[test]
fn initial_cursor_position() {
    // Arrange
    let app = helpers::create_e2e_app();

    // Act
    let buffer = helpers::render(&app, WIDTH, HEIGHT);

    // Assert
    assert!(
        helpers::buffer_contains(&buffer, "Selected:"),
        "should display Selected: info line"
    );
    assert!(
        helpers::buffer_contains(&buffer, "L0,0"),
        "cursor should start at left half, row 0, col 0"
    );
}

/// 4a-2: Arrow right moves cursor to L0,1.
#[test]
fn arrow_right_moves_cursor() {
    // Arrange
    let mut app = helpers::create_e2e_app();

    // Act
    helpers::send_key(&mut app, KeyCode::Right);
    let buffer = helpers::render(&app, WIDTH, HEIGHT);

    // Assert
    assert!(
        helpers::buffer_contains(&buffer, "L0,1"),
        "after pressing Right, cursor should be at L0,1"
    );
}

/// 4a-3: Tab switches to right half.
#[test]
fn tab_switches_to_right_half() {
    // Arrange
    let mut app = helpers::create_e2e_app();
    helpers::send_key(&mut app, KeyCode::Right); // move to col 1

    // Act
    helpers::send_key(&mut app, KeyCode::Tab);
    let buffer = helpers::render(&app, WIDTH, HEIGHT);

    // Assert
    assert!(
        helpers::buffer_contains(&buffer, "R0,1"),
        "after Tab, cursor should be on right half at R0,1"
    );
}

/// 4a-4: Tab again switches back to left half.
#[test]
fn tab_back_to_left_half() {
    // Arrange
    let mut app = helpers::create_e2e_app();
    helpers::send_key(&mut app, KeyCode::Right); // move to col 1
    helpers::send_key(&mut app, KeyCode::Tab); // switch to right half

    // Act
    helpers::send_key(&mut app, KeyCode::Tab);
    let buffer = helpers::render(&app, WIDTH, HEIGHT);

    // Assert
    assert!(
        helpers::buffer_contains(&buffer, "L0,1"),
        "after Tab back, cursor should be on left half at L0,1"
    );
}

/// 4a-5: Quick paint with digit 0 changes the key abbreviation.
#[test]
fn quick_paint_changes_key_display() {
    // Arrange
    let mut app = helpers::create_e2e_app();
    let before = helpers::render(&app, WIDTH, HEIGHT);
    assert!(
        helpers::buffer_contains(&before, "Selected: ___ @ L0,0"),
        "precondition: cursor key should be ___ at L0,0"
    );

    // Act
    helpers::send_key(&mut app, KeyCode::Char('0'));
    let buffer = helpers::render(&app, WIDTH, HEIGHT);

    // Assert
    assert!(
        helpers::buffer_contains(&buffer, "Selected: RED @ L0,0"),
        "after pressing 0, key should show RED (palette color 0)"
    );
}

/// 4a-6: Backspace clears the key and shows "Cleared" status.
#[test]
fn clear_shows_blank_and_status() {
    // Arrange
    let mut app = helpers::create_e2e_app();
    helpers::send_key(&mut app, KeyCode::Char('0')); // paint RED

    // Act
    helpers::send_key(&mut app, KeyCode::Backspace);
    let buffer = helpers::render(&app, WIDTH, HEIGHT);

    // Assert
    assert!(
        helpers::buffer_contains(&buffer, "Selected: ___ @ L0,0"),
        "after Backspace, key should be cleared to ___"
    );
    assert!(
        helpers::buffer_contains(&buffer, "Cleared"),
        "status bar should show 'Cleared'"
    );
}

/// 4a-7: Undo shows "Undo" status.
#[test]
fn undo_shows_status() {
    // Arrange
    let mut app = helpers::create_e2e_app();
    helpers::send_key(&mut app, KeyCode::Char('0')); // paint RED
    helpers::send_key(&mut app, KeyCode::Backspace); // clear

    // Act
    helpers::send_key(&mut app, KeyCode::Char('u'));
    let buffer = helpers::render(&app, WIDTH, HEIGHT);

    // Assert
    assert!(
        helpers::buffer_contains(&buffer, "Undo"),
        "status bar should show 'Undo'"
    );
}

/// 4a-8: Yank + move + paste shows "Pasted" status.
#[test]
fn copy_paste_color_shows_pasted_status() {
    // Arrange
    let mut app = helpers::create_e2e_app();
    helpers::send_key(&mut app, KeyCode::Char('0')); // paint RED at L0,0

    // Act
    helpers::send_key(&mut app, KeyCode::Char('y')); // yank
    helpers::send_key(&mut app, KeyCode::Right); // move to L0,1
    helpers::send_key(&mut app, KeyCode::Char('p')); // paste
    let buffer = helpers::render(&app, WIDTH, HEIGHT);

    // Assert
    assert!(
        helpers::buffer_contains(&buffer, "Pasted"),
        "status bar should show 'Pasted'"
    );
}

/// 4a-9: Enter opens color picker (focused title).
#[test]
fn enter_opens_color_picker() {
    // Arrange
    let mut app = helpers::create_e2e_app();

    // Act
    helpers::send_key(&mut app, KeyCode::Enter);
    let buffer = helpers::render(&app, WIDTH, HEIGHT);

    // Assert
    assert!(
        helpers::buffer_contains(&buffer, "Enter to apply"),
        "color picker should show focused title with 'Enter to apply'"
    );
}

/// 4a-10: Navigate in picker then cancel with Escape returns to unfocused title.
#[test]
fn palette_navigate_then_cancel() {
    // Arrange
    let mut app = helpers::create_e2e_app();
    helpers::send_key(&mut app, KeyCode::Enter); // open picker

    // Act
    helpers::send_key(&mut app, KeyCode::Right); // navigate in picker
    helpers::send_key(&mut app, KeyCode::Esc); // cancel
    let buffer = helpers::render(&app, WIDTH, HEIGHT);

    // Assert
    assert!(
        helpers::buffer_contains(&buffer, "Enter to pick"),
        "after Escape, color picker should show unfocused title with 'Enter to pick'"
    );
}

/// 4a-11: Open picker, navigate, confirm returns to normal mode.
#[test]
fn palette_confirm_returns_to_normal() {
    // Arrange
    let mut app = helpers::create_e2e_app();

    // Act
    helpers::send_key(&mut app, KeyCode::Enter); // open picker
    helpers::send_key(&mut app, KeyCode::Right); // navigate
    helpers::send_key(&mut app, KeyCode::Enter); // confirm
    let buffer = helpers::render(&app, WIDTH, HEIGHT);

    // Assert
    assert!(
        helpers::buffer_contains(&buffer, "Enter to pick"),
        "after confirming, color picker should show unfocused title (normal mode)"
    );
}

/// 4a-12: PageDown switches to the next layer.
#[test]
fn layer_switch_with_pagedown() {
    // Arrange
    let mut app = helpers::create_e2e_app();
    let before = helpers::render(&app, WIDTH, HEIGHT);
    assert!(
        helpers::buffer_contains(&before, "HRM_WinLinx"),
        "precondition: first layer should be HRM_WinLinx"
    );

    // Act
    helpers::send_key(&mut app, KeyCode::PageDown);
    let buffer = helpers::render(&app, WIDTH, HEIGHT);

    // Assert
    assert!(
        helpers::buffer_contains(&buffer, "HRM_macOS"),
        "after PageDown, keyboard title should show the next layer (HRM_macOS)"
    );
}

// ============================================================================
// Journey 4b: Help Overlay
// ============================================================================

/// 4b-1: `?` opens help overlay with title and content.
#[test]
fn question_mark_opens_help() {
    // Arrange
    let mut app = helpers::create_e2e_app();

    // Act
    helpers::send_key(&mut app, KeyCode::Char('?'));
    let buffer = helpers::render(&app, WIDTH, HEIGHT);

    // Assert
    assert!(
        helpers::buffer_contains(&buffer, "Help"),
        "help overlay should show 'Help' title"
    );
    assert!(
        helpers::buffer_contains(&buffer, "NAVIGATION"),
        "help overlay should contain 'NAVIGATION' section"
    );
}

/// 4b-2: `?` twice opens then closes help.
#[test]
fn question_mark_toggles_help_closed() {
    // Arrange
    let mut app = helpers::create_e2e_app();

    // Act
    helpers::send_key(&mut app, KeyCode::Char('?')); // open
    helpers::send_key(&mut app, KeyCode::Char('?')); // close
    let buffer = helpers::render(&app, WIDTH, HEIGHT);

    // Assert
    assert!(
        !helpers::buffer_contains(&buffer, "NAVIGATION"),
        "help should be closed — 'NAVIGATION' should not appear"
    );
}

/// 4b-3: Escape closes help overlay.
#[test]
fn escape_closes_help() {
    // Arrange
    let mut app = helpers::create_e2e_app();

    // Act
    helpers::send_key(&mut app, KeyCode::Char('?')); // open
    helpers::send_key(&mut app, KeyCode::Esc); // close
    let buffer = helpers::render(&app, WIDTH, HEIGHT);

    // Assert
    assert!(
        !helpers::buffer_contains(&buffer, "NAVIGATION"),
        "help should be closed after Escape — 'NAVIGATION' should not appear"
    );
}
