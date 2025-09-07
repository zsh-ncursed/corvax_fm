use std::path::PathBuf;
use std::process::Child;
use directories::ProjectDirs;

#[derive(Debug)]
pub struct Plugin {
    pub path: PathBuf,
    pub process: Option<Child>,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Default)]
pub struct PluginManager {
    plugins: Vec<Plugin>,
    plugin_dir: PathBuf,
}

impl PluginManager {
    pub fn new() -> Self {
        let plugin_dir = if let Some(proj_dirs) = ProjectDirs::from("com", "rtfm", "rust-tui-fm") {
            proj_dirs.config_dir().join("plugins")
        } else {
            PathBuf::from("plugins")
        };

        let mut manager = Self {
            plugins: Vec::new(),
            plugin_dir,
        };
        manager.discover_plugins();
        manager
    }

    fn discover_plugins(&mut self) {
        // Placeholder for discovery logic
        log::info!("Discovering plugins in {:?}", self.plugin_dir);
    }
}
