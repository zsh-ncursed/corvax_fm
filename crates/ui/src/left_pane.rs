use ratatui::{
    prelude::{Color, Rect, Frame, Style, Span},
    widgets::{block::Title, Block, Borders, List, ListItem, ListState},
};
use rtfm_core::app_state::{AppState, FocusBlock};

pub fn render_xdg_block(frame: &mut Frame, area: Rect, app_state: &AppState) {
    let items: Vec<ListItem> = app_state
        .xdg_dirs
        .iter()
        .map(|(name, _path)| ListItem::new(name.clone()))
        .collect();

    let is_focused = app_state.focus == FocusBlock::Xdg;
    let title_style = if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };
    let highlight_style = if is_focused { Style::default().bg(Color::Blue) } else { Style::default().bg(Color::DarkGray) };

    let list = List::new(items)
        .block(
            Block::default()
                .title(Title::from(Span::styled("XDG Dirs", title_style)))
                .borders(Borders::BOTTOM)
        )
        .highlight_style(highlight_style);

    let mut list_state = ListState::default();
    list_state.select(Some(app_state.xdg_cursor));

    frame.render_stateful_widget(list, area, &mut list_state);
}

pub fn render_bookmarks_block(frame: &mut Frame, area: Rect, app_state: &AppState) {
    let items: Vec<ListItem> = app_state
        .bookmarks
        .iter()
        .map(|(name, _path)| ListItem::new(name.clone()))
        .collect();

    let is_focused = app_state.focus == FocusBlock::Bookmarks;
    let title_style = if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };
    let highlight_style = if is_focused { Style::default().bg(Color::Blue) } else { Style::default().bg(Color::DarkGray) };

    let list = List::new(items)
        .block(
            Block::default()
                .title(Title::from(Span::styled("Bookmarks", title_style)))
                .borders(Borders::BOTTOM)
        )
        .highlight_style(highlight_style);

    let mut list_state = ListState::default();
    list_state.select(Some(app_state.bookmarks_cursor));

    frame.render_stateful_widget(list, area, &mut list_state);
}

#[cfg(feature = "mounts")]
pub fn render_mounts_block(frame: &mut Frame, area: Rect, app_state: &AppState) {
    let items: Vec<ListItem> = app_state
        .mounts
        .iter()
        .map(|mount| ListItem::new(mount.dest.to_string_lossy().to_string()))
        .collect();

    let is_focused = app_state.focus == FocusBlock::Disks;
    let title_style = if is_focused { Style::default().fg(Color::Yellow) } else { Style::default() };
    let highlight_style = if is_focused { Style::default().bg(Color::Blue) } else { Style::default().bg(Color::DarkGray) };

    let list = List::new(items)
        .block(Block::default().title(Title::from(Span::styled("Mounts", title_style))))
        .highlight_style(highlight_style);

    let mut list_state = ListState::default();
    list_state.select(Some(app_state.disks_cursor));

    frame.render_stateful_widget(list, area, &mut list_state);
}

#[cfg(not(feature = "mounts"))]
pub fn render_mounts_block(frame: &mut Frame, area: Rect, _app_state: &AppState) {
    let block = Block::new().borders(Borders::ALL).title("Mounts (unsupported)");
    frame.render_widget(block, area);
}
