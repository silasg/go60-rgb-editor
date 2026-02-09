use std::io::Write;
use std::process::{Command, Stdio};

/// Try platform clipboard commands: pbcopy (macOS), xclip/xsel (Linux), clip (Windows)
fn spawn_clipboard_process() -> std::io::Result<std::process::Child> {
    Command::new("pbcopy")
        .stdin(Stdio::piped())
        .spawn()
        .or_else(|_| {
            Command::new("xclip")
                .args(["-selection", "clipboard"])
                .stdin(Stdio::piped())
                .spawn()
        })
        .or_else(|_| {
            Command::new("xsel")
                .args(["--clipboard", "--input"])
                .stdin(Stdio::piped())
                .spawn()
        })
        .or_else(|_| Command::new("clip").stdin(Stdio::piped()).spawn())
}

/// Copy the given content to the system clipboard.
pub fn copy_to_clipboard(content: &str) -> Result<(), String> {
    let mut child = spawn_clipboard_process().map_err(|_| "No clipboard command available")?;

    let write_result = match child.stdin.take() {
        Some(mut stdin) => stdin.write_all(content.as_bytes()),
        None => Err(std::io::Error::other("no stdin")),
    };

    if write_result.is_ok() {
        let _ = child.wait();
        Ok(())
    } else {
        Err("Clipboard write failed".to_string())
    }
}
