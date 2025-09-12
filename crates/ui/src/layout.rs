use crate::{left_pane, middle_pane, top_bar};
use ratatui::{
    prelude::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use rtfm_core::app_state::{AppState, CreateFileType};
use rtfm_core::clipboard::ClipboardMode;

fn render_input_dialog(frame: &mut Frame, app_state: &AppState) {
    let file_type = match app_state.create_file_type {
        Some(CreateFileType::File) => "file",
        Some(CreateFileType::Directory) => "directory",
        None => "",
    };
    let title = format!("Create new {}", file_type);
    let text = Paragraph::new(app_state.input_buffer.as_str())
        .block(Block::default().title(title).borders(Borders::ALL))
        .style(Style::default().fg(Color::Yellow));

    // Center the dialog
    let area = centered_rect(50, 20, frame.size());
    frame.render_widget(Clear, area); //this clears the background
    frame.render_widget(text, area);
}

fn render_confirmation_dialog(frame: &mut Frame, app_state: &AppState) {
    let message = &app_state.confirmation_message;
    let text = Paragraph::new(message.as_str())
        .block(Block::default().title("Confirmation").borders(Borders::ALL))
        .style(Style::default().fg(Color::Yellow));

    // Center the dialog
    let area = centered_rect(50, 20, frame.size());
    frame.render_widget(Clear, area); //this clears the background
    frame.render_widget(text, area);
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}


pub fn render_main_layout(frame: &mut Frame, app_state: &AppState) {
    let top_bar_height = if app_state.show_tabs { 2 } else { 0 };
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(top_bar_height), // Top bar
            Constraint::Min(0),    // Main content
            Constraint::Length(6), // Footer
        ])
        .split(frame.size());

    let top_bar_area = main_chunks[0];
    let main_area = main_chunks[1];
    let footer_area = main_chunks[2];

    // --- Top Bar (Tabs) ---
    top_bar::render_top_bar(frame, top_bar_area, app_state);

    // --- Main Area (Left, Middle) ---
    let main_horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20), // Left
            Constraint::Percentage(80), // Middle
        ])
        .split(main_area);

    let left_pane_area = main_horizontal_chunks[0];
    let middle_pane_area = main_horizontal_chunks[1];

    // --- Render Panes with Borders ---
    let active_tab = app_state.get_active_tab();

    // Left Pane
    let left_pane_block = Block::default().borders(Borders::ALL);
    let left_pane_inner_area = left_pane_block.inner(left_pane_area);
    frame.render_widget(left_pane_block, left_pane_area);
    render_left_pane(frame, left_pane_inner_area, app_state);

    // Middle Pane
    let middle_pane_block = Block::default()
        .title(format!("Current: {}", active_tab.current_dir.display()))
        .borders(Borders::ALL);
    let middle_pane_inner_area = middle_pane_block.inner(middle_pane_area);
    frame.render_widget(middle_pane_block, middle_pane_area);
    middle_pane::render_middle_pane(frame, middle_pane_inner_area, active_tab);

    // --- Footer (Tasks, Info) ---
    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Tasks
            Constraint::Percentage(50), // Info
        ])
        .split(footer_area);

    render_tasks_footer(frame, footer_chunks[0], app_state);
    render_info_panel(frame, footer_chunks[1], app_state);

    if app_state.show_confirmation {
        render_confirmation_dialog(frame, app_state);
    }
    if app_state.show_input_dialog {
        render_input_dialog(frame, app_state);
    }
}

fn render_left_pane(frame: &mut Frame, area: Rect, app_state: &AppState) {
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(33), // XDG Folders
            Constraint::Percentage(33), // Bookmarks
            Constraint::Percentage(34), // Mounts/Remotes
        ])
        .split(area);

    left_pane::render_xdg_block(frame, left_chunks[0], app_state);
    left_pane::render_bookmarks_block(frame, left_chunks[1], app_state);
    left_pane::render_mounts_block(frame, left_chunks[2], app_state);
}

fn render_tasks_footer(frame: &mut Frame, area: Rect, app_state: &AppState) {
    let block = Block::default().borders(Borders::ALL).title("Tasks");
    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let tasks = app_state.task_manager.get_tasks();
    let task_items: Vec<ListItem> = tasks
        .iter()
        .map(|task| ListItem::new(task.description.clone()))
        .collect();

    let task_list = List::new(task_items);

    frame.render_widget(task_list, inner_area);
}

fn render_info_panel(frame: &mut Frame, area: Rect, app_state: &AppState) {
    let block = Block::default().borders(Borders::ALL).title("Info");
    let inner_area = block.inner(area);

    // Always display clipboard info
    let clipboard = &app_state.clipboard;
    let clipboard_info = if !clipboard.paths.is_empty() {
        let mode = match clipboard.mode {
            Some(ClipboardMode::Copy) => "Copy",
            Some(ClipboardMode::Move) => "Move",
            None => "None",
        };
        format!("Buffer: {} files ({})", clipboard.paths.len(), mode)
    } else {
        "Buffer: Empty".to_string()
    };

    let paragraph = Paragraph::new(clipboard_info)
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, inner_area);
    frame.render_widget(block, area);
}
