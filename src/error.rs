//! Error types for the Kitty graphics protocol

use thiserror::Error;

/// Result type alias for Kitty graphics protocol operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for Kitty graphics protocol operations
#[derive(Error, Debug)]
pub enum Error {
    /// Base64 decoding error
    #[error("Base64 decoding error: {0}")]
    Base64Decode(#[from] base64::DecodeError),

    /// Invalid image dimensions
    #[error("Invalid image dimensions: width={width}, height={height}")]
    InvalidDimensions {
        width: u32,
        height: u32,
    },

    /// Invalid image ID
    #[error("Invalid image ID: {0}")]
    InvalidImageId(u32),

    /// Invalid placement ID
    #[error("Invalid placement ID: {0}")]
    InvalidPlacementId(u32),

    /// Invalid chunk size
    #[error("Invalid chunk size: {0} (must be multiple of 4, max 4096)")]
    InvalidChunkSize(usize),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(&'static str),

    /// Terminal response error
    #[error("Terminal error: {0}")]
    TerminalError(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// UTF-8 error (FromUtf8Error)
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    /// UTF-8 error (Utf8Error)
    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),

    /// Invalid response from terminal
    #[error("Invalid response from terminal: {0}")]
    InvalidResponse(String),

    /// Protocol error
    #[error("Protocol error: {0}")]
    Protocol(String),
}

impl Error {
    /// Create a new protocol error
    pub fn protocol(msg: impl Into<String>) -> Self {
        Self::Protocol(msg.into())
    }

    /// Create a new terminal error
    pub fn terminal(msg: impl Into<String>) -> Self {
        Self::TerminalError(msg.into())
    }
}
