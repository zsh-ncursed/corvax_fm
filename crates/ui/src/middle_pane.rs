use crate::right_pane;
use ratatui::{
    prelude::{Color, Rect, Style},
    widgets::{List, ListItem, ListState},
    Frame,
};
use rtfm_core::app_state::TabState;

pub fn render_middle_pane(frame: &mut Frame, area: Rect, tab_state: &TabState) {
    let items: Vec<ListItem> = tab_state
        .entries
        .iter()
        .map(|entry| {
            let is_hidden = entry.name.starts_with('.');
            let style = if is_hidden {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            };

            let icon = right_pane::get_icon_for_path(&entry.path, entry.is_dir);
            let mut name = format!("{} {}", icon, entry.name.clone());
            if entry.is_dir {
                name.push('/');
            }
            ListItem::new(name).style(style)
        })
        .collect();

    let list = List::new(items).highlight_symbol(">> ");

    let mut list_state = ListState::default();
    list_state.select(Some(tab_state.cursor));

    frame.render_stateful_widget(list, area, &mut list_state);
}
