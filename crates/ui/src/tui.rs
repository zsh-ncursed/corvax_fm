use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::{CrosstermBackend, Terminal};
use std::io::{self, stdout, Stdout};
use crate::layout;
use rtfm_core::app_state::AppState;

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
        stdout().execute(EnterAlternateScreen)?;
        Ok(())
    }

    pub fn exit(&mut self) -> io::Result<()> {
        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }

    pub async fn main_loop(&mut self, app_state: &mut AppState) -> io::Result<()> {
        loop {
            app_state.task_manager.process_pending_tasks();
            app_state.task_manager.update_task_statuses();

            self.terminal.draw(|frame| {
                layout::render_main_layout(frame, app_state);
            })?;

            if event::poll(std::time::Duration::from_millis(16))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        if !handle_key_press(key, app_state) {
                            break; // Quit signal received
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

/// Handles key presses and returns `false` if the app should quit.
fn handle_key_press(key: KeyEvent, app_state: &mut AppState) -> bool {
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


    // Normal mode keybindings
    use rtfm_core::app_state::FocusBlock;
    match key.code {
        KeyCode::Char('q') => return false, // Signal to quit
        KeyCode::Char('i') => app_state.update_info_panel(),
        KeyCode::Tab => app_state.cycle_focus(),
        KeyCode::Char('.') => app_state.toggle_hidden_files(),
        KeyCode::Char('j') | KeyCode::Down => {
            match app_state.focus {
                FocusBlock::Middle => {
                    let show_hidden = app_state.show_hidden_files;
                    app_state.get_active_tab_mut().move_cursor_down(show_hidden);
                    app_state.clear_info_panel();
                },
                _ => {
                    app_state.move_left_pane_cursor_down();
                    app_state.clear_info_panel();
                }
            }
        },
        KeyCode::Char('k') | KeyCode::Up => {
            match app_state.focus {
                FocusBlock::Middle => {
                    let show_hidden = app_state.show_hidden_files;
                    app_state.get_active_tab_mut().move_cursor_up(show_hidden);
                    app_state.clear_info_panel();
                },
                _ => {
                    app_state.move_left_pane_cursor_up();
                    app_state.clear_info_panel();
                }
            }
        },
        KeyCode::Char('h') | KeyCode::Left => {
            if app_state.focus == FocusBlock::Middle {
                let show_hidden = app_state.show_hidden_files;
                app_state.get_active_tab_mut().leave_directory(show_hidden);
                app_state.clear_info_panel();
            }
        },
        KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
            if app_state.focus == FocusBlock::Middle {
                let show_hidden = app_state.show_hidden_files;
                app_state.get_active_tab_mut().enter_directory(show_hidden);
                app_state.clear_info_panel();
            }
        },
        KeyCode::Char('y') => app_state.yank_selection(),
        KeyCode::Char('d') => app_state.cut_selection(),
        KeyCode::Char('p') => app_state.paste(),
        KeyCode::Char('m') => app_state.add_bookmark(),
        _ => {}
    }
    true
}
