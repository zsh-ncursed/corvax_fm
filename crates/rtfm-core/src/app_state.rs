use std::fs;
use std::path::PathBuf;
use crate::task_manager::{TaskManager, TaskKind};
use crate::clipboard::{Clipboard, ClipboardMode};
use crate::preview::PreviewState;
use std::sync::{Arc, Mutex};
use io::fs_ops;
use directories::UserDirs;
use config::Config;
use log;
use crate::plugin_manager::PluginManager;
#[cfg(feature = "mounts")]
use proc_mounts::MountIter;
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

#[derive(Debug, Clone)]
pub struct TabState {
    pub id: usize,
    pub current_dir: PathBuf,
    pub entries: Vec<DirEntry>,
    pub cursor: usize,
    pub preview_state: Arc<Mutex<PreviewState>>,
}

impl TabState {
    pub fn new(id: usize, current_dir: PathBuf) -> Self {
        Self {
            id,
            current_dir,
            entries: Vec::new(),
            cursor: 0,
            preview_state: Arc::new(Mutex::new(PreviewState::default())),
        }
    }

    pub fn update_entries(&mut self, show_hidden: bool) {
        self.entries = match fs::read_dir(&self.current_dir) {
            Ok(entries) => entries
                .filter_map(|res| res.ok())
                .filter(|entry| {
                    if show_hidden {
                        true
                    } else {
                        !entry.file_name().to_string_lossy().starts_with('.')
                    }
                })
                .map(|entry| {
                    let path = entry.path();
                    let is_dir = path.is_dir();
                    let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                    DirEntry { name, path, is_dir }
                })
                .collect(),
            Err(e) => {
                log::error!("Failed to read directory {:?}: {}", self.current_dir, e);
                vec![]
            }
        };
        self.entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.cmp(&b.name)));
        self.cursor = 0;
        self.update_preview(show_hidden);
    }

    pub fn move_cursor_down(&mut self, show_hidden: bool) {
        let max = self.entries.len().saturating_sub(1);
        if self.cursor < max {
            self.cursor += 1;
            self.update_preview(show_hidden);
        }
    }

    pub fn move_cursor_up(&mut self, show_hidden: bool) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.update_preview(show_hidden);
        }
    }

    pub fn enter_directory(&mut self, show_hidden: bool) {
        if let Some(entry) = self.entries.get(self.cursor) {
            if entry.is_dir {
                self.current_dir = entry.path.clone();
                self.update_entries(show_hidden);
            }
        }
    }

    pub fn leave_directory(&mut self, show_hidden: bool) {
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.update_entries(show_hidden);
        }
    }

    pub fn get_selected_entry_path(&self) -> Option<PathBuf> {
        self.entries.get(self.cursor).map(|e| e.path.clone())
    }

    pub fn update_preview(&self, show_hidden: bool) {
        let preview_state_clone = Arc::clone(&self.preview_state);

        if let Some(path) = self.get_selected_entry_path() {
            *preview_state_clone.lock().unwrap() = PreviewState::Loading;

            if path.is_dir() {
                tokio::spawn(async move {
                    let entries = match fs::read_dir(&path) {
                        Ok(entries) => entries
                            .filter_map(|res| res.ok())
                            .filter(|entry| {
                                if show_hidden {
                                    true
                                } else {
                                    !entry.file_name().to_string_lossy().starts_with('.')
                                }
                            })
                            .map(|entry| {
                                let path = entry.path();
                                let is_dir = path.is_dir();
                                let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                                DirEntry { name, path, is_dir }
                            })
                            .collect(),
                        Err(e) => {
                            *preview_state_clone.lock().unwrap() = PreviewState::Error(e.to_string());
                            return;
                        }
                    };
                    *preview_state_clone.lock().unwrap() = PreviewState::Directory(entries);
                });
            } else {
                tokio::spawn(async move {
                    let result = fs_ops::load_text_preview(path).await;
                    *preview_state_clone.lock().unwrap() = match result {
                        Ok(text) => PreviewState::Text(text),
                        Err(e) => PreviewState::Error(e),
                    };
                });
            }
        } else {
            *preview_state_clone.lock().unwrap() = PreviewState::Empty;
        }
    }
}


#[derive(Debug, PartialEq, Eq)]
pub enum FocusBlock {
    Xdg,
    Bookmarks,
    Disks,
}

#[derive(Debug)]
pub struct AppState {
    pub tabs: Vec<TabState>,
    pub active_tab_index: usize,
    pub task_manager: TaskManager,
    pub clipboard: Clipboard,
    pub show_terminal: bool,
    pub show_hidden_files: bool,
    pub focus: FocusBlock,
    pub xdg_dirs: Vec<(String, PathBuf)>,
    pub bookmarks: Vec<(String, PathBuf)>,
    #[cfg(feature = "mounts")]
    pub mounts: Vec<proc_mounts::MountInfo>,
    pub config: Config,
    pub plugin_manager: PluginManager,
}

