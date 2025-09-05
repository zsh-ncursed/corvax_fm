use crate::config::Config;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone)]
pub enum Action {
    Copy(PathBuf),
    Move(PathBuf),
}

#[derive(Clone)]
pub enum AppMode {
    Normal,
    Settings,
    ImagePreview(PathBuf),
    ConfirmingDelete,
}

pub enum Focus {
    Left,
    Middle,
    Right,
}

pub enum LeftColumnSection {
    Home,
    Pinned,
    Drives,
}

pub struct App {
    // Middle column
    pub items: Vec<PathBuf>,
    pub middle_col_selected: usize,
    pub current_dir: PathBuf,

    // Left column
    pub home_dirs: Vec<PathBuf>,
    pub pinned_dirs: Vec<PathBuf>,
    pub drives: Vec<String>,
    pub left_col_selected_section: LeftColumnSection,
    pub left_col_selected_item: usize,

    // General
    pub action: Option<Action>,
    pub config: Config,
    pub focus: Focus,
    pub mode: AppMode,
    pub running: bool,
    pub settings_selected: usize,
}

impl App {
    pub fn new(config: Config) -> Self {
        let mut home_dirs = Vec::new();
        if let Some(home) = dirs::home_dir() {
            home_dirs.push(home.clone());
            home_dirs.push(home.join("Downloads"));
            home_dirs.push(home.join("Documents"));
            home_dirs.push(home.join("Pictures"));
            home_dirs.push(home.join("Videos"));
            home_dirs.push(home.join("Music"));
        }

        let mut app = Self {
            items: Vec::new(),
            middle_col_selected: 0,
            current_dir: dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")),
            home_dirs,
            pinned_dirs: config.pinned_dirs.iter().map(PathBuf::from).collect(),
            drives: Vec::new(),
            left_col_selected_section: LeftColumnSection::Home,
            left_col_selected_item: 0,
            action: None,
            config,
            focus: Focus::Middle,
            mode: AppMode::Normal,
            running: true,
            settings_selected: 0,
        };
        app.load_dir().unwrap();
        app.load_drives();
        app
    }

    pub fn load_dir(&mut self) -> io::Result<()> {
        self.items = fs::read_dir(&self.current_dir)?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()?;
        self.middle_col_selected = 0;
        Ok(())
    }

    pub fn load_drives(&mut self) {
        let output = Command::new("mount").output().expect("failed to execute process");
        let output = String::from_utf8_lossy(&output.stdout);
        self.drives = output
            .lines()
            .map(|line| line.split_whitespace().nth(2).unwrap_or("").to_string())
            .filter(|s| !s.is_empty() && s.starts_with('/'))
            .collect();
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn left_col_down(&mut self) {
        match self.left_col_selected_section {
            LeftColumnSection::Home => {
                if self.left_col_selected_item < self.home_dirs.len() - 1 {
                    self.left_col_selected_item += 1;
                } else if !self.pinned_dirs.is_empty() {
                    self.left_col_selected_section = LeftColumnSection::Pinned;
                    self.left_col_selected_item = 0;
                } else if !self.drives.is_empty() {
                    self.left_col_selected_section = LeftColumnSection::Drives;
                    self.left_col_selected_item = 0;
                }
            }
            LeftColumnSection::Pinned => {
                if self.left_col_selected_item < self.pinned_dirs.len() - 1 {
                    self.left_col_selected_item += 1;
                } else if !self.drives.is_empty() {
                    self.left_col_selected_section = LeftColumnSection::Drives;
                    self.left_col_selected_item = 0;
                }
            }
            LeftColumnSection::Drives => {
                if self.left_col_selected_item < self.drives.len() - 1 {
                    self.left_col_selected_item += 1;
                }
            }
        }
    }

    pub fn left_col_up(&mut self) {
        match self.left_col_selected_section {
            LeftColumnSection::Home => {
                if self.left_col_selected_item > 0 {
                    self.left_col_selected_item -= 1;
                }
            }
            LeftColumnSection::Pinned => {
                if self.left_col_selected_item > 0 {
                    self.left_col_selected_item -= 1;
                } else {
                    self.left_col_selected_section = LeftColumnSection::Home;
                    self.left_col_selected_item = self.home_dirs.len() - 1;
                }
            }
            LeftColumnSection::Drives => {
                if self.left_col_selected_item > 0 {
                    self.left_col_selected_item -= 1;
                } else if !self.pinned_dirs.is_empty() {
                    self.left_col_selected_section = LeftColumnSection::Pinned;
                    self.left_col_selected_item = self.pinned_dirs.len() - 1;
                } else {
                    self.left_col_selected_section = LeftColumnSection::Home;
                    self.left_col_selected_item = self.home_dirs.len() - 1;
                }
            }
        }
    }
}
