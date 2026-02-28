//! High-level image display utilities

use crate::command::Command;
use crate::error::Result;
use crate::types::{Action, ImageFormat};
use std::io::Write;
use std::path::Path;

/// A high-level interface for displaying images in the terminal
pub struct ImageDisplay {
    quiet: u8,
}

impl Default for ImageDisplay {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageDisplay {
    /// Create a new ImageDisplay instance
    pub fn new() -> Self {
        Self { quiet: 2 }
    }

    /// Set quiet mode (0 = all responses, 1 = suppress OK, 2 = suppress all)
    pub fn quiet(mut self, mode: u8) -> Self {
        self.quiet = mode;
        self
    }

    /// Display a PNG image from file
    pub fn display_png_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let data = std::fs::read(path)?;
        self.display_png(&data)
    }

    /// Display a PNG image from memory
    pub fn display_png(&self, data: &[u8]) -> Result<()> {
        let chunks = Command::transmit_png(data)?;
        let mut stdout = std::io::stdout().lock();
        for chunk in chunks {
            stdout.write_all(chunk.as_bytes())?;
        }
        stdout.flush()?;
        Ok(())
    }

    /// Display raw RGBA data
    pub fn display_rgba(&self, data: &[u8], width: u32, height: u32) -> Result<()> {
        let chunks = Command::transmit_rgba(data, width, height)?;
        let mut stdout = std::io::stdout().lock();
        for chunk in chunks {
            stdout.write_all(chunk.as_bytes())?;
        }
        stdout.flush()?;
        Ok(())
    }

    /// Display raw RGB data
    pub fn display_rgb(&self, data: &[u8], width: u32, height: u32) -> Result<()> {
        let chunks = Command::transmit_rgb(data, width, height)?;
        let mut stdout = std::io::stdout().lock();
        for chunk in chunks {
            stdout.write_all(chunk.as_bytes())?;
        }
        stdout.flush()?;
        Ok(())
    }

    /// Clear all visible images
    pub fn clear_all(&self) -> Result<()> {
        let cmd = Command::delete_all();
        let seq = cmd.serialize(&[])?;
        let mut stdout = std::io::stdout().lock();
        stdout.write_all(seq.as_bytes())?;
        stdout.flush()?;
        Ok(())
    }

    /// Transmit an image without displaying it (returns image ID for later use)
    pub fn transmit_png(&self, data: &[u8], image_id: u32) -> Result<()> {
        let cmd = Command::builder()
            .action(Action::Transmit)
            .format(ImageFormat::Png)
            .image_id(image_id)
            .quiet(self.quiet)
            .build();

        let chunks: Vec<String> = cmd.serialize_chunked(data)?.collect();
        let mut stdout = std::io::stdout().lock();
        for chunk in chunks {
            stdout.write_all(chunk.as_bytes())?;
        }
        stdout.flush()?;
        Ok(())
    }

    /// Place a previously transmitted image
    pub fn place_image(&self, image_id: u32, cols: u32, rows: u32) -> Result<()> {
        let cmd = Command::place(image_id, cols, rows);
        let seq = cmd.serialize(&[])?;
        let mut stdout = std::io::stdout().lock();
        stdout.write_all(seq.as_bytes())?;
        stdout.flush()?;
        Ok(())
    }
}

/// Quick function to display a PNG file
pub fn display_png<P: AsRef<Path>>(path: P) -> Result<()> {
    ImageDisplay::new().display_png_file(path)
}

/// Quick function to display PNG data from memory
pub fn display_png_data(data: &[u8]) -> Result<()> {
    ImageDisplay::new().display_png(data)
}

/// Quick function to clear all visible images
pub fn clear_all_images() -> Result<()> {
    ImageDisplay::new().clear_all()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_display_creation() {
        let display = ImageDisplay::new().quiet(1);
        assert_eq!(display.quiet, 1);
    }
}
