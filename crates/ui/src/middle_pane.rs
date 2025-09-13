use ratatui::{
    prelude::{Color, Rect, Style},
    widgets::{List, ListItem, ListState},
    Frame,
};
use rtfm_core::app_state::TabState;

fn get_icon_for_file(name: &str, is_dir: bool) -> &'static str {
    if is_dir {
        return ""; // Folder icon
    }
    match name.split('.').last() {
        Some("rs") => "",   // Rust
        Some("js") => "",   // JavaScript
        Some("html") => "", // HTML
        Some("css") => "",  // CSS
        Some("json") => "", // JSON
        Some("md") => "",   // Markdown
        Some("toml") => "", // TOML
        Some("lock") => "", // Lock
        Some("git") | Some("gitignore") => "", // Git
        // Audio
        Some("mp3") | Some("wav") | Some("flac") => "🎵",
        // Video
        Some("mp4") | Some("avi") | Some("mkv") | Some("mov") => "🎞",
        Some("zip") | Some("rar") | Some("7z") | Some("tar") | Some("gz") => "", // Archive
        Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("webp") | Some("ico") => "", // Image
        Some("pdf") => "",   // PDF
        Some("txt") => "",   // Text file
        _ => "",           // Default file
    }
}

pub fn render_middle_pane(frame: &mut Frame, area: Rect, tab_state: &TabState) {
    let items: Vec<ListItem> = tab_state
        .entries
        .iter()
        .map(|entry| {
            let is_hidden = entry.name.starts_with('.');
            let style = if is_hidden {
                Style::default().fg(Color::Gray)
            } else {
                Style::default()
            };

            let icon = get_icon_for_file(&entry.name, entry.is_dir);
            let mut name = format!("{} {}", icon, entry.name);
            if entry.is_dir {
                name.push('/');
            }
            ListItem::new(name).style(style)
        })
        .collect();

    let list = List::new(items).highlight_style(Style::default().bg(Color::Rgb(50, 50, 80)));

    let mut list_state = ListState::default();
    list_state.select(Some(tab_state.cursor));

    frame.render_stateful_widget(list, area, &mut list_state);
}
