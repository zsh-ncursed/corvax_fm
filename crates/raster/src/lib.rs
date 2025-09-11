#[derive(Debug, Clone, Default)]
pub struct PreviewImage {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub pixels: Vec<u8>, // RGBA8
}
