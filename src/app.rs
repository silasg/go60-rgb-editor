use std::path::PathBuf;
use std::time::Instant;

pub use crate::domain::cursor::Direction;
use crate::domain::editor::EditorState;
use crate::domain::Config;
use crate::ui::ColorPickerState;

const FADE_STEP_MS: u16 = 5;
const STATUS_TIMEOUT_SECS: u64 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    ColorPick,
    Help,
    ConfirmQuit,
    ConfirmCopy,
    SaveAs,
    SaveAsConfirm,
}



pub struct App {
    pub editor: EditorState,
    pub file_path: PathBuf,
    pub mode: Mode,
    pub color_picker: ColorPickerState,
    pub status_message: Option<(String, Instant)>,
    pub should_quit: bool,
    pub filename_input: String,
}

impl App {
    pub fn new(config: Config, file_path: PathBuf) -> Self {
        Self {
            editor: EditorState::new(config),
            file_path,
            mode: Mode::Normal,
            color_picker: ColorPickerState::new(),
            status_message: None,
            should_quit: false,
            filename_input: String::new(),
        }
    }

    pub fn file_name(&self) -> &str {
        self.file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
    }

    // --- Domain delegation with status messages ---

    pub fn move_cursor(&mut self, direction: Direction) {
        self.editor.move_cursor(direction);
    }

    pub fn switch_half(&mut self) {
        self.editor.switch_half();
    }

    pub fn next_layer(&mut self) {
        self.editor.next_layer();
    }

    pub fn prev_layer(&mut self) {
        self.editor.prev_layer();
    }

    pub fn increase_fade(&mut self) {
        if let Some(new_delay) = self.editor.adjust_fade(FADE_STEP_MS as i32) {
            self.show_status(&format!("Fade: {}ms", new_delay));
        }
    }

    pub fn decrease_fade(&mut self) {
        if let Some(new_delay) = self.editor.adjust_fade(-(FADE_STEP_MS as i32)) {
            self.show_status(&format!("Fade: {}ms", new_delay));
        }
    }

    pub fn set_current_key_color(&mut self, color: &str) {
        self.editor.set_key_color(color);
    }

    pub fn undo(&mut self) {
        if self.editor.undo() {
            self.show_status("Undo");
        } else {
            self.show_status("Nothing to undo");
        }
    }

    pub fn redo(&mut self) {
        if self.editor.redo() {
            self.show_status("Redo");
        } else {
            self.show_status("Nothing to redo");
        }
    }

    pub fn save(&mut self) {
        match crate::io::save_config_with_backup(&self.editor.config, &self.file_path) {
            Ok(()) => {
                self.editor.mark_saved();
                self.show_status("Saved!");
            }
            Err(e) => {
                self.show_status(&format!("Save failed: {}", e));
            }
        }
    }

    pub fn save_as(&mut self) {
        self.filename_input = self.file_path.to_string_lossy().to_string();
        self.mode = Mode::SaveAs;
    }

    pub fn try_save_as(&mut self) {
        if self.filename_input.is_empty() {
            self.show_status("Filename cannot be empty");
            return;
        }

        let path = PathBuf::from(&self.filename_input);

        if path.exists() && path != self.file_path {
            self.mode = Mode::SaveAsConfirm;
        } else {
            self.execute_save_as();
        }
    }

    pub fn execute_save_as(&mut self) {
        let path = PathBuf::from(&self.filename_input);
        match crate::io::save_config_with_backup(&self.editor.config, &path) {
            Ok(()) => {
                self.file_path = path;
                self.editor.mark_saved();
                self.mode = Mode::Normal;
                self.filename_input.clear();
                self.show_status("Saved!");
            }
            Err(e) => {
                self.show_status(&format!("Save failed: {}", e));
                self.mode = Mode::SaveAs;
            }
        }
    }

    pub fn cancel_save_as(&mut self) {
        self.filename_input.clear();
        self.mode = Mode::Normal;
    }

