use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardMode {
    Copy,
    Move,
}

#[derive(Debug, Clone, Default)]
pub struct Clipboard {
    pub paths: Vec<PathBuf>,
    pub mode: Option<ClipboardMode>,
}

impl Clipboard {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn yank(&mut self, paths: Vec<PathBuf>) {
        self.paths = paths;
        self.mode = Some(ClipboardMode::Copy);
    }

    pub fn cut(&mut self, paths: Vec<PathBuf>) {
        self.paths = paths;
        self.mode = Some(ClipboardMode::Move);
    }

    pub fn clear(&mut self) {
        self.paths.clear();
        self.mode = None;
    }
}
