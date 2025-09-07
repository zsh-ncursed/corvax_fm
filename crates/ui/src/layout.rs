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

    let left_pane = main_horizontal_chunks[0];
    let middle_right_area = main_horizontal_chunks[1];

    let middle_right_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(55), // Middle
            Constraint::Percentage(45), // Right
        ])
        .split(middle_right_area);

    let middle_pane = middle_right_chunks[0];
    let right_pane = middle_right_chunks[1];

    // Render left pane with its vertical blocks
    render_left_pane(frame, left_pane, app_state);

    // Render middle pane using the active tab's state
    let active_tab = app_state.get_active_tab();
    render_middle_pane(frame, middle_pane, active_tab);

    // Render right pane
    render_right_pane(frame, right_pane, active_tab);

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

fn render_middle_pane(frame: &mut Frame, area: Rect, tab_state: &TabState) {
    let items: Vec<ListItem> = tab_state
        .entries
        .iter()
        .map(|entry| {
            let mut name = entry.name.clone();
            if entry.is_dir {
                name.push('/');
            }
            ListItem::new(name)
        })
        .collect();

    let list_title = format!("Current: {}", tab_state.current_dir.display());
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(list_title))
        .highlight_symbol(">> ");

    let mut list_state = ListState::default();
    list_state.select(Some(tab_state.cursor));

    frame.render_stateful_widget(list, area, &mut list_state);
}

fn render_info_footer(frame: &mut Frame, area: Rect, app_state: &AppState) {
    let tasks = app_state.task_manager.get_tasks();
    let task_items: Vec<ListItem> = tasks
        .iter()
        .map(|task| ListItem::new(task.description.clone()))
        .collect();

    let task_list = List::new(task_items)
        .block(Block::default().borders(Borders::ALL).title("Tasks"));

    frame.render_widget(task_list, area);
}

fn render_right_pane(frame: &mut Frame, area: Rect, tab_state: &TabState) {
    let preview_state = tab_state.preview_state.lock().unwrap();
    let content = match &*preview_state {
        PreviewState::Empty => "Empty",
        PreviewState::Loading => "Loading...",
        PreviewState::Text(text) => text,
        PreviewState::Error(e) => e,
    };

    let block = Block::default().borders(Borders::ALL).title("Preview");
    let paragraph = Paragraph::new(content).block(block);

    frame.render_widget(paragraph, area);
}
