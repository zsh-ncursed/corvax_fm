use serde::{Deserialize, Serialize};
use ratatui::style::Color;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub colors: Colors,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Colors {
    pub bg: String,
    pub fg: String,
    pub highlight_bg: String,
    pub highlight_fg: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            colors: Colors {
                bg: "#000000".to_string(),
                fg: "#ffffff".to_string(),
                highlight_bg: "#0000ff".to_string(),
                highlight_fg: "#ffff00".to_string(),
            },
        }
    }
}

pub fn hex_to_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Color::Reset;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    Color::Rgb(r, g, b)
}

pub fn load_config() -> Config {
    dirs::config_dir()
        .map(|config_dir| config_dir.join("fm/config.yml"))
        .and_then(|config_path| std::fs::File::open(config_path).ok())
        .and_then(|f| serde_yaml::from_reader(f).ok())
        .unwrap_or_default()
}
