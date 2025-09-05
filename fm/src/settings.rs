use crate::app::App;
use crate::config::Colors;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, ListState},
};

#[derive(Clone, Copy)]
pub struct StaticColors {
    pub bg: &'static str,
    pub fg: &'static str,
    pub highlight_bg: &'static str,
    pub highlight_fg: &'static str,
}

impl StaticColors {
    pub fn to_colors(&self) -> Colors {
        Colors {
            bg: self.bg.to_string(),
            fg: self.fg.to_string(),
            highlight_bg: self.highlight_bg.to_string(),
            highlight_fg: self.highlight_fg.to_string(),
        }
    }
}

pub struct Theme {
    pub name: &'static str,
    pub colors: StaticColors,
}

pub const THEMES: &[Theme] = &[
    Theme { name: "Default", colors: StaticColors { bg: "#000000", fg: "#ffffff", highlight_bg: "#0000ff", highlight_fg: "#ffff00" } },
    Theme { name: "Dracula", colors: StaticColors { bg: "#282a36", fg: "#f8f8f2", highlight_bg: "#44475a", highlight_fg: "#bd93f9" } },
    Theme { name: "Solarized Light", colors: StaticColors { bg: "#fdf6e3", fg: "#657b83", highlight_bg: "#eee8d5", highlight_fg: "#b58900" } },
    Theme { name: "Solarized Dark", colors: StaticColors { bg: "#002b36", fg: "#839496", highlight_bg: "#073642", highlight_fg: "#b58900" } },
    Theme { name: "Nord", colors: StaticColors { bg: "#2e3440", fg: "#d8dee9", highlight_bg: "#4c566a", highlight_fg: "#88c0d0" } },
    Theme { name: "Gruvbox", colors: StaticColors { bg: "#282828", fg: "#ebdbb2", highlight_bg: "#504945", highlight_fg: "#fe8019" } },
    Theme { name: "Monokai", colors: StaticColors { bg: "#272822", fg: "#f8f8f2", highlight_bg: "#75715e", highlight_fg: "#a6e22e" } },
    Theme { name: "Tomorrow Night", colors: StaticColors { bg: "#1d1f21", fg: "#c5c8c6", highlight_bg: "#373b41", highlight_fg: "#b294bb" } },
    Theme { name: "Oceanic Next", colors: StaticColors { bg: "#1b2b34", fg: "#c0c5ce", highlight_bg: "#343d46", highlight_fg: "#6699cc" } },
    Theme { name: "Ayu Mirage", colors: StaticColors { bg: "#1f2430", fg: "#cbccc6", highlight_bg: "#2b3040", highlight_fg: "#ffb454" } },
    Theme { name: "Catppuccin", colors: StaticColors { bg: "#1e1e2e", fg: "#cdd6f4", highlight_bg: "#45475a", highlight_fg: "#cba6f7" } },
    Theme { name: "One Dark", colors: StaticColors { bg: "#282c34", fg: "#abb2bf", highlight_bg: "#3e4451", highlight_fg: "#61afef" } },
];

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.area());

    let title = Paragraph::new("Settings").alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    let items: Vec<ListItem> = THEMES.iter().map(|t| ListItem::new(t.name)).collect();
    let list = List::new(items)
        .block(Block::default().title("Color Schemes").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    let mut list_state = ListState::default();
    list_state.select(Some(app.settings_selected));

    f.render_stateful_widget(list, chunks[1], &mut list_state);
}
