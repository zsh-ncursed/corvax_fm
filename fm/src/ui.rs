use crate::app::{App, AppMode, Focus, Action, LeftColumnSection};
use crate::settings;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, ListState},
};
use std::path::PathBuf;
use std::fs;
use std::process::Command;
use zip::ZipArchive;
use tar::Archive;
use rascii_art::{render_to, RenderOptions};
use std::fs::File;

fn hex_to_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Color::Reset;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    Color::Rgb(r, g, b)
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let bg_color = hex_to_color(&app.config.colors.bg);
    f.render_widget(Block::default().bg(bg_color), f.area());

    match app.mode {
        AppMode::Normal | AppMode::ConfirmingDelete => {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(20), // Left column
                    Constraint::Percentage(40), // Middle column
                    Constraint::Percentage(40), // Right column
                ].as_ref())
                .split(f.area());

            draw_left_column(f, app, chunks[0]);
            draw_middle_column(f, app, chunks[1]);
            draw_right_column(f, app, chunks[2]);

            if let AppMode::ConfirmingDelete = app.mode {
                let block = Block::default().title("Confirm Delete").borders(Borders::ALL);
                let area = centered_rect(60, 20, f.area());
                let p = Paragraph::new(format!(
                    "Are you sure you want to delete '{}'? (y/n)",
                    app.items[app.middle_col_selected].file_name().map(|s| s.to_string_lossy()).unwrap_or_default()
                ))
                .block(block);
                f.render_widget(Clear, area); //this clears the background
                f.render_widget(p, area);
            }
        }
        AppMode::Settings => {
            settings::draw(f, app);
        }
        AppMode::ImagePreview(_) => {
            // Do nothing, as the image is drawn outside of the TUI
        }
    }
}

fn draw_left_column(f: &mut Frame, app: &mut App, area: Rect) {
    let border_style = if let Focus::Left = app.focus { Style::default().fg(Color::Yellow) } else { Style::default() };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ].as_ref())
        .split(area);

    let home_dirs: Vec<ListItem> = app.home_dirs.iter().map(|d| ListItem::new(d.file_name().map(|s| s.to_string_lossy()).unwrap_or_default())).collect();
    let pinned_dirs: Vec<ListItem> = app.pinned_dirs.iter().map(|d| ListItem::new(d.file_name().map(|s| s.to_string_lossy()).unwrap_or_default())).collect();
    let drives: Vec<ListItem> = app.drives.iter().map(|d| ListItem::new(d.as_str())).collect();

    let highlight_style = Style::default().add_modifier(Modifier::BOLD).bg(Color::Yellow).fg(Color::Black);

    let mut home_list_state = ListState::default();
    if let LeftColumnSection::Home = app.left_col_selected_section {
        home_list_state.select(Some(app.left_col_selected_item));
    }
    let home_list = List::new(home_dirs).block(Block::default().title("Home").borders(Borders::ALL).border_style(border_style)).highlight_style(highlight_style);

    let mut pinned_list_state = ListState::default();
    if let LeftColumnSection::Pinned = app.left_col_selected_section {
        pinned_list_state.select(Some(app.left_col_selected_item));
    }
    let pinned_list = List::new(pinned_dirs).block(Block::default().title("Pinned").borders(Borders::ALL).border_style(border_style)).highlight_style(highlight_style);

    let mut drives_list_state = ListState::default();
    if let LeftColumnSection::Drives = app.left_col_selected_section {
        drives_list_state.select(Some(app.left_col_selected_item));
    }
    let drives_list = List::new(drives).block(Block::default().title("Drives").borders(Borders::ALL).border_style(border_style)).highlight_style(highlight_style);

    f.render_stateful_widget(home_list, chunks[0], &mut home_list_state);
    f.render_stateful_widget(pinned_list, chunks[1], &mut pinned_list_state);
    f.render_stateful_widget(drives_list, chunks[2], &mut drives_list_state);
}

fn draw_middle_column(f: &mut Frame, app: &mut App, area: Rect) {
    let border_style = if let Focus::Middle = app.focus { Style::default().fg(Color::Yellow) } else { Style::default() };
    let items: Vec<ListItem> = app.items.iter().enumerate().map(|(_i, item)| {
        let mut style = Style::default();
        if let Some(action) = &app.action {
            let path_in_action = match action {
                Action::Copy(p) => p,
                Action::Move(p) => p,
            };
            if path_in_action == item {
                style = style.fg(Color::Yellow);
            }
        }
        ListItem::new(item.file_name().map(|s| s.to_string_lossy()).unwrap_or_default()).style(style)
    }).collect();

    let list = List::new(items)
        .block(Block::default().title("Files").borders(Borders::ALL).border_style(border_style))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    let mut list_state = ListState::default();
    list_state.select(Some(app.middle_col_selected));

    f.render_stateful_widget(list, area, &mut list_state);
}

fn draw_right_column(f: &mut Frame, app: &mut App, area: Rect) {
    let border_style = if let Focus::Right = app.focus { Style::default().fg(Color::Yellow) } else { Style::default() };
    let preview_text = if let Some(selected_item) = app.items.get(app.middle_col_selected) {
        if selected_item.is_dir() {
            if let Ok(entries) = fs::read_dir(selected_item) {
                entries.map(|res| res.map(|e| e.path().file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default()).unwrap_or_default())
                       .collect::<Vec<String>>().join("\n")
            } else {
                "Cannot read directory".to_string()
            }
        } else {
            generate_preview(selected_item)
        }
    } else {
        "".to_string()
    };

    let preview = Paragraph::new(preview_text)
        .block(Block::default().title("Preview").borders(Borders::ALL).border_style(border_style));
    f.render_widget(preview, area);
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
        "doc" => {
            let output = Command::new("catdoc").arg(path).output();
            match output {
                Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
                Err(_) => "Could not run catdoc. Is it installed?".to_string(),
            }
        }
        "docx" => {
            let output = Command::new("docx2txt").arg(path).arg("-").output();
            match output {
                Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
                Err(_) => "Could not run docx2txt. Is it installed?".to_string(),
            }
        }
        "xls" | "xlsx" => {
            let output = Command::new("xls2csv").arg(path).output();
            match output {
                Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
                Err(_) => "Could not run xls2csv. Is it installed?".to_string(),
            }
        }
        "ppt" | "pptx" => {
            let output = Command::new("catppt").arg(path).output();
            match output {
                Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
                Err(_) => "Could not run catppt. Is it installed?".to_string(),
            }
        }
        _ => fs::read_to_string(path).unwrap_or_else(|_| "Cannot read file".to_string()),
    }
}
