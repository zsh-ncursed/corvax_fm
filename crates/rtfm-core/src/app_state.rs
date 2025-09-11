use std::fs;
use std::path::PathBuf;
use crate::task_manager::{TaskManager, TaskKind};
use crate::clipboard::{Clipboard, ClipboardMode};
use crate::preview::PreviewState;
use std::sync::{Arc, Mutex};
use directories::UserDirs;
use config::Config;
use log;
use crate::plugin_manager::PluginManager;
#[cfg(feature = "mounts")]
use proc_mounts::MountIter;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FocusBlock {
    Xdg,
    Bookmarks,
    Disks,
    Middle,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InputMode {
    Normal,
    Create,
}

#[derive(Debug, Clone)]
pub enum CreateFileType {
    File,
    Directory,
}
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
    pub fn new(id: usize) -> Self {
        Self {
            id,
            current_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            entries: Vec::new(),
            cursor: 0,
            preview_state: Arc::new(Mutex::new(PreviewState::default())),
        }
    }

    pub fn set_current_dir(&mut self, new_path: PathBuf, show_hidden: bool) {
        self.current_dir = new_path;
        self.update_entries(show_hidden);
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

    pub fn update_preview(&self, _show_hidden: bool) {
        // This method is now a no-op. Preview is handled by the PreviewController.
        // We could set the state to Loading here, but it's better to do it
        // right before we send the request to the controller.
    }
}


#[derive(Debug)]
pub struct AppState {
    pub tabs: Vec<TabState>,
    pub active_tab_index: usize,
    pub show_tabs: bool,
    pub task_manager: TaskManager,
    pub clipboard: Clipboard,
    pub show_terminal: bool,
    pub show_hidden_files: bool, // Re-add this
    pub focus: FocusBlock,
    pub xdg_dirs: Vec<(String, PathBuf)>,
    pub xdg_cursor: usize,
    pub bookmarks: Vec<(String, PathBuf)>,
    pub bookmarks_cursor: usize,
    #[cfg(feature = "mounts")]
    pub mounts: Vec<proc_mounts::MountInfo>,
    #[cfg(feature = "mounts")]
    pub disks_cursor: usize,
    pub config: Config,
    pub plugin_manager: PluginManager,
    pub info_panel_content: Arc<Mutex<Option<String>>>,
    pub show_confirmation: bool,
    pub confirmation_message: String,
    pub path_to_delete: Option<PathBuf>,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub show_input_dialog: bool,
    pub create_file_type: Option<CreateFileType>,
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
        let mut initial_tab = TabState::new(0);
        initial_tab.update_entries(show_hidden_files);

        Self {
            tabs: vec![initial_tab],
            active_tab_index: 0,
            show_tabs: false, // Hidden by default with one tab
            task_manager: TaskManager::new(),
            clipboard: Clipboard::new(),
            show_terminal: false,
            show_hidden_files,
            focus: FocusBlock::Middle,
            xdg_dirs,
            xdg_cursor: 0,
            bookmarks,
            bookmarks_cursor: 0,
            plugin_manager: PluginManager::new(),
            #[cfg(feature = "mounts")]
            mounts,
            #[cfg(feature = "mounts")]
            disks_cursor: 0,
            config,
            info_panel_content: Arc::new(Mutex::new(None)),
            show_confirmation: false,
            confirmation_message: String::new(),
            path_to_delete: None,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            show_input_dialog: false,
            create_file_type: None,
        }
    }

    pub fn toggle_tabs(&mut self) {
        self.show_tabs = !self.show_tabs;
    }

    pub fn get_active_tab_mut(&mut self) -> &mut TabState {
        &mut self.tabs[self.active_tab_index]
    }

    pub fn get_active_tab(&self) -> &TabState {
        &self.tabs[self.active_tab_index]
    }

    pub fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            FocusBlock::Xdg => FocusBlock::Bookmarks,
            FocusBlock::Bookmarks => FocusBlock::Disks,
            FocusBlock::Disks => FocusBlock::Middle,
            FocusBlock::Middle => FocusBlock::Xdg,
        };
    }

    pub fn move_left_pane_cursor_down(&mut self) {
        match self.focus {
            FocusBlock::Xdg => {
                let max = self.xdg_dirs.len().saturating_sub(1);
                if self.xdg_cursor < max { self.xdg_cursor += 1; }
            },
            FocusBlock::Bookmarks => {
                let max = self.bookmarks.len().saturating_sub(1);
                if self.bookmarks_cursor < max { self.bookmarks_cursor += 1; }
            },
            FocusBlock::Disks => {
                #[cfg(feature = "mounts")]
                {
                    let max = self.mounts.len().saturating_sub(1);
                    if self.disks_cursor < max { self.disks_cursor += 1; }
                }
            },
            FocusBlock::Middle => {}, // Should not happen
        }
        self.update_middle_pane_from_left_pane_selection();
    }

    pub fn move_left_pane_cursor_up(&mut self) {
        match self.focus {
            FocusBlock::Xdg => {
                if self.xdg_cursor > 0 { self.xdg_cursor -= 1; }
            },
            FocusBlock::Bookmarks => {
                if self.bookmarks_cursor > 0 { self.bookmarks_cursor -= 1; }
            },
            FocusBlock::Disks => {
                #[cfg(feature = "mounts")]
                {
                    if self.disks_cursor > 0 { self.disks_cursor -= 1; }
                }
            },
            FocusBlock::Middle => {}, // Should not happen
        }
        self.update_middle_pane_from_left_pane_selection();
    }

    pub fn update_middle_pane_from_left_pane_selection(&mut self) {
        let path = match self.focus {
            FocusBlock::Xdg => self.xdg_dirs.get(self.xdg_cursor).map(|(_, path)| path.clone()),
            FocusBlock::Bookmarks => self.bookmarks.get(self.bookmarks_cursor).map(|(_, path)| path.clone()),
            FocusBlock::Disks => {
                #[cfg(feature = "mounts")]
                {
                    self.mounts.get(self.disks_cursor).map(|mount| mount.dest.clone())
                }
                #[cfg(not(feature = "mounts"))]
                {
                    None
                }
            },
            FocusBlock::Middle => None, // No-op
        };

        if let Some(path) = path {
            let show_hidden = self.show_hidden_files;
            self.get_active_tab_mut().set_current_dir(path, show_hidden);
        }
    }

    pub fn toggle_hidden_files(&mut self) {
        self.show_hidden_files = !self.show_hidden_files;
        for tab in &mut self.tabs {
            tab.update_entries(self.show_hidden_files);
        }
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
        if self.tabs.len() >= 10 {
            return;
        }
        log::info!("new_tab called. Current tab count: {}", self.tabs.len());
        let new_id = self.tabs.len();
        let mut new_tab = TabState::new(new_id);
        new_tab.update_entries(self.show_hidden_files);
        self.tabs.push(new_tab);
        self.active_tab_index = new_id;
        self.show_tabs = true; // Show tabs when a new one is created
        log::info!("new_tab finished. New tab count: {}. Active index: {}", self.tabs.len(), self.active_tab_index);
    }

    pub fn close_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.active_tab_index);
            if self.active_tab_index >= self.tabs.len() {
                self.active_tab_index = self.tabs.len() - 1;
            }
            if self.tabs.len() == 1 {
                self.show_tabs = false; // Hide tabs when only one is left
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

    pub fn clear_info_panel(&mut self) {
        *self.info_panel_content.lock().unwrap() = None;
    }

    pub fn update_info_panel(&mut self) {
        if let Some(path) = self.get_active_tab().get_selected_entry_path() {
            let info_panel_content_clone = Arc::clone(&self.info_panel_content);
            *info_panel_content_clone.lock().unwrap() = Some("Calculating...".to_string());

            tokio::spawn(async move {
                let mut info_text = vec![];
                if let Ok(metadata) = fs::metadata(&path) {
                    if let Ok(created) = metadata.created() {
                        let datetime: chrono::DateTime<chrono::Local> = created.into();
                        info_text.push(format!("Created: {}", datetime.format("%Y-%m-%d %H:%M:%S")));
                    }

                    let size = if metadata.is_dir() {
                        fs_extra::dir::get_size(&path).unwrap_or(0)
                    } else {
                        metadata.len()
                    };
                    info_text.push(format!("Size: {} bytes", size));

                    let final_info = info_text.join("\n");
                    *info_panel_content_clone.lock().unwrap() = Some(final_info);
                } else {
                    *info_panel_content_clone.lock().unwrap() = Some("Failed to get info".to_string());
                }
            });
        }
    }

    pub fn delete_selection(&mut self) {
        if let Some(path) = self.get_active_tab().get_selected_entry_path() {
            self.path_to_delete = Some(path.clone());
            self.confirmation_message = format!("Are you sure you want to delete {:?}? (y/n)", path.file_name().unwrap());
            self.show_confirmation = true;
        }
    }

    pub fn confirm_delete(&mut self) {
        if let Some(path) = self.path_to_delete.take() {
            let description = format!("Delete {:?}", path.file_name().unwrap());
            let task_kind = TaskKind::Delete { path };
            self.task_manager.add_task(task_kind, description);
        }
        self.show_confirmation = false;
        self.path_to_delete = None;
    }

    pub fn cancel_delete(&mut self) {
        self.show_confirmation = false;
        self.path_to_delete = None;
    }

    pub fn create_item(&mut self) {
        if self.input_buffer.is_empty() {
            return;
        }

        let new_item_name = self.input_buffer.clone();
        self.input_buffer.clear();

        let current_dir = self.get_active_tab().current_dir.clone();
        let new_item_path = current_dir.join(new_item_name);

        let task_kind = match self.create_file_type.as_ref().unwrap() {
            CreateFileType::File => TaskKind::CreateFile {
                path: new_item_path.clone(),
            },
            CreateFileType::Directory => TaskKind::CreateDirectory {
                path: new_item_path.clone(),
            },
        };

        let description = format!("Create {:?}", new_item_path);
        self.task_manager.add_task(task_kind, description);

        self.create_file_type = None;
    }
}
