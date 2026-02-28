//! Type definitions for the Kitty graphics protocol

use std::fmt;

/// Image format for pixel data transmission
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
#[derive(Default)]
pub enum ImageFormat {
    /// 24-bit RGB format (3 bytes per pixel)
    Rgb = 24,
    /// 32-bit RGBA format (4 bytes per pixel, default)
    #[default]
    Rgba = 32,
    /// PNG format (compressed image data)
    Png = 100,
}

impl fmt::Display for ImageFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self as u8)
    }
}

/// Transmission medium for image data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TransmissionMedium {
    /// Direct transmission within the escape code itself (default)
    #[default]
    Direct,
    /// Read from a file (regular files only)
    File,
    /// Read from a temporary file (terminal will delete after reading)
    TempFile,
    /// Read from a shared memory object
    SharedMemory,
}

impl fmt::Display for TransmissionMedium {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match self {
            Self::Direct => 'd',
            Self::File => 'f',
            Self::TempFile => 't',
            Self::SharedMemory => 's',
        };
        write!(f, "{c}")
    }
}

/// Action to perform with the graphics command
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Action {
    /// Query support and transmission medium availability
    Query,
    /// Transmit image data only (don't display)
    Transmit,
    /// Transmit and display the image (a=T)
    #[default]
    TransmitAndDisplay,
    /// Display a previously transmitted image (a=p)
    Place,
    /// Delete images (a=d)
    Delete,
    /// Transmit animation frame data (a=f)
    Frame,
    /// Control animation (a=a)
    AnimationControl,
    /// Compose animation frames (a=c)
    ComposeFrame,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Query => "q",
            Self::Transmit => "t",
            Self::TransmitAndDisplay => "T",
            Self::Place => "p",
            Self::Delete => "d",
            Self::Frame => "f",
            Self::AnimationControl => "a",
            Self::ComposeFrame => "c",
        };
        write!(f, "{s}")
    }
}

/// Delete target specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeleteTarget {
    /// Delete all visible placements (a/A)
    All,
    /// Delete all visible placements and free image data (A)
    AllWithFree,
    /// Delete by image ID (i/I)
    ById { free_data: bool },
    /// Delete by image number (n/N)
    ByNumber { free_data: bool },
    /// Delete at cursor position (c/C)
    AtCursor { free_data: bool },
    /// Delete animation frames (f/F)
    Frames { free_data: bool },
    /// Delete at specific cell (p/P)
    AtCell { free_data: bool },
    /// Delete at cell with z-index (q/Q)
    AtCellWithZIndex { free_data: bool },
    /// Delete by ID range (r/R)
    ByIdRange { free_data: bool },
    /// Delete by column (x/X)
    ByColumn { free_data: bool },
    /// Delete by row (y/Y)
    ByRow { free_data: bool },
    /// Delete by z-index (z/Z)
    ByZIndex { free_data: bool },
}

impl DeleteTarget {
    /// Get the character code for this delete target
    pub fn code(&self) -> char {
        match self {
            Self::All => 'a',
            Self::AllWithFree => 'A',
            Self::ById { free_data: false } => 'i',
            Self::ById { free_data: true } => 'I',
            Self::ByNumber { free_data: false } => 'n',
            Self::ByNumber { free_data: true } => 'N',
            Self::AtCursor { free_data: false } => 'c',
            Self::AtCursor { free_data: true } => 'C',
            Self::Frames { free_data: false } => 'f',
            Self::Frames { free_data: true } => 'F',
            Self::AtCell { free_data: false } => 'p',
            Self::AtCell { free_data: true } => 'P',
            Self::AtCellWithZIndex { free_data: false } => 'q',
            Self::AtCellWithZIndex { free_data: true } => 'Q',
            Self::ByIdRange { free_data: false } => 'r',
            Self::ByIdRange { free_data: true } => 'R',
            Self::ByColumn { free_data: false } => 'x',
            Self::ByColumn { free_data: true } => 'X',
            Self::ByRow { free_data: false } => 'y',
            Self::ByRow { free_data: true } => 'Y',
            Self::ByZIndex { free_data: false } => 'z',
            Self::ByZIndex { free_data: true } => 'Z',
        }
    }
}

/// Animation control commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimationControl {
    /// Stop the animation
    Stop,
    /// Run in loading mode (wait for more frames at end)
    Loading,
    /// Run normally (loop at end)
    Run,
}

impl fmt::Display for AnimationControl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let n = match self {
            Self::Stop => 1,
            Self::Loading => 2,
            Self::Run => 3,
        };
        write!(f, "{n}")
    }
}

/// Composition mode for frame operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CompositionMode {
    /// Alpha blend (default)
    #[default]
    AlphaBlend,
    /// Simple pixel replacement
    Replace,
}

impl fmt::Display for CompositionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let n = match self {
            Self::AlphaBlend => 0,
            Self::Replace => 1,
        };
        write!(f, "{n}")
    }
}

/// Frame composition parameters for a=c action
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameComposition {
    /// Source frame number (1-based)
    pub source_frame: u32,
    /// Destination frame number (1-based)
    pub dest_frame: u32,
    /// Rectangle width in pixels
    pub width: Option<u32>,
    /// Rectangle height in pixels
    pub height: Option<u32>,
    /// Source X offset
    pub source_x: Option<u32>,
    /// Source Y offset
    pub source_y: Option<u32>,
    /// Destination X offset
    pub dest_x: Option<u32>,
    /// Destination Y offset
    pub dest_y: Option<u32>,
    /// Composition mode
    pub mode: CompositionMode,
}

impl Default for FrameComposition {
    fn default() -> Self {
        Self {
            source_frame: 1,
            dest_frame: 1,
            width: None,
            height: None,
            source_x: None,
            source_y: None,
            dest_x: None,
            dest_y: None,
            mode: CompositionMode::AlphaBlend,
        }
    }
}

/// Unicode placeholder configuration for virtual placements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnicodePlaceholder {
    /// Number of columns for the placeholder
    pub columns: u16,
    /// Number of rows for the placeholder
    pub rows: u16,
}

/// Compression algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Compression {
    /// ZLIB deflate compression (RFC 1950)
    Zlib,
}

impl fmt::Display for Compression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Zlib => "z",
        };
        write!(f, "{s}")
    }
}

/// Cursor movement policy after placing an image
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CursorPolicy {
    /// Default: move cursor right by columns and down by rows
    #[default]
    Default,
    /// Don't move the cursor
    NoMove,
}

impl fmt::Display for CursorPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let n = match self {
            Self::Default => 0,
            Self::NoMove => 1,
        };
        write!(f, "{n}")
    }
}
