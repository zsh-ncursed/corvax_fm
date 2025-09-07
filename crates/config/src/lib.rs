use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    #[serde(default)]
    pub keybindings: Keybindings,
    #[serde(default)]
    pub theme: Theme,
    #[serde(default)]
    pub bookmarks: HashMap<String, PathBuf>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Keybindings {
    // Add keybindings here later
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Theme {
    // Add theme settings here later
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "rtfm", "rust-tui-fm") {
        let config_path = proj_dirs.config_dir().join("config.toml");
        if config_path.exists() {
            let config_content = fs::read_to_string(config_path)?;
            let config: Config = toml::from_str(&config_content)?;
            return Ok(config);
        }
    }
    Ok(Config::default())
}

pub fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "rtfm", "rust-tui-fm") {
        let config_path = proj_dirs.config_dir();
        fs::create_dir_all(config_path)?;
        let config_file = config_path.join("config.toml");
        let toml_string = toml::to_string_pretty(config)?;
        fs::write(config_file, toml_string)?;
    }
    Ok(())
}
