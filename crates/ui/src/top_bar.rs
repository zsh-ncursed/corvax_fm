use ratatui::{
    prelude::{Rect, Style},
    style::{Color, Modifier},
    widgets::{Block, Borders, Tabs},
    Frame,
};

use rtfm_core::app_state::AppState;

pub fn render_top_bar(frame: &mut Frame, area: Rect, app_state: &AppState) {
    let titles: Vec<String> = app_state
        .tabs
        .iter()
        .map(|tab| {
            format!(
                "{} {}",
                tab.id + 1,
                tab.current_dir
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
            )
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::BOTTOM))
        .select(app_state.active_tab_index)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(tabs, area);
}
