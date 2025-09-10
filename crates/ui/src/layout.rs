use crate::{left_pane, middle_pane, right_pane, top_bar};
use ratatui::{
    prelude::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use rtfm_core::app_state::AppState;
use rtfm_core::clipboard::ClipboardMode;

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

    if let Some(path_to_delete) = &app_state.show_delete_confirmation {
        let popup_title = "Confirm Deletion";
        let file_name = path_to_delete.file_name().unwrap_or_default().to_string_lossy();
        let popup_text = format!("Are you sure you want to delete '{}'? (y/n)", file_name);
        render_popup(frame, popup_title, &popup_text);
    }

    // --- Main Area (Left, Middle, Right) ---
    let main_horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20), // Left
            Constraint::Percentage(80), // Middle + Right
        ])
        .split(main_area);

    let left_pane_area = main_horizontal_chunks[0];
    let middle_right_area = main_horizontal_chunks[1];

    let middle_right_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(55), // Middle
            Constraint::Percentage(45), // Right
        ])
        .split(middle_right_area);

    let middle_pane_area = middle_right_chunks[0];
    let right_pane_area = middle_right_chunks[1];

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

    // Right Pane
    let right_pane_block = Block::default().title("Preview").borders(Borders::ALL);
    let right_pane_inner_area = right_pane_block.inner(right_pane_area);
    frame.render_widget(right_pane_block, right_pane_area);
    right_pane::render_right_pane(frame, right_pane_inner_area, active_tab);

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

    let content = app_state.info_panel_content.lock().unwrap();
    let text = if let Some(content) = content.as_ref() {
        content.clone()
    } else {
        String::new()
    };

    // Always display clipboard info
    let clipboard = &app_state.clipboard;
    let clipboard_info = if !clipboard.paths.is_empty() {
        let mode = match clipboard.mode {
            Some(ClipboardMode::Copy) => "Copy",
            Some(ClipboardMode::Move) => "Move",
            None => "None",
        };
        format!("\n\nBuffer: {} files ({})", clipboard.paths.len(), mode)
    } else {
        "\n\nBuffer: Empty".to_string()
    };

    let final_text = format!("{}{}", text, clipboard_info);
    let paragraph = Paragraph::new(final_text)
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, inner_area);
    frame.render_widget(block, area);
}

fn render_popup(frame: &mut Frame, title: &str, text: &str) {
    let area = centered_rect(60, 20, frame.size());
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));

    let paragraph = Paragraph::new(text)
        .block(block.clone())
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    frame.render_widget(Clear, area); //this clears the background
    frame.render_widget(paragraph, area);
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
