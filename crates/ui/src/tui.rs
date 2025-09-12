use crossterm::{
    event::{KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::{CrosstermBackend, Terminal};
use std::io::{self, stdout, Stdout};
use rtfm_core::app_state::{AppState, InputMode, CreateFileType};

pub struct Tui {
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Tui {
    pub fn new() -> io::Result<Self> {
        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        Ok(Self { terminal })
    }

    pub fn enter(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        Ok(())
    }

    pub fn exit(&mut self) -> io::Result<()> {
        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }
}

/// Handles key presses and returns `false` if the app should quit.
pub fn handle_key_press(key: KeyEvent, app_state: &mut AppState) -> bool {
    // Global keybindings
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('n') => {
                log::info!("Ctrl+N key press detected");
                app_state.new_tab();
                return true;
            }
            KeyCode::Char('w') => {
                app_state.close_tab();
                return true;
            }
            KeyCode::Tab => {
                app_state.next_tab();
                return true;
            }
            KeyCode::Char('`') => {
                app_state.toggle_terminal();
                return true;
            }
            _ => {}
        }
    }

    // Ctrl-Shift-Tab for previous tab
    if key.modifiers.contains(KeyModifiers::CONTROL | KeyModifiers::SHIFT) && key.code == KeyCode::Tab {
         app_state.previous_tab();
         return true;
    }

    // Alt-number for tab switching
    if key.modifiers.contains(KeyModifiers::ALT) {
        match key.code {
            KeyCode::Char(c @ '1'..='9') => {
                let tab_index = c.to_digit(10).unwrap_or(0) as usize;
                if tab_index > 0 && tab_index <= app_state.tabs.len() {
                    app_state.active_tab_index = tab_index - 1;
                }
                return true;
            }
            KeyCode::Char('t') => {
                app_state.toggle_tabs();
                return true;
            }
            _ => {}
        }
    }
    // crossterm might send BackTab for Shift-Tab
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::BackTab {
        app_state.previous_tab();
        return true;
    }


    if app_state.show_confirmation {
        match key.code {
            KeyCode::Char('y') => {
                app_state.confirm_delete();
                return true;
            }
            KeyCode::Char('n') | KeyCode::Esc => {
                app_state.cancel_delete();
                return true;
            }
            _ => {}
        }
    }

    if app_state.show_input_dialog {
        match key.code {
            KeyCode::Char(c) => {
                app_state.input_buffer.push(c);
                return true;
            }
            KeyCode::Backspace => {
                app_state.input_buffer.pop();
                return true;
            }
            KeyCode::Enter => {
                app_state.create_item();
                app_state.show_input_dialog = false;
                return true;
            }
            KeyCode::Esc => {
                app_state.show_input_dialog = false;
                app_state.input_buffer.clear();
                return true;
            }
            _ => {}
        }
    }

    // Normal mode keybindings
    use rtfm_core::app_state::FocusBlock;
    match app_state.input_mode {
        InputMode::Normal => {
            match key.code {
                KeyCode::Char('n') => {
                    app_state.input_mode = InputMode::Create;
                    return true;
                }
                KeyCode::Char('q') => return false, // Signal to quit
                KeyCode::Tab => app_state.cycle_focus(),
                KeyCode::Char('.') => app_state.toggle_hidden_files(),
                KeyCode::Char('j') | KeyCode::Down => {
                    match app_state.focus {
                        FocusBlock::Middle => {
                            let show_hidden = app_state.show_hidden_files;
                            app_state.get_active_tab_mut().move_cursor_down(show_hidden);
                        },
                        _ => {
                            app_state.move_left_pane_cursor_down();
                        }
                    }
                },
                KeyCode::Char('k') | KeyCode::Up => {
                    match app_state.focus {
                        FocusBlock::Middle => {
                            let show_hidden = app_state.show_hidden_files;
                            app_state.get_active_tab_mut().move_cursor_up(show_hidden);
                        },
                        _ => {
                            app_state.move_left_pane_cursor_up();
                        }
                    }
                },
                KeyCode::Char('h') | KeyCode::Left => {
                    if app_state.focus == FocusBlock::Middle {
                        let show_hidden = app_state.show_hidden_files;
                        app_state.get_active_tab_mut().leave_directory(show_hidden);
                    }
                },
                KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                    if app_state.focus == FocusBlock::Middle {
                        let show_hidden = app_state.show_hidden_files;
                        app_state.get_active_tab_mut().enter_directory(show_hidden);
                    }
                },
                KeyCode::Char('y') => app_state.yank_selection(),
                KeyCode::Char('x') => app_state.cut_selection(),
                KeyCode::Char('d') => app_state.delete_selection(),
                KeyCode::Char('p') => app_state.paste(),
                KeyCode::Char('m') => app_state.add_bookmark(),
                _ => {}
            }
        },
        InputMode::Create => match key.code {
            KeyCode::Char('f') => {
                app_state.create_file_type = Some(CreateFileType::File);
                app_state.show_input_dialog = true;
                app_state.input_mode = InputMode::Normal;
                return true;
            }
            KeyCode::Char('d') => {
                app_state.create_file_type = Some(CreateFileType::Directory);
                app_state.show_input_dialog = true;
                app_state.input_mode = InputMode::Normal;
                return true;
            }
            _ => {
                app_state.input_mode = InputMode::Normal;
                return true;
            }
        }
    }
    true
}
