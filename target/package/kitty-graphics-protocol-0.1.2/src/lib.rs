//! Kitty Graphics Protocol - A Rust library for the Kitty terminal graphics protocol
//!
//! This library provides a complete implementation of the Kitty terminal graphics protocol,
//! allowing you to display images and animations in terminal emulators that support this protocol.
//!
//! # Features
//!
//! - Full support for all graphics protocol commands
//! - Support for RGB, RGBA, and PNG image formats
//! - Chunked data transmission for large images
//! - Animation support
//! - Unicode placeholder support
//! - Terminal size detection
//! - Protocol support detection
//!
//! # Quick Start
//!
//! ```no_run
//! use kitty_graphics_protocol::display_png;
//!
//! // Display a PNG file
//! display_png("image.png").unwrap();
//! ```
//!
//! # Advanced Usage
//!
//! ```no_run
//! use kitty_graphics_protocol::{Command, ImageFormat, Action};
//!
//! // Create a command to transmit and display a PNG image
//! let png_data = std::fs::read("image.png").unwrap();
//! let cmd = Command::builder()
//!     .action(Action::TransmitAndDisplay)
//!     .format(ImageFormat::Png)
//!     .build();
//!
//! // Serialize the command to escape sequence chunks
//! let chunks: Vec<String> = cmd.serialize_chunked(&png_data).unwrap().collect();
//! for chunk in chunks {
//!     print!("{}", chunk);
//! }
//! ```

pub mod command;
pub mod error;
pub mod image;
pub mod response;
pub mod terminal;
pub mod types;

pub use command::{ChunkedSerializer, Command, CommandBuilder};
pub use error::{Error, Result};
pub use image::{ImageDisplay, clear_all_images, display_png, display_png_data};
pub use response::Response;
pub use terminal::{WindowSize, check_protocol_support, get_window_size, query_window_size};
pub use types::{
    Action, AnimationControl, CompositionMode, Compression, CursorPolicy, DeleteTarget,
    FrameComposition, ImageFormat, TransmissionMedium, UnicodePlaceholder,
};

/// The ESC character (0x1b)
pub const ESC: u8 = 0x1b;

/// The APC (Application Programming Command) start sequence: ESC _
pub const APC_START: &[u8] = &[ESC, b'_'];

/// The APC end sequence: ESC \
pub const APC_END: &[u8] = &[ESC, b'\\'];

/// The graphics command prefix: G
pub const GRAPHICS_PREFIX: &str = "G";

/// Maximum chunk size for data transmission (4096 bytes)
pub const MAX_CHUNK_SIZE: usize = 4096;
