use crate::{OutputBackend, PreviewImage};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::{codecs::png::PngEncoder, ColorType, ImageEncoder};
use ratatui::layout::Rect;
use std::io::Write;

/// An `OutputBackend` that uses the Kitty graphics protocol to draw images.
#[derive(Default)]
pub struct KittyBackend {
    last_image_id: Option<u32>,
}

impl KittyBackend {
    pub fn new() -> Self {
        Self::default()
    }
}

impl OutputBackend for KittyBackend {
    fn draw(&mut self, img: &PreviewImage, area: Rect, writer: &mut dyn Write) -> anyhow::Result<()> {
        // First, clear any previously drawn image by this backend.
        self.clear(writer)?;

        // 1. Encode the raw RGBA pixels to PNG format in memory.
        let mut png_buffer = Vec::new();
        let encoder = PngEncoder::new(&mut png_buffer);
        encoder.write_image(&img.pixels, img.width, img.height, ColorType::Rgba8.into())?;

        // 2. Base64 encode the PNG data.
        let b64_data = STANDARD.encode(&png_buffer);

        // 3. Construct the Kitty graphics protocol escape sequence.

        // Move cursor to the top-left of the target area.
        let move_cursor = format!("\x1b[{};{}H", area.y + 1, area.x + 1);

        // Use an ID to allow for specific deletion later.
        let image_id = self.last_image_id.map_or(1, |id| id.wrapping_add(1));
        self.last_image_id = Some(image_id);

        // The Kitty command to transmit and display an image.
        // f=100: PNG format
        // a=T: Transmit and display
        // c, r: columns and rows for scaling
        // i: image ID
        let kitty_cmd = format!(
            "\x1b_Gf=100,a=T,c={},r={},i={};{}\x1b\\",
            area.width, area.height, image_id, b64_data
        );

        write!(writer, "{}{}", move_cursor, kitty_cmd)?;
        writer.flush()?;

        Ok(())
    }

    fn clear(&mut self, writer: &mut dyn Write) -> anyhow::Result<()> {
        if let Some(id) = self.last_image_id.take() {
            // Command to delete an image by its ID.
            let delete_cmd = format!("\x1b_Ga=d,d=i,i={}\x1b\\", id);
            write!(writer, "{}", delete_cmd)?;
            writer.flush()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    #[test]
    fn test_kitty_draw() {
        let mut backend = KittyBackend::new();
        let img = PreviewImage {
            width: 1,
            height: 1,
            stride: 4,
            pixels: vec![255, 0, 0, 255], // A single red pixel
        };
        let area = Rect::new(10, 5, 20, 30);
        let mut writer = Vec::new();

        backend.draw(&img, area, &mut writer).unwrap();

        let output = String::from_utf8(writer).unwrap();

        // 1. Check for cursor move
        assert!(output.contains("\x1b[6;11H"));

        // 2. Check for kitty protocol start
        assert!(output.contains("\x1b_Gf=100,a=T,c=20,r=30,i=1;"));

        // 3. Check for protocol end
        assert!(output.ends_with("\x1b\\"));

        // 4. Check clear
        let mut writer = Vec::new();
        backend.clear(&mut writer).unwrap();
        let output = String::from_utf8(writer).unwrap();
        assert_eq!(output, "\x1b_Ga=d,d=i,i=1\x1b\\");
    }
}
