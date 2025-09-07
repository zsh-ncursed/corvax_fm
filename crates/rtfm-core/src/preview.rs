#[derive(Debug, Clone, Default)]
pub enum PreviewState {
    #[default]
    Empty,
    Loading,
    Text(String),
    Error(String),
}
