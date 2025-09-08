use ratatui::{
    prelude::{Color, Constraint, Direction, Layout, Modifier, Rect, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
    Frame,
};
use rtfm_core::app_state::{AppState, TabState};
use rtfm_core::preview::PreviewState;
use crate::left_pane;

pub fn render_main_layout(frame: &mut Frame, app_state: &AppState) {
    // The spec defines a top bar for tabs
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Tabs
            Constraint::Min(0),    // Main content
            Constraint::Length(6), // Footer
        ])
        .split(frame.size());

    let top_bar = main_chunks[0];
    let main_area = main_chunks[1];
    let footer = main_chunks[2];

    // --- Top Bar (Tabs) ---
    render_tabs(frame, top_bar, app_state);

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
    render_middle_pane(frame, middle_pane_inner_area, active_tab);

    // Right Pane
    let right_pane_block = Block::default().title("Preview").borders(Borders::ALL);
    let right_pane_inner_area = right_pane_block.inner(right_pane_area);
    frame.render_widget(right_pane_block, right_pane_area);
    render_right_pane(frame, right_pane_inner_area, active_tab);


    // --- Footer (Terminal, Info) ---
    if app_state.show_terminal {
        let footer_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // Terminal
                Constraint::Percentage(50), // Info
            ])
            .split(footer);

        frame.render_widget(
            Block::new().borders(Borders::ALL).title("Terminal"),
            footer_chunks[0],
        );
        render_info_footer(frame, footer_chunks[1], app_state);
    } else {
        // If terminal is hidden, info pane takes the whole footer
        render_info_footer(frame, footer, app_state);
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

fn render_tabs(frame: &mut Frame, area: Rect, app_state: &AppState) {
    log::debug!("Rendering tabs. Tab count: {}", app_state.tabs.len());
    let titles: Vec<String> = app_state
        .tabs
        .iter()
        .map(|tab| {
            format!(
                "{} {}",
                tab.id + 1,
                tab.current_dir.file_name().unwrap_or_default().to_string_lossy()
            )
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::BOTTOM))
        .select(app_state.active_tab_index)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    frame.render_widget(tabs, area);
}

use std::path::Path;

fn get_icon_for_path(path: &Path, is_dir: bool) -> &'static str {
    if is_dir {
        return ""; // Folder icon
    }
    match path.extension().and_then(|s| s.to_str()) {
        Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | Some("bmp") => "", // Image icon
        Some("zip") | Some("gz") | Some("tar") | Some("rar") | Some("7z") => "", // Archive icon
        Some("txt") | Some("md") => "", // Text file icon
        Some("mp3") | Some("wav") | Some("flac") => "🎵", // Audio icon
        Some("mp4") | Some("mkv") | Some("mov") | Some("avi") => "", // Video icon
        Some("rs") | Some("js") | Some("html") | Some("css") | Some("py") | Some("toml") | Some("sh") => "", // Code icon
        _ => "", // Generic file icon
    }
}

fn render_middle_pane(frame: &mut Frame, area: Rect, tab_state: &TabState) {
    let items: Vec<ListItem> = tab_state
        .entries
        .iter()
        .map(|entry| {
            let is_hidden = entry.name.starts_with('.');
            let style = if is_hidden { Style::default().fg(Color::DarkGray) } else { Style::default() };

            let icon = get_icon_for_path(&entry.path, entry.is_dir);
            let mut name = format!("{} {}", icon, entry.name.clone());
            if entry.is_dir {
                name.push('/');
            }
            ListItem::new(name).style(style)
        })
        .collect();

    let list = List::new(items)
        .highlight_symbol(">> ");

    let mut list_state = ListState::default();
    list_state.select(Some(tab_state.cursor));

    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_info_footer(frame: &mut Frame, area: Rect, app_state: &AppState) {
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

fn render_right_pane(frame: &mut Frame, area: Rect, tab_state: &TabState) {
    let preview_state = tab_state.preview_state.lock().unwrap();

    match &*preview_state {
        PreviewState::Directory(entries) => {
            let items: Vec<ListItem> = entries
                .iter()
                .map(|entry| {
                    let is_hidden = entry.name.starts_with('.');
                    let style = if is_hidden { Style::default().fg(Color::DarkGray) } else { Style::default() };

                    let icon = get_icon_for_path(&entry.path, entry.is_dir);
                    let mut name = format!("{} {}", icon, entry.name.clone());
                    if entry.is_dir {
                        name.push('/');
                    }
                    ListItem::new(name).style(style)
                })
                .collect();
            let list = List::new(items);
            frame.render_widget(list, area);
        }
        _ => {
            let content = match &*preview_state {
                PreviewState::Empty => "Empty",
                PreviewState::Loading => "Loading...",
                PreviewState::Text(text) => text,
                PreviewState::Error(e) => e,
                PreviewState::Directory(_) => unreachable!(),
            };
            let paragraph = Paragraph::new(content);
            frame.render_widget(paragraph, area);
        }
    }
}
