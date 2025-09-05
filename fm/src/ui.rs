use crate::app::{App, AppMode};
use crate::config;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, ListState},
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let size = f.area();

    let bg_color = config::hex_to_color(&app.config.colors.bg);
    let fg_color = config::hex_to_color(&app.config.colors.fg);
    let highlight_bg_color = config::hex_to_color(&app.config.colors.highlight_bg);
    let highlight_fg_color = config::hex_to_color(&app.config.colors.highlight_fg);

    f.render_widget(Block::default().bg(bg_color), size);

    match app.mode {
        AppMode::Normal => {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(size);

            let items: Vec<ListItem> = app
                .items
                .iter()
                .map(|i| ListItem::new(i.file_name().unwrap().to_str().unwrap()))
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .title("Files")
                        .borders(Borders::ALL)
                        .fg(fg_color),
                )
                .highlight_style(
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .bg(highlight_bg_color)
                        .fg(highlight_fg_color),
                )
                .highlight_symbol("> ");

            let mut list_state = ListState::default();
            list_state.select(Some(app.selected));

            f.render_stateful_widget(list, chunks[0], &mut list_state);

            let preview_text = if let Some(selected_item) = app.items.get(app.selected) {
                generate_preview(selected_item)
            } else {
                "".to_string()
            };

            let preview = Paragraph::new(preview_text)
                .block(Block::default().title("Preview").borders(Borders::ALL).fg(fg_color));
            f.render_widget(preview, chunks[1]);
        }
        AppMode::Drives => {
            let items: Vec<ListItem> = app.drives.iter().map(|i| ListItem::new(i.as_str())).collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .title("Drives")
                        .borders(Borders::ALL)
                        .fg(fg_color),
                )
                .highlight_style(
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .bg(highlight_bg_color)
                        .fg(highlight_fg_color),
                )
                .highlight_symbol("> ");

            let mut list_state = ListState::default();
            list_state.select(Some(app.selected));

            f.render_stateful_widget(list, size, &mut list_state);
        }
    }

    if app.deleting {
        let block = Block::default().title("Confirm Delete").borders(Borders::ALL);
        let area = centered_rect(60, 20, size);
        let p = Paragraph::new(format!(
            "Are you sure you want to delete '{}'? (y/n)",
            app.items[app.selected].file_name().unwrap().to_str().unwrap()
        ))
        .block(block);
        f.render_widget(Clear, area); //this clears the background
        f.render_widget(p, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

use std::path::PathBuf;
use std::fs;
use std::process::Command;
use zip::ZipArchive;
use tar::Archive;
use rascii_art::{render_to, RenderOptions};
use std::fs::File;


fn generate_preview(path: &PathBuf) -> String {
    if path.is_dir() {
        return "Directory".to_string();
    }

    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    match extension {
        "png" | "jpg" | "jpeg" | "gif" => {
            let mut buffer: Vec<u8> = Vec::new();
            let result = render_to(
                path,
                &mut buffer,
                RenderOptions::new().width(50),
            );
            if result.is_ok() {
                String::from_utf8_lossy(&buffer).to_string()
            } else {
                "Cannot preview image".to_string()
            }
        }
        "zip" => {
            if let Ok(file) = File::open(path) {
                if let Ok(mut archive) = ZipArchive::new(file) {
                    let mut content = String::new();
                    for i in 0..archive.len() {
                        if let Ok(file) = archive.by_index(i) {
                            content.push_str(file.name());
                            content.push('\n');
                        }
                    }
                    return content;
                }
            }
            "Cannot read zip file".to_string()
        }
        "tar" => {
            if let Ok(file) = File::open(path) {
                let mut archive = Archive::new(file);
                let mut content = String::new();
                if let Ok(entries) = archive.entries() {
                    for entry in entries {
                        if let Ok(file) = entry {
                            if let Ok(path) = file.path() {
                                content.push_str(&path.to_string_lossy());
                                content.push('\n');
                            }
                        }
                    }
                    return content;
                }
            }
            "Cannot read tar file".to_string()
        }
        "pdf" => {
            let output = Command::new("pdftotext")
                .arg(path)
                .arg("-")
                .output();
            match output {
                Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
                Err(_) => "Could not run pdftotext. Is it installed?".to_string(),
            }
        }
        _ => fs::read_to_string(path).unwrap_or_else(|_| "Cannot read file".to_string()),
    }
}
