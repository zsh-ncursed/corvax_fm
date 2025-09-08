use crate::app_state::DirEntry;

#[derive(Debug, Clone, Default)]
pub enum PreviewState {
    #[default]
    Empty,
    Loading,
    Text(String),
    Directory(Vec<DirEntry>),
    Error(String),
}
