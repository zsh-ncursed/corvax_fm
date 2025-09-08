use std::io;
use rtfm_core::app_state::AppState;
use ui::tui::Tui;

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file("rtfm.log")?)
        .apply()?;
    Ok(())
}


#[tokio::main]
async fn main() -> io::Result<()> {
    setup_logger().expect("Failed to set up logger");
    log::info!("Application starting up");

    // State initialization
    let mut app_state = AppState::new();

    // TUI setup
    let mut tui = Tui::new()?;
    tui.enter()?;

    // The main application loop, passing the state
    let result = tui.main_loop(&mut app_state).await;

    tui.exit()?;
    log::info!("Application shutting down");

    result
}