impl AppState {
    pub fn new() -> Self {
        let config = config::load_config().unwrap_or_else(|err| {
            log::error!("Failed to load config: {}", err);
            Config::default()
        });

        let mut xdg_dirs = Vec::new();
        if let Some(user_dirs) = UserDirs::new() {
            if let Some(path) = user_dirs.document_dir() { xdg_dirs.push(("Documents".to_string(), path.to_path_buf())); }
            if let Some(path) = user_dirs.download_dir() { xdg_dirs.push(("Downloads".to_string(), path.to_path_buf())); }
            if let Some(path) = user_dirs.picture_dir() { xdg_dirs.push(("Pictures".to_string(), path.to_path_buf())); }
            if let Some(path) = user_dirs.video_dir() { xdg_dirs.push(("Videos".to_string(), path.to_path_buf())); }
            if let Some(path) = user_dirs.audio_dir() { xdg_dirs.push(("Music".to_string(), path.to_path_buf())); }
            if let Some(path) = user_dirs.desktop_dir() { xdg_dirs.push(("Desktop".to_string(), path.to_path_buf())); }
            xdg_dirs.push(("Home".to_string(), user_dirs.home_dir().to_path_buf()));
        }

        let bookmarks = config.bookmarks.clone().into_iter().collect();

        #[cfg(feature = "mounts")]
        let mounts = {
            const IGNORED_FS_TYPES: &[&str] = &[
                "proc", "sysfs", "devtmpfs", "devpts", "tmpfs", "securityfs",
                "cgroup", "cgroup2", "pstore", "bpf", "efivarfs", "debugfs",
                "hugetlbfs", "mqueue"
            ];
            match MountIter::new() {
                Ok(iter) => iter
                    .filter_map(|res| res.ok())
                    .filter(|mount| !IGNORED_FS_TYPES.contains(&mount.fstype.as_str()))
                    .collect(),
                Err(e) => {
                    log::error!("Failed to get mounts: {}", e);
                    Vec::new()
                }
            }
        };

        let show_hidden_files = false;
        let mut initial_tab = TabState::new(0, std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")));
        initial_tab.update_entries(show_hidden_files);

        Self {
            tabs: vec![initial_tab],
            active_tab_index: 0,
            task_manager: TaskManager::new(),
            clipboard: Clipboard::new(),
            show_terminal: false,
            show_hidden_files,
            focus: FocusBlock::Xdg,
            xdg_dirs,
            bookmarks,
            plugin_manager: PluginManager::new(),
            #[cfg(feature = "mounts")]
            mounts,
            config,
        }
    }

    pub fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            FocusBlock::Xdg => FocusBlock::Bookmarks,
            FocusBlock::Bookmarks => FocusBlock::Disks,
            FocusBlock::Disks => FocusBlock::Xdg,
        };
    }

    pub fn get_active_tab_mut(&mut self) -> &mut TabState {
        &mut self.tabs[self.active_tab_index]
    }

    pub fn get_active_tab(&self) -> &TabState {
        &self.tabs[self.active_tab_index]
    }

    pub fn yank_selection(&mut self) {
        let selected_path = self.get_active_tab().get_selected_entry_path();
        if let Some(path) = selected_path {
            self.clipboard.yank(vec![path]);
        }
    }

    pub fn cut_selection(&mut self) {
        let selected_path = self.get_active_tab().get_selected_entry_path();
        if let Some(path) = selected_path {
            self.clipboard.cut(vec![path]);
        }
    }

    pub fn paste(&mut self) {
        if self.clipboard.paths.is_empty() {
            return;
        }

        let destination = self.get_active_tab().current_dir.clone();
        let mode = self.clipboard.mode.clone().unwrap();

        for src_path in &self.clipboard.paths {
            let dest_path = destination.join(src_path.file_name().unwrap());
            let description = format!("{:?} {:?} -> {:?}", mode, src_path.file_name().unwrap(), destination);
            let task_kind = match mode {
                ClipboardMode::Copy => TaskKind::Copy { src: src_path.clone(), dest: dest_path },
                ClipboardMode::Move => TaskKind::Move { src: src_path.clone(), dest: dest_path },
            };
            self.task_manager.add_task(task_kind, description);
        }

        if mode == ClipboardMode::Move {
            self.clipboard.clear();
        }

        // Refresh the view to show the new files
        let show_hidden = self.show_hidden_files;
        self.get_active_tab_mut().update_entries(show_hidden);
    }

    pub fn next_tab(&mut self) {
        self.active_tab_index = (self.active_tab_index + 1) % self.tabs.len();
    }

    pub fn previous_tab(&mut self) {
        if self.active_tab_index > 0 {
            self.active_tab_index -= 1;
        } else {
            self.active_tab_index = self.tabs.len() - 1;
        }
    }

    pub fn new_tab(&mut self) {
        let new_id = self.tabs.len();
        let current_dir = self.get_active_tab().current_dir.clone();
        let mut new_tab = TabState::new(new_id, current_dir);
        new_tab.update_entries(self.show_hidden_files);
        self.tabs.push(new_tab);
        self.active_tab_index = new_id;
    }

    pub fn toggle_hidden_files(&mut self) {
        self.show_hidden_files = !self.show_hidden_files;
        for tab in &mut self.tabs {
            tab.update_entries(self.show_hidden_files);
        }
    }

    pub fn close_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.active_tab_index);
            if self.active_tab_index >= self.tabs.len() {
                self.active_tab_index = self.tabs.len() - 1;
            }
        }
    }

    pub fn toggle_terminal(&mut self) {
        self.show_terminal = !self.show_terminal;
    }

    pub fn add_bookmark(&mut self) {
        let path = self.get_active_tab().current_dir.clone();
        let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        if !name.is_empty() {
            self.config.bookmarks.insert(name.clone(), path.clone());
            self.bookmarks.push((name, path));
            if let Err(e) = config::save_config(&self.config) {
                log::error!("Failed to save config: {}", e);
            }
        }
    }
}
