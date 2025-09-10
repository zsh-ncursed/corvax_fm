use ratatui::{
    prelude::{Color, Rect, Style},
    widgets::{List, ListItem, Paragraph},
    Frame,
};
use rtfm_core::{app_state::TabState, preview::PreviewState};
use std::path::Path;

pub fn get_icon_for_path(path: &Path, is_dir: bool) -> &'static str {
    if is_dir {
        return ""; // Folder icon
    }
    match path.extension().and_then(|s| s.to_str()) {
        Some("jpg") | Some("jpeg") | Some("png") | Some("gif") | Some("bmp") => "", // Image icon
        Some("zip") | Some("gz") | Some("tar") | Some("rar") | Some("7z") => "",    // Archive icon
        Some("txt") | Some("md") => "", // Text file icon
        Some("mp3") | Some("wav") | Some("flac") => "🎵", // Audio icon
        Some("mp4") | Some("mkv") | Some("mov") | Some("avi") => "", // Video icon
        Some("rs") | Some("js") | Some("html") | Some("css") | Some("py") | Some("toml")
        | Some("sh") => "", // Code icon
        _ => "", // Generic file icon
    }
}

pub fn render_right_pane(frame: &mut Frame, area: Rect, tab_state: &TabState) {
    let preview_state = tab_state.preview_state.lock().unwrap();

    match &*preview_state {
        PreviewState::Directory(entries) => {
            let items: Vec<ListItem> = entries
                .iter()
                .map(|entry| {
                    let is_hidden = entry.name.starts_with('.');
                    let style = if is_hidden {
                        Style::default().fg(Color::DarkGray)
                    } else {
                        Style::default()
                    };

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
