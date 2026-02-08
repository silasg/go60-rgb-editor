use std::io::{self, stdout, Stdout};

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Tui {
    pub fn new() -> io::Result<Self> {
        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        Ok(Self { terminal })
    }

    pub fn enter(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen)?;
        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }

    pub fn exit(&mut self) -> io::Result<()> {
        disable_raw_mode()?;
        execute!(stdout(), LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    pub fn terminal(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        &mut self.terminal
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = self.exit();
    }
}

/// Install panic handler that restores terminal state before printing the panic.
pub fn install_panic_handler() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen);
        original_hook(panic);
    }));
}
