use crate::config::Config;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;

pub enum Action {
    Copy(PathBuf),
    Move(PathBuf),
}

pub enum AppMode {
    Normal,
    Drives,
}

pub struct App {
    pub items: Vec<PathBuf>,
    pub drives: Vec<String>,
    pub selected: usize,
    pub current_dir: PathBuf,
    pub action: Option<Action>,
    pub deleting: bool,
    pub mode: AppMode,
    pub config: Config,
}

impl App {
    pub fn new(config: Config) -> Self {
        let mut app = Self {
            items: Vec::new(),
            drives: Vec::new(),
            selected: 0,
            current_dir: PathBuf::from("."),
            action: None,
            deleting: false,
            mode: AppMode::Normal,
            config,
        };
        app.load_dir().unwrap();
        app
    }

    pub fn load_dir(&mut self) -> io::Result<()> {
        self.items = fs::read_dir(&self.current_dir)?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()?;
        self.selected = 0;
        Ok(())
    }

    pub fn load_drives(&mut self) {
        let output = Command::new("mount").output().expect("failed to execute process");
        let output = String::from_utf8_lossy(&output.stdout);
        self.drives = output
            .lines()
            .map(|line| line.split_whitespace().nth(2).unwrap_or("").to_string())
            .filter(|s| !s.is_empty())
            .collect();
        self.selected = 0;
    }
}
