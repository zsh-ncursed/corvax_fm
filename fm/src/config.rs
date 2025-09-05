use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub colors: Colors,
    #[serde(default)]
    pub pinned_dirs: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
            pinned_dirs: Vec::new(),
        }
    }
}

pub fn load_config() -> Config {
    dirs::config_dir()
        .map(|config_dir| config_dir.join("fm/config.yml"))
        .and_then(|config_path| std::fs::File::open(config_path).ok())
        .and_then(|f| serde_yaml::from_reader(f).ok())
        .unwrap_or_default()
}

pub fn save_config(config: &Config) -> std::io::Result<()> {
    let config_path = dirs::config_dir().unwrap().join("fm/config.yml");
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let f = std::fs::File::create(config_path)?;
    serde_yaml::to_writer(f, config).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}
