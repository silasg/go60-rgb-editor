mod app;
mod event;
mod model;
mod parser;
mod tui;
mod ui;

use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;
use color_eyre::Result;

use app::App;
use model::Config;

const TICK_RATE_MS: u64 = 250;

#[derive(Parser, Debug)]
#[command(name = "go60-rgb-editor")]
#[command(author, version, about = "TUI RGB Underglow Editor for ZMK Keyboards", long_about = None)]
struct Args {
    /// Path to the RGB config file
    #[arg(required = true)]
    file: PathBuf,
}

fn main() -> Result<()> {
    // Install panic handler for clean terminal restore
    tui::install_panic_handler();

    // Initialize color-eyre for better error reporting
    color_eyre::install()?;

    // Parse command line arguments
    let args = Args::parse();

    // Load the config file
    let config = Config::load(&args.file).map_err(|e| color_eyre::eyre::eyre!(e))?;

    // Create application state
    let mut app = App::new(config);

    // Setup terminal
    let mut tui = tui::Tui::new()?;
    tui.enter()?;

    // Main event loop
    let tick_rate = Duration::from_millis(TICK_RATE_MS);

    loop {
        // Draw UI
        tui.terminal().draw(|frame| ui::draw(frame, &app))?;

        // Handle events
        if !event::handle_events(&mut app, tick_rate)? {
            break;
        }

        // Tick for status message timeout
        app.clear_expired_status();
    }

    // Cleanup
    tui.exit()?;

    Ok(())
}
