mod app;
mod config;
mod ui;
mod settings;

use std::io::{self, stdout};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use app::{App, AppMode, Focus, LeftColumnSection, Action};
use config::{load_config, save_config};
use ui::draw;
use std::path::{Path, PathBuf};
use open;
use viuer;
use std::process::Command;

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Load config and create app
    let config = load_config();
    let mut app = App::new(config);

    // App loop
    while app.running {
        if let AppMode::ImagePreview(path) = app.mode.clone() {
            stdout().execute(LeaveAlternateScreen)?;
            disable_raw_mode()?;

            viuer::print_from_file(&path, &viuer::Config::default()).expect("Image printing failed.");

            // Wait for any key press
            loop {
                if event::poll(std::time::Duration::from_millis(250))? {
                    if let Event::Key(_) = event::read()? {
                        break;
                    }
                }
            }

            enable_raw_mode()?;
            stdout().execute(EnterAlternateScreen)?;
            app.mode = AppMode::Normal;
        }

        terminal.draw(|f| draw(f, &mut app))?;

        if event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                handle_key_event(key.code, &mut app)?;
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn handle_key_event(key: KeyCode, app: &mut App) -> io::Result<()> {
    if key == KeyCode::Char('q') {
        app.quit();
        return Ok(());
    }
    if key == KeyCode::Tab {
        app.mode = match app.mode {
            AppMode::Normal => AppMode::Settings,
            AppMode::Settings => AppMode::Normal,
            _ => AppMode::Normal,
        };
        return Ok(());
    }

    match app.mode {
        AppMode::Normal => handle_normal_mode(key, app)?,
        AppMode::Settings => handle_settings_mode(key, app)?,
        AppMode::ConfirmingDelete => handle_delete_mode(key, app)?,
        _ => {}
    }
    Ok(())
}

fn handle_normal_mode(key: KeyCode, app: &mut App) -> io::Result<()> {
    match key {
        KeyCode::Char('c') => {
            if let Some(selected_item) = app.items.get(app.middle_col_selected) {
                app.action = Some(Action::Copy(selected_item.clone()));
            }
        },
        KeyCode::Char('x') => {
            if let Some(selected_item) = app.items.get(app.middle_col_selected) {
                app.action = Some(Action::Move(selected_item.clone()));
            }
        },
        KeyCode::Char('p') => {
            if let Some(action) = app.action.take() {
                match action {
                    Action::Copy(from) => {
                        if let Some(file_name) = from.file_name() {
                            let to = app.current_dir.join(file_name);
                            if from.is_dir() {
                                copy_dir_all(&from, &to)?;
                            } else {
                                std::fs::copy(&from, &to)?;
                            }
                        }
                    }
                    Action::Move(from) => {
                        if let Some(file_name) = from.file_name() {
                            let to = app.current_dir.join(file_name);
                            if std::fs::rename(&from, &to).is_err() {
                                // fallback to copy and delete
                                if from.is_dir() {
                                    copy_dir_all(&from, &to)?;
                                    std::fs::remove_dir_all(&from)?;
                                } else {
                                    std::fs::copy(&from, &to)?;
                                    std::fs::remove_file(&from)?;
                                }
                            }
                        }
                    }
                }
                app.load_dir()?;
            }
        },
        KeyCode::Char('d') => {
            if !app.items.is_empty() {
                app.mode = AppMode::ConfirmingDelete;
            }
        },
        KeyCode::Char('P') => {
            if let Some(cd_str) = app.current_dir.to_str() {
                app.config.pinned_dirs.push(cd_str.to_string());
            }
            save_config(&app.config)?;
            app.pinned_dirs = app.config.pinned_dirs.iter().map(PathBuf::from).collect();
        },
        // Navigation
        KeyCode::Char('j') => match app.focus {
            Focus::Left => app.left_col_down(),
            Focus::Middle => {
                if !app.items.is_empty() && app.middle_col_selected < app.items.len() - 1 {
                    app.middle_col_selected += 1;
                }
            },
            _ => {}
        },
        KeyCode::Char('k') => match app.focus {
            Focus::Left => app.left_col_up(),
            Focus::Middle => {
                if app.middle_col_selected > 0 {
                    app.middle_col_selected -= 1;
                }
            },
            _ => {}
        },
        KeyCode::Char('h') => {
            if let Focus::Middle = app.focus {
                app.focus = Focus::Left
            }
        },
        KeyCode::Char('l') => {
            match app.focus {
                Focus::Left => {
                    let new_dir = match app.left_col_selected_section {
                        LeftColumnSection::Home => app.home_dirs[app.left_col_selected_item].clone(),
                        LeftColumnSection::Pinned => app.pinned_dirs[app.left_col_selected_item].clone(),
                        LeftColumnSection::Drives => PathBuf::from(&app.drives[app.left_col_selected_item]),
                    };
                    app.current_dir = new_dir;
                    app.load_dir()?;
                    app.focus = Focus::Middle;
                },
                Focus::Middle => {
                    if let Some(selected_item) = app.items.get(app.middle_col_selected) {
                        if selected_item.is_dir() {
                            app.current_dir = selected_item.clone();
                            app.load_dir()?;
                        } else {
                            let extension = selected_item.extension().and_then(|s| s.to_str()).unwrap_or("");
                            match extension {
                                "png" | "jpg" | "jpeg" | "gif" => {
                                    app.mode = AppMode::ImagePreview(selected_item.clone());
                                }
                                "pdf" => {
                                    let tmp_path = PathBuf::from("/tmp/fm-preview.png");
                                    Command::new("pdftoppm")
                                        .arg("-png")
                                        .arg("-f")
                                        .arg("1")
                                        .arg("-l")
                                        .arg("1")
                                        .arg("-singlefile")
                                        .arg(selected_item)
                                        .arg(tmp_path.with_extension(""))
                                        .status()?;
                                    app.mode = AppMode::ImagePreview(tmp_path);
                                }
                                _ => {
                                    open::that(selected_item)?;
                                }
                            }
                        }
                    } else {
                        app.focus = Focus::Right;
                    }
                },
                _ => {}
            }
        },
        _ => {}
    }
    Ok(())
}

fn handle_settings_mode(key: KeyCode, app: &mut App) -> io::Result<()> {
    match key {
        KeyCode::Char('j') => {
            if app.settings_selected < settings::THEMES.len() - 1 {
                app.settings_selected += 1;
            }
        },
        KeyCode::Char('k') => {
            if app.settings_selected > 0 {
                app.settings_selected -= 1;
            }
        },
        KeyCode::Enter => {
            let selected_theme = &settings::THEMES[app.settings_selected];
            app.config.colors = selected_theme.colors.to_colors();
            save_config(&app.config)?;
            app.mode = AppMode::Normal;
        },
        KeyCode::Esc | KeyCode::Char('q') => {
            app.mode = AppMode::Normal;
        }
        _ => {}
    }
    Ok(())
}

fn handle_delete_mode(key: KeyCode, app: &mut App) -> io::Result<()> {
    match key {
        KeyCode::Char('y') => {
            let path = app.items[app.middle_col_selected].clone();
            if path.is_dir() {
                std::fs::remove_dir_all(&path)?;
            } else {
                std::fs::remove_file(&path)?;
            }
            app.load_dir()?;
            app.mode = AppMode::Normal;
        },
        KeyCode::Char('n') | KeyCode::Esc => {
            app.mode = AppMode::Normal;
        },
        _ => {}
    }
    Ok(())
}


fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    if !dst.exists() {
        std::fs::create_dir_all(dst)?;
    }

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let dst_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            std::fs::copy(entry.path(), &dst_path)?;
        }
    }
    Ok(())
}
