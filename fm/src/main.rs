mod app;
mod config;
mod ui;

use std::io::{self, stdout};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use app::{App, AppMode, Action};
use config::load_config;
use ui::draw;
use std::fs;
use std::path::PathBuf;
use open;


fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Load config and create app
    let config = load_config();
    let mut app = App::new(config);

    // App loop
    loop {
        terminal.draw(|f| draw(f, &mut app))?;

        // Event handling
        if event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if app.deleting {
                    if key.code == KeyCode::Char('y') {
                        let path = app.items[app.selected].clone();
                        if path.is_dir() {
                            fs::remove_dir_all(&path)?;
                        } else {
                            fs::remove_file(&path)?;
                        }
                        app.load_dir()?;
                    }
                    app.deleting = false;
                } else {
                    match app.mode {
                        AppMode::Normal => {
                            match key.code {
                                KeyCode::Char('q') => break,
                                KeyCode::Char('c') => {
                                    if let Some(selected_item) = app.items.get(app.selected) {
                                        app.action = Some(Action::Copy(selected_item.clone()));
                                    }
                                }
                                KeyCode::Char('x') => {
                                    if let Some(selected_item) = app.items.get(app.selected) {
                                        app.action = Some(Action::Move(selected_item.clone()));
                                    }
                                }
                                KeyCode::Char('p') => {
                                    if let Some(action) = app.action.take() {
                                        match action {
                                            Action::Copy(from) => {
                                                let to = app.current_dir.join(from.file_name().unwrap());
                                                fs::copy(from, to)?;
                                            }
                                            Action::Move(from) => {
                                                let to = app.current_dir.join(from.file_name().unwrap());
                                                fs::rename(from, to)?;
                                            }
                                        }
                                        app.load_dir()?;
                                    }
                                }
                                KeyCode::Char('d') => {
                                    if !app.items.is_empty() {
                                        app.deleting = true;
                                    }
                                }
                                KeyCode::Char('m') => {
                                    app.mode = AppMode::Drives;
                                    app.load_drives();
                                }
                                KeyCode::Down => {
                                    if !app.items.is_empty() && app.selected < app.items.len() - 1 {
                                        app.selected += 1;
                                    }
                                }
                                KeyCode::Up => {
                                    if app.selected > 0 {
                                        app.selected -= 1;
                                    }
                                }
                                KeyCode::Enter => {
                                    if let Some(selected_item) = app.items.get(app.selected) {
                                        if selected_item.is_dir() {
                                            app.current_dir = selected_item.clone();
                                            app.load_dir()?;
                                        }
                                    }
                                }
                                KeyCode::Char('o') => {
                                    if let Some(selected_item) = app.items.get(app.selected) {
                                        open::that(selected_item)?;
                                    }
                                }
                                KeyCode::Backspace => {
                                    if app.current_dir.parent().is_some() {
                                        app.current_dir.pop();
                                        app.load_dir()?;
                                    }
                                }
                                _ => {}
                            }
                        }
                        AppMode::Drives => {
                            match key.code {
                                KeyCode::Char('q') | KeyCode::Esc => {
                                    app.mode = AppMode::Normal;
                                    app.selected = 0;
                                }
                                KeyCode::Down => {
                                    if !app.drives.is_empty() && app.selected < app.drives.len() - 1 {
                                        app.selected += 1;
                                    }
                                }
                                KeyCode::Up => {
                                    if app.selected > 0 {
                                        app.selected -= 1;
                                    }
                                }
                                KeyCode::Enter => {
                                    if let Some(drive) = app.drives.get(app.selected) {
                                        app.current_dir = PathBuf::from(drive);
                                        app.load_dir()?;
                                        app.mode = AppMode::Normal;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
