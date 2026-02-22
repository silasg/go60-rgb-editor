mod app;
mod event;
mod io;
mod tui;
mod ui;

use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;
use color_eyre::Result;

use app::App;

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
    tui::install_panic_handler();
    color_eyre::install()?;

    let args = Args::parse();
    let config = io::load_config(&args.file).map_err(|e| color_eyre::eyre::eyre!(e))?;
    let mut app = App::new(config, args.file);

    let mut tui = tui::Tui::new()?;
    tui.enter()?;

    let tick_rate = Duration::from_millis(TICK_RATE_MS);

    loop {
        tui.terminal().draw(|frame| ui::draw(frame, &app))?;

        if !event::handle_events(&mut app, tick_rate)? {
            break;
        }

        app.clear_expired_status();
    }

    tui.exit()?;
    Ok(())
}
