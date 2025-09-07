use ratatui::{
    prelude::{Rect, Frame},
    widgets::{Block, Borders, List, ListItem},
};
use rtfm_core::app_state::AppState;

pub fn render_xdg_block(frame: &mut Frame, area: Rect, app_state: &AppState) {
    let items: Vec<ListItem> = app_state
        .xdg_dirs
        .iter()
        .map(|(name, _path)| ListItem::new(name.clone()))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("XDG Dirs"));

    frame.render_widget(list, area);
}

pub fn render_bookmarks_block(frame: &mut Frame, area: Rect, app_state: &AppState) {
    let items: Vec<ListItem> = app_state
        .bookmarks
        .iter()
        .map(|(name, _path)| ListItem::new(name.clone()))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Bookmarks"));

    frame.render_widget(list, area);
}

#[cfg(feature = "mounts")]
pub fn render_mounts_block(frame: &mut Frame, area: Rect, app_state: &AppState) {
    let items: Vec<ListItem> = app_state
        .mounts
        .iter()
        .map(|mount| ListItem::new(mount.dest.to_string_lossy().to_string()))
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Mounts"));

    frame.render_widget(list, area);
}

#[cfg(not(feature = "mounts"))]
pub fn render_mounts_block(frame: &mut Frame, area: Rect, _app_state: &AppState) {
    let block = Block::new().borders(Borders::ALL).title("Mounts (unsupported)");
    frame.render_widget(block, area);
}