    pub fn copy_to_clipboard(&mut self) {
        let content = match std::fs::read_to_string(&self.file_path) {
            Ok(c) => c,
            Err(e) => {
                self.show_status(&format!("Read failed: {}", e));
                return;
            }
        };

        match crate::io::copy_to_clipboard(&content) {
            Ok(()) => self.show_status("Copied to clipboard!"),
            Err(e) => self.show_status(&e),
        }
    }

    pub fn show_status(&mut self, message: &str) {
        self.status_message = Some((message.to_string(), Instant::now()));
    }

    pub fn clear_expired_status(&mut self) {
        if let Some((_, time)) = &self.status_message {
            if time.elapsed().as_secs() >= STATUS_TIMEOUT_SECS {
                self.status_message = None;
            }
        }
    }

    pub fn get_current_color(&self) -> Option<&str> {
        self.editor.current_color()
    }

    pub fn copy_color(&mut self) {
        if let Some(c) = self.editor.yank_color() {
            self.show_status(&format!("Copied: {}", c));
        }
    }

    pub fn paste_color(&mut self) {
        if let Some(color) = self.editor.paste_color() {
            self.show_status(&format!("Pasted: {}", color));
        } else {
            self.show_status("Nothing to paste");
        }
    }

    pub fn clear_color(&mut self) {
        self.editor.clear_key_color();
        self.show_status("Cleared");
    }

    pub fn move_color_selection(&mut self, direction: Direction) {
        self.color_picker
            .move_selection(direction, &self.editor.config.palette);
    }

    pub fn apply_selected_color(&mut self) {
        if let Some(color) = self.editor.config.palette.colors.get(self.color_picker.selected) {
            let abbrev = color.abbrev.clone();
            self.set_current_key_color(&abbrev);
            self.mode = Mode::Normal;
        }
    }

    pub fn request_quit(&mut self) {
        if self.editor.modified {
            self.mode = Mode::ConfirmQuit;
        } else {
            self.should_quit = true;
        }
    }

    pub fn request_copy(&mut self) {
        if self.editor.modified {
            self.mode = Mode::ConfirmCopy;
        } else {
            self.copy_to_clipboard();
        }
    }

    pub fn enter_color_pick(&mut self) {
        if let Some(color) = self.editor.current_color() {
            if let Some(&idx) = self.editor.config.palette.abbrev_to_index.get(color) {
                self.color_picker.selected = idx;
            }
        }
        self.mode = Mode::ColorPick;
    }

    pub fn apply_quick_color(&mut self, index: usize) {
        if let Some(color) = self.editor.config.palette.colors.get(index) {
            let abbrev = color.abbrev.clone();
            self.set_current_key_color(&abbrev);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Half, RgbPos};

    fn create_test_app() -> App {
        use crate::domain::{ColorPalette, Config, Layer};

        let mut config = Config::new();
        config.palette = ColorPalette::new();
        config
            .layers
            .push(Layer::new("Test".to_string(), "LAYER_Test".to_string()));

        App::new(config, PathBuf::from("test.txt"))
    }

    // --- Clipboard ---

    #[test]
    fn test_copy_to_clipboard_with_valid_file() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "test content").unwrap();
        let mut config = crate::domain::Config::new();
        config.palette = crate::domain::ColorPalette::new();
        let mut app = App::new(config, temp_file.path().to_path_buf());

        app.copy_to_clipboard();

