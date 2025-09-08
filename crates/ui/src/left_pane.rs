use ratatui::{
    prelude::{Color, Rect, Frame, Style, Span},
    widgets::{block::Title, Block, List, ListItem},
};
use rtfm_core::app_state::{AppState, FocusBlock};

pub fn render_xdg_block(frame: &mut Frame, area: Rect, app_state: &AppState) {
    let items: Vec<ListItem> = app_state
        .xdg_dirs
        .iter()
        .map(|(name, _path)| ListItem::new(name.clone()))
        .collect();

    let is_focused = app_state.focus == FocusBlock::Xdg;
    let title_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let list = List::new(items)
        .block(Block::default().title(Title::from(Span::styled("XDG Dirs", title_style))));

    frame.render_widget(list, area);
}

pub fn render_bookmarks_block(frame: &mut Frame, area: Rect, app_state: &AppState) {
    let items: Vec<ListItem> = app_state
        .bookmarks
        .iter()
        .map(|(name, _path)| ListItem::new(name.clone()))
        .collect();

    let is_focused = app_state.focus == FocusBlock::Bookmarks;
    let title_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let list = List::new(items)
        .block(Block::default().title(Title::from(Span::styled("Bookmarks", title_style))));

    frame.render_widget(list, area);
}

#[cfg(feature = "mounts")]
pub fn render_mounts_block(frame: &mut Frame, area: Rect, app_state: &AppState) {
    let items: Vec<ListItem> = app_state
        .mounts
        .iter()
        .map(|mount| ListItem::new(mount.dest.to_string_lossy().to_string()))
        .collect();

    let is_focused = app_state.focus == FocusBlock::Disks;
    let title_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let list = List::new(items)
        .block(Block::default().title(Title::from(Span::styled("Mounts", title_style))));

    frame.render_widget(list, area);
}

#[cfg(not(feature = "mounts"))]
pub fn render_mounts_block(frame: &mut Frame, area: Rect, _app_state: &AppState) {
    let block = Block::new().title("Mounts (unsupported)");
    frame.render_widget(block, area);
}
