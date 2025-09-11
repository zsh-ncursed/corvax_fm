use std::path::PathBuf;
use std::sync::Arc;
use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyEventKind};
use futures::StreamExt;
use raster::PreviewImage;
use rtfm_core::app_state::{AppState, FocusBlock};
use rtfm_core::preview::{PreviewController, PreviewEvent, PreviewState, TaskId};
use config::BackendType;
use output_backends::kitty::KittyBackend;
use output_backends::OutputBackend;
use ui::tui::{self, Tui};
use tokio::sync::mpsc;

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
    preview_controller: PreviewController,
    kitty_backend: Box<dyn OutputBackend>,
    preview_event_rx: mpsc::Receiver<PreviewEvent>,
    last_previewed_path: Option<PathBuf>,
    current_image_preview: Option<Arc<PreviewImage>>,
    current_task_id: Option<TaskId>,
}

impl App {
    fn new() -> Result<Self> {
        let (preview_event_tx, preview_event_rx) = mpsc::channel(10);
        let app_state = AppState::new();

        let backend: Box<dyn OutputBackend> = match app_state.config.preview.backend {
            BackendType::Kitty => Box::new(KittyBackend::new()),
            // BackendType::Sixel => Box::new(SixelBackend::new()),
        };

        Ok(Self {
            app_state,
            tui: Tui::new()?,
            preview_controller: PreviewController::new(preview_event_tx),
            kitty_backend: backend,
            preview_event_rx,
            last_previewed_path: None,
            current_image_preview: None,
            current_task_id: None,
        })
    }

    async fn run(&mut self) -> Result<()> {
        self.tui.enter()?;
        let mut event_stream = EventStream::new();

        loop {
            self.app_state.task_manager.process_pending_tasks();
            if self.app_state.task_manager.update_task_statuses() {
                let show_hidden = self.app_state.show_hidden_files;
                self.app_state.get_active_tab_mut().update_entries(show_hidden);
            }

            self.check_and_request_preview().await;

            self.tui.terminal.draw(|frame| {
                ui::layout::render_main_layout(frame, &self.app_state);
            })?;

            if let Some(image) = &self.current_image_preview {
                let right_pane_area = self.tui.get_right_pane_area(&self.app_state);
                let draw_area = ratatui::layout::Rect::new(
                    right_pane_area.x + 1,
                    right_pane_area.y + 1,
                    right_pane_area.width.saturating_sub(2),
                    right_pane_area.height.saturating_sub(2),
                );
                self.kitty_backend.draw(image, draw_area, &mut std::io::stdout())?;
            }

            tokio::select! {
                Some(Ok(event)) = event_stream.next() => {
                    if let Event::Key(key) = event {
                        if key.kind == KeyEventKind::Press {
                            if !tui::handle_key_press(key, &mut self.app_state) {
                                break; // Quit
                            }
                        }
                    }
                },
                Some(preview_event) = self.preview_event_rx.recv() => {
                    self.handle_preview_event(preview_event);
                }
            };
        }
        self.tui.exit()?;
        Ok(())
    }

    fn handle_preview_event(&mut self, event: PreviewEvent) {
        let event_task_id = match &event {
            PreviewEvent::ThumbnailLoaded(id, _) => *id,
            PreviewEvent::FinalImageLoaded(id, _) => *id,
            PreviewEvent::Error(id, _) => *id,
        };

        if self.current_task_id != Some(event_task_id) {
            return; // This event is for a task that has been superseded.
        }

        match event {
            PreviewEvent::ThumbnailLoaded(_, image_arc) | PreviewEvent::FinalImageLoaded(_, image_arc) => {
                self.current_image_preview = Some(image_arc.clone());
                let mut state_guard = self.app_state.get_active_tab_mut().preview_state.lock().unwrap();
                // Avoid replacing a final image with a (late-arriving) thumbnail
                if let PreviewState::Image(existing_img) = &*state_guard {
                    if image_arc.width > existing_img.width {
                        *state_guard = PreviewState::Image(image_arc);
                    }
                } else {
                    *state_guard = PreviewState::Image(image_arc);
                }
            }
            PreviewEvent::Error(_, msg) => {
                // Only show an error if we haven't already loaded an image for this task.
                if self.current_image_preview.is_none() {
                    let mut state_guard = self.app_state.get_active_tab_mut().preview_state.lock().unwrap();
                    *state_guard = PreviewState::Error(msg);
                    self.current_image_preview = None;
                }
            }
        }
    }

    async fn check_and_request_preview(&mut self) {
        if self.app_state.focus != FocusBlock::Middle {
            if self.current_image_preview.is_some() {
                self.kitty_backend.clear(&mut std::io::stdout()).unwrap();
                self.current_image_preview = None;
                self.last_previewed_path = None;
                self.current_task_id = None;
            }
            return;
        }

        let selected_path = self.app_state.get_active_tab().get_selected_entry_path();

        let should_clear = match selected_path {
            Some(ref path) => self.last_previewed_path.as_ref() != Some(path),
            None => self.last_previewed_path.is_some(),
        };

        if should_clear {
            self.current_image_preview = None;
            self.kitty_backend.clear(&mut std::io::stdout()).unwrap();
            self.last_previewed_path = selected_path.clone();
            self.current_task_id = None;

            if let Some(path) = selected_path {
                let width_px = self.app_state.config.preview.resolution.width;
                let height_px = self.app_state.config.preview.resolution.height;
                let progressive = self.app_state.config.preview.progressive;

                let mut state_guard = self.app_state.get_active_tab_mut().preview_state.lock().unwrap();
                *state_guard = PreviewState::Loading;

                let task_id = self.preview_controller.request_preview(path, width_px, height_px, progressive).await;
                self.current_task_id = Some(task_id);
            } else {
                 let mut state_guard = self.app_state.get_active_tab_mut().preview_state.lock().unwrap();
                *state_guard = PreviewState::Empty;
            }
        }
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
