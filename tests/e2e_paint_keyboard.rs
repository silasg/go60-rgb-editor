mod helpers;

use crossterm::event::{KeyCode, KeyModifiers};

const WIDTH: u16 = 120;
const HEIGHT: u16 = 30;

/// Journey 1, Step 1: Initial state renders layer names, palette, and Normal mode.
#[test]
fn initial_state_shows_layers_palette_and_normal_mode() {
    // Arrange
    let app = helpers::create_e2e_app();

    // Act
    let buffer = helpers::render(&app, WIDTH, HEIGHT);

    // Assert — layer names visible in layer list
    assert!(
        helpers::buffer_contains(&buffer, "HRM_WinLinx"),
        "first layer name should be visible"
    );
    assert!(
        helpers::buffer_contains(&buffer, "Keypad"),
        "Keypad layer name should be visible"
    );

    // Assert — palette colors visible
    assert!(
        helpers::buffer_contains(&buffer, "RED"),
        "palette should show RED"
    );
    assert!(
        helpers::buffer_contains(&buffer, "CYN"),
        "palette should show CYN"
    );

    // Assert — Normal mode shows no mode indicator in status bar
    // (status bar renders empty string for Normal mode)
    assert!(
        !helpers::buffer_contains(&buffer, "COLOR"),
        "should not show COLOR mode indicator"
    );
    assert!(
        !helpers::buffer_contains(&buffer, "HELP"),
        "should not show HELP mode indicator"
    );
}

/// Journey 1, Step 2: Pressing quick color `0` (RED) paints the key at cursor.
#[test]
fn apply_quick_color_paints_key_at_cursor() {
    // Arrange
    let mut app = helpers::create_e2e_app();
    // Cursor starts at row 0, col 0, Left half — which is "___" in HRM_WinLinx
    let before = helpers::render(&app, WIDTH, HEIGHT);
    assert!(
        helpers::buffer_contains(&before, "Selected: ___ @ L0,0"),
        "cursor should initially be on ___ at L0,0"
    );

    // Act — quick color 0 = RED (first palette color)
    helpers::send_key(&mut app, KeyCode::Char('0'));

    // Assert
    let after = helpers::render(&app, WIDTH, HEIGHT);
    assert!(
        helpers::buffer_contains(&after, "Selected: RED @ L0,0"),
        "after pressing 0, selected key should show RED"
    );
}

/// Journey 1, Step 3: Move right and paint with quick color `1` (COR).
#[test]
fn move_and_paint_another_key() {
    // Arrange
    let mut app = helpers::create_e2e_app();
    helpers::send_key(&mut app, KeyCode::Char('0')); // paint RED at (0,0)

    // Act — move right then paint COR
    helpers::send_key(&mut app, KeyCode::Right);
    helpers::send_key(&mut app, KeyCode::Char('1'));

    // Assert
    let buffer = helpers::render(&app, WIDTH, HEIGHT);
    assert!(
        helpers::buffer_contains(&buffer, "Selected: COR @ L0,1"),
        "after moving right and pressing 1, selected key should show COR at col 1"
    );
}

/// Journey 1, Step 4: Backspace clears the key color to `___`.
#[test]
fn clear_key_with_backspace() {
    // Arrange
    let mut app = helpers::create_e2e_app();
    helpers::send_key(&mut app, KeyCode::Char('0')); // paint RED at (0,0)
    let painted = helpers::render(&app, WIDTH, HEIGHT);
    assert!(
        helpers::buffer_contains(&painted, "Selected: RED @ L0,0"),
        "precondition: key should be RED before clearing"
    );

    // Act
    helpers::send_key(&mut app, KeyCode::Backspace);

    // Assert
    let buffer = helpers::render(&app, WIDTH, HEIGHT);
    assert!(
        helpers::buffer_contains(&buffer, "Selected: ___ @ L0,0"),
        "after Backspace, key should be cleared to ___"
    );
    assert!(
        helpers::buffer_contains(&buffer, "Cleared"),
        "status bar should show 'Cleared' message"
    );
}

/// Journey 1, Step 5: Undo restores the cleared color and shows "Undo" status.
#[test]
fn undo_restores_color_and_shows_status() {
    // Arrange
    let mut app = helpers::create_e2e_app();
    helpers::send_key(&mut app, KeyCode::Char('0')); // paint RED
    helpers::send_key(&mut app, KeyCode::Backspace); // clear to ___
    let cleared = helpers::render(&app, WIDTH, HEIGHT);
    assert!(
        helpers::buffer_contains(&cleared, "Selected: ___ @ L0,0"),
        "precondition: key should be ___ before undo"
    );

    // Act
    helpers::send_key(&mut app, KeyCode::Char('u'));

    // Assert
    let buffer = helpers::render(&app, WIDTH, HEIGHT);
    assert!(
        helpers::buffer_contains(&buffer, "Selected: RED @ L0,0"),
        "after undo, key should be restored to RED"
    );
    assert!(
        helpers::buffer_contains(&buffer, "Undo"),
        "status bar should show 'Undo' message"
    );
}

/// Journey 1, Step 6: Redo re-applies the clear and shows "Redo" status.
#[test]
fn redo_reapplies_clear_and_shows_status() {
    // Arrange
    let mut app = helpers::create_e2e_app();
    helpers::send_key(&mut app, KeyCode::Char('0')); // paint RED
    helpers::send_key(&mut app, KeyCode::Backspace); // clear to ___
    helpers::send_key(&mut app, KeyCode::Char('u')); // undo → RED
    let undone = helpers::render(&app, WIDTH, HEIGHT);
    assert!(
        helpers::buffer_contains(&undone, "Selected: RED @ L0,0"),
        "precondition: key should be RED after undo"
    );

    // Act
    helpers::send_key_modified(&mut app, KeyCode::Char('r'), KeyModifiers::CONTROL);

    // Assert
    let buffer = helpers::render(&app, WIDTH, HEIGHT);
    assert!(
        helpers::buffer_contains(&buffer, "Selected: ___ @ L0,0"),
        "after redo, key should be ___ again"
    );
    assert!(
        helpers::buffer_contains(&buffer, "Redo"),
        "status bar should show 'Redo' message"
    );
}

/// Journey 1, Step 7: Copy config when modified shows ConfirmCopy modal.
#[test]
fn copy_config_when_modified_shows_confirm_modal() {
    // Arrange
    let mut app = helpers::create_e2e_app();
    helpers::send_key(&mut app, KeyCode::Char('0')); // paint RED → makes editor modified

    // Act
    helpers::send_key(&mut app, KeyCode::Char('c'));

    // Assert
    let buffer = helpers::render(&app, WIDTH, HEIGHT);
    assert!(
        helpers::buffer_contains(&buffer, "COPY?"),
        "status bar should show COPY? mode indicator"
    );
    assert!(
        helpers::buffer_contains(&buffer, "unsaved changes"),
        "confirm copy modal should mention unsaved changes"
    );
}
