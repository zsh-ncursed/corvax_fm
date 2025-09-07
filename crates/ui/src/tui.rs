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
    if key.modifiers == KeyModifiers::CONTROL {
        match key.code {
            KeyCode::Char('n') => {
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
    if key.modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) && key.code == KeyCode::Tab {
         app_state.previous_tab();
         return true;
    }
    // crossterm might send BackTab for Shift-Tab
    if key.code == KeyCode::BackTab && key.modifiers == KeyModifiers::CONTROL {
        app_state.previous_tab();
        return true;
    }


    // Normal mode keybindings
    match key.code {
        KeyCode::Char('q') => return false, // Signal to quit
        KeyCode::Char('y') => {
            app_state.yank_selection();
        }
        KeyCode::Char('d') => {
            app_state.cut_selection();
        }
        KeyCode::Char('p') => {
            app_state.paste();
        }
        KeyCode::Char('m') => {
            app_state.add_bookmark();
        }
        _ => {
            let active_tab = app_state.get_active_tab_mut();
            match key.code {
                KeyCode::Char('j') | KeyCode::Down => active_tab.move_cursor_down(),
                KeyCode::Char('k') | KeyCode::Up => active_tab.move_cursor_up(),
                KeyCode::Char('h') | KeyCode::Left => active_tab.leave_directory(),
                KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => active_tab.enter_directory(),
                _ => {}
            }
        }
    }
    true
}
