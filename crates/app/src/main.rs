use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyEventKind};
use futures::StreamExt;
use rtfm_core::app_state::AppState;
use ui::tui::{self, Tui};

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

struct App {
    app_state: AppState,
    tui: Tui,
}

impl App {
    fn new() -> Result<Self> {
        let app_state = AppState::new();
        Ok(Self {
            app_state,
            tui: Tui::new()?,
        })
    }

    async fn run(&mut self) -> Result<()> {
        self.tui.enter()?;
        let mut event_stream = EventStream::new();

        'main: loop {
            self.app_state.task_manager.process_pending_tasks();

            self.tui.terminal.draw(|frame| {
                ui::layout::render_main_layout(frame, &self.app_state);
            })?;

            tokio::select! {
                biased;
                maybe_event = event_stream.next() => {
                    if let Some(Ok(event)) = maybe_event {
                        if let Event::Key(key) = event {
                            if key.kind == KeyEventKind::Press {
                                if !tui::handle_key_press(key, &mut self.app_state) {
                                    break 'main;
                                }
                            }
                        }
                    } else {
                        break 'main;
                    }
                }
                _ = self.app_state.task_manager.wait_for_event() => {
                    let show_hidden = self.app_state.show_hidden_files;
                    self.app_state.get_active_tab_mut().update_entries(show_hidden);
                }
            }
        }
        self.tui.exit()?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    setup_logger().expect("Failed to set up logger");
    log::info!("Application starting up");

    if let Err(e) = App::new()?.run().await {
        eprintln!("Error: {:?}", e);
        // To ensure the terminal state is restored.
        let mut tui = Tui::new().unwrap();
        tui.exit().unwrap();
        std::process::exit(1);
    }

    log::info!("Application shutting down");
    Ok(())
}
