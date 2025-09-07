use std::io;
use rtfm_core::app_state::AppState;
use ui::tui::Tui;

#[tokio::main]
async fn main() -> io::Result<()> {
    // State initialization
    let mut app_state = AppState::new();

    // TUI setup
    let mut tui = Tui::new()?;
    tui.enter()?;

    // The main application loop, passing the state
    let result = tui.main_loop(&mut app_state).await;

    tui.exit()?;

    result
}
