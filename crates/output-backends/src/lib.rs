pub mod kitty;

use raster::PreviewImage;
use ratatui::layout::Rect;
use std::io::Write;

pub trait OutputBackend {
    /// Draws an image to the terminal in the specified area.
    fn draw(&mut self, img: &PreviewImage, area: Rect, writer: &mut dyn Write) -> anyhow::Result<()>;

    /// Clears any image previously drawn by this backend.
    fn clear(&mut self, writer: &mut dyn Write) -> anyhow::Result<()>;
}