        assert!(app.status_message.is_some());
    }

    #[test]
    fn test_copy_to_clipboard_with_nonexistent_file() {
        let nonexistent_path = PathBuf::from("/nonexistent/path/file.txt");
        let mut config = crate::domain::Config::new();
        config.palette = crate::domain::ColorPalette::new();
        let mut app = App::new(config, nonexistent_path);

        app.copy_to_clipboard();

        let (msg, _) = app.status_message.as_ref().unwrap();
        assert!(msg.contains("Read failed"));
    }

    // --- Save As ---

    #[test]
    fn test_save_as_opens_dialog_with_current_filename() {
        let mut app = create_test_app();

        app.save_as();

        assert_eq!(app.mode, Mode::SaveAs);
        assert_eq!(app.filename_input, "test.txt");
    }

    #[test]
    fn test_cancel_save_as() {
        let mut app = create_test_app();
        app.save_as();
        app.filename_input = "modified.txt".to_string();

        app.cancel_save_as();

        assert_eq!(app.mode, Mode::Normal);
        assert!(app.filename_input.is_empty());
    }

    #[test]
    fn test_try_save_as_empty_filename_shows_error() {
        let mut app = create_test_app();
        app.mode = Mode::SaveAs;
        app.filename_input = String::new();

        app.try_save_as();

        assert_eq!(app.mode, Mode::SaveAs);
        let (msg, _) = app.status_message.as_ref().unwrap();
        assert!(msg.contains("empty"));
    }

    #[test]
    fn test_try_save_as_existing_file_prompts_confirmation() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let source_file = NamedTempFile::new().unwrap();
        let mut config = crate::domain::Config::new();
        config.palette = crate::domain::ColorPalette::new();
        let mut app = App::new(config, source_file.path().to_path_buf());
        let mut existing_target_file = NamedTempFile::new().unwrap();
        writeln!(existing_target_file, "existing content").unwrap();
        app.mode = Mode::SaveAs;
        app.filename_input = existing_target_file.path().to_string_lossy().to_string();

        app.try_save_as();

        assert_eq!(app.mode, Mode::SaveAsConfirm);
    }

    #[test]
    fn test_try_save_as_same_file_no_confirmation() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "content").unwrap();
        let mut config = crate::domain::Config::new();
        config.palette = crate::domain::ColorPalette::new();
        let mut app = App::new(config, temp_file.path().to_path_buf());
        app.mode = Mode::SaveAs;
        app.filename_input = temp_file.path().to_string_lossy().to_string();

        app.try_save_as();

        assert_eq!(app.mode, Mode::Normal);
    }

    #[test]
    fn test_try_save_as_new_file_no_confirmation() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.txt");
        std::fs::write(&source_path, "content").unwrap();
        let mut config = crate::domain::Config::new();
        config.palette = crate::domain::ColorPalette::new();
        let mut app = App::new(config, source_path);
        app.mode = Mode::SaveAs;
        app.filename_input = temp_dir.path().join("new_file.txt").to_string_lossy().to_string();

        app.try_save_as();

        assert_eq!(app.mode, Mode::Normal);
        assert!(!app.editor.modified);
    }

    #[test]
    fn test_execute_save_as_updates_file_path() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.txt");
        std::fs::write(&source_path, "content").unwrap();
        let mut config = crate::domain::Config::new();
        config.palette = crate::domain::ColorPalette::new();
        let mut app = App::new(config, source_path);
        app.editor.modified = true;
        let new_path = temp_dir.path().join("new_file.txt");
        app.filename_input = new_path.to_string_lossy().to_string();

        app.execute_save_as();

        assert_eq!(app.file_path, new_path);
        assert!(!app.editor.modified);
        assert_eq!(app.mode, Mode::Normal);
        assert!(app.filename_input.is_empty());
    }

    #[test]
    fn test_execute_save_as_invalid_path_shows_error() {
        let mut app = create_test_app();
        app.filename_input = "/nonexistent/directory/file.txt".to_string();

        app.execute_save_as();

        assert_eq!(app.mode, Mode::SaveAs);
        let (msg, _) = app.status_message.as_ref().unwrap();
        assert!(msg.contains("Save failed"));
    }

    // --- Status message wiring ---

    #[test]
    fn test_undo_with_empty_stack_shows_nothing_to_undo() {
        let mut app = create_test_app();

        app.undo();

        let (msg, _) = app.status_message.as_ref().unwrap();
        assert!(msg.contains("Nothing to undo"));
    }

    #[test]
    fn test_redo_with_empty_stack_shows_nothing_to_redo() {
        let mut app = create_test_app();

        app.redo();

        let (msg, _) = app.status_message.as_ref().unwrap();
        assert!(msg.contains("Nothing to redo"));
    }

    #[test]
    fn test_paste_without_copy_shows_nothing_to_paste() {
        let mut app = create_test_app();

        app.paste_color();

        let (msg, _) = app.status_message.as_ref().unwrap();
        assert!(msg.contains("Nothing to paste"));
    }

    // --- Quick color / color picker wiring ---

    #[test]
    fn test_apply_quick_color_applies_palette_color_by_index() {
        let mut app = create_test_app();
        let red = crate::domain::ColorDef::new("RED".to_string(), crate::domain::RgbColor::new(255, 0, 0));
        let grn = crate::domain::ColorDef::new("GRN".to_string(), crate::domain::RgbColor::new(0, 255, 0));
        app.editor.config.palette.add(red);
        app.editor.config.palette.add(grn);
        app.editor.cursor = RgbPos { row: 0, col: 0, half: Half::Left };

        app.apply_quick_color(1);

        let color = app.editor.current_color().unwrap();
        assert_eq!(color, "GRN");
    }

    #[test]
    fn test_apply_quick_color_with_out_of_range_index_does_nothing() {
        let mut app = create_test_app();
        app.editor.cursor = RgbPos { row: 0, col: 0, half: Half::Left };
        let before = app.editor.current_color().unwrap().to_string();

        app.apply_quick_color(99);

        assert_eq!(app.editor.current_color().unwrap(), before);
    }

    #[test]
    fn test_apply_selected_color_sets_key_and_returns_to_normal_mode() {
        let mut app = create_test_app();
        let cyan = crate::domain::ColorDef::new("CYN".to_string(), crate::domain::RgbColor::new(0, 255, 255));
        app.editor.config.palette.add(cyan);
        app.editor.cursor = RgbPos { row: 0, col: 0, half: Half::Left };
        app.mode = Mode::ColorPick;
        app.color_picker.selected = 0;

        app.apply_selected_color();

        assert_eq!(app.editor.current_color().unwrap(), "CYN");
        assert_eq!(app.mode, Mode::Normal);
    }

    // --- Guard methods ---

    #[test]
    fn test_request_quit_when_unmodified_quits() {
        let mut app = create_test_app();

        app.request_quit();

        assert!(app.should_quit);
    }

    #[test]
    fn test_request_quit_when_modified_prompts_confirmation() {
        let mut app = create_test_app();
        app.editor.modified = true;

        app.request_quit();

        assert!(!app.should_quit);
        assert_eq!(app.mode, Mode::ConfirmQuit);
    }

    #[test]
    fn test_request_copy_when_modified_prompts_confirmation() {
        let mut app = create_test_app();
        app.editor.modified = true;

        app.request_copy();

        assert_eq!(app.mode, Mode::ConfirmCopy);
    }

    #[test]
    fn test_enter_color_pick_sets_mode() {
        let mut app = create_test_app();

        app.enter_color_pick();

        assert_eq!(app.mode, Mode::ColorPick);
    }

    #[test]
    fn test_enter_color_pick_selects_current_color_in_palette() {
        let mut app = create_test_app();
        let red = crate::domain::ColorDef::new("RED".to_string(), crate::domain::RgbColor::new(255, 0, 0));
        let grn = crate::domain::ColorDef::new("GRN".to_string(), crate::domain::RgbColor::new(0, 255, 0));
        app.editor.config.palette.add(red);
        app.editor.config.palette.add(grn);
        app.editor.cursor = RgbPos { row: 0, col: 0, half: Half::Left };
        app.set_current_key_color("GRN");

        app.enter_color_pick();

        assert_eq!(app.color_picker.selected, 1);
    }

    // --- Status expiry ---

    #[test]
    fn test_tick_clears_expired_status_message() {
        let mut app = create_test_app();
        let expired_time = Instant::now() - std::time::Duration::from_secs(5);
        app.status_message = Some(("old message".to_string(), expired_time));

        app.clear_expired_status();

        assert!(app.status_message.is_none());
    }

    #[test]
    fn test_tick_keeps_fresh_status_message() {
        let mut app = create_test_app();
        app.show_status("fresh message");

        app.clear_expired_status();

        assert!(app.status_message.is_some());
    }
}
