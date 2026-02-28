//! Response parsing for the Kitty graphics protocol

use crate::error::{Error, Result};

/// Response from the terminal
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    /// Image ID (if applicable)
    pub image_id: Option<u32>,
    /// Image number (if applicable)
    pub image_number: Option<u32>,
    /// Placement ID (if applicable)
    pub placement_id: Option<u32>,
    /// Whether the operation was successful
    pub success: bool,
    /// Error message (if failed)
    pub error: Option<String>,
}

impl Response {
    /// Parse a response from the terminal
    pub fn parse(data: &[u8]) -> Result<Self> {
        // Expected format: <ESC>_Gi=<id>;OK<ESC>\ or <ESC>_Gi=<id>;ERROR:message<ESC>\
        // Also: <ESC>_Gi=<id>,p=<placement_id>;OK<ESC>\
        // And: <ESC>_Gi=<id>,I=<number>;OK<ESC>\

        // Check for APC start
        if data.len() < 6 {
            return Err(Error::InvalidResponse(String::from_utf8_lossy(data).into_owned()));
        }

        if data[0] != crate::ESC || data[1] != b'_' || data[2] != b'G' {
            return Err(Error::InvalidResponse(String::from_utf8_lossy(data).into_owned()));
        }

        // Find the semicolon separator
        let semicolon_pos = data.iter().position(|&b| b == b';').ok_or_else(|| {
            Error::InvalidResponse(String::from_utf8_lossy(data).into_owned())
        })?;

        // Parse control data (between G and ;)
        let control = &data[3..semicolon_pos];
        let control_str = std::str::from_utf8(control).map_err(Error::from)?;

        // Parse the message (after semicolon until ESC \)
        let end_pos = data
            .iter()
            .rposition(|&b| b == crate::ESC)
            .ok_or_else(|| Error::InvalidResponse(String::from_utf8_lossy(data).into_owned()))?;

        let message = &data[semicolon_pos + 1..end_pos];
        let message_str = std::str::from_utf8(message).map_err(Error::from)?;

        // Parse control fields
        let mut image_id = None;
        let mut image_number = None;
        let mut placement_id = None;

        for part in control_str.split(',') {
            let parts: Vec<&str> = part.splitn(2, '=').collect();
            if parts.len() == 2 {
                match parts[0] {
                    "i" => image_id = parts[1].parse().ok(),
                    "I" => image_number = parts[1].parse().ok(),
                    "p" => placement_id = parts[1].parse().ok(),
                    _ => {}
                }
            }
        }

        // Parse message
        let (success, error) = if message_str == "OK" {
            (true, None)
        } else if let Some(err_msg) = message_str.strip_prefix("ENOENT:") {
            (false, Some(format!("Not found: {err_msg}")))
        } else if let Some(err_msg) = message_str.strip_prefix("EINVAL:") {
            (false, Some(format!("Invalid argument: {err_msg}")))
        } else if let Some(err_msg) = message_str.strip_prefix("EIO:") {
            (false, Some(format!("IO error: {err_msg}")))
        } else if let Some(err_msg) = message_str.strip_prefix("ETOODEEP:") {
            (false, Some(format!("Chain too deep: {err_msg}")))
        } else if let Some(err_msg) = message_str.strip_prefix("ECYCLE:") {
            (false, Some(format!("Cycle detected: {err_msg}")))
        } else if let Some(err_msg) = message_str.strip_prefix("ENOPARENT:") {
            (false, Some(format!("Parent not found: {err_msg}")))
        } else {
            (false, Some(message_str.to_string()))
        };

        Ok(Response {
            image_id,
            image_number,
            placement_id,
            success,
            error,
        })
    }

    /// Check if this is a success response
    pub fn is_ok(&self) -> bool {
        self.success
    }

    /// Check if this is an error response
    pub fn is_error(&self) -> bool {
        !self.success
    }

    /// Get the error message if this is an error
    pub fn error_message(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

impl std::fmt::Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.success {
            write!(f, "OK")?;
            if let Some(id) = self.image_id {
                write!(f, " (image_id={}", id)?;
                if let Some(pid) = self.placement_id {
                    write!(f, ", placement_id={}", pid)?;
                }
                write!(f, ")")?;
            }
        } else if let Some(err) = &self.error {
            write!(f, "ERROR: {}", err)?;
        }
        Ok(())
    }
}

/// Common error codes returned by the terminal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    /// Not found (ENOENT)
    NotFound,
    /// Invalid argument (EINVAL)
    InvalidArgument,
    /// IO error (EIO)
    IoError,
    /// Chain too deep (ETOODEEP)
    TooDeep,
    /// Cycle detected (ECYCLE)
    Cycle,
    /// Parent not found (ENOPARENT)
    NoParent,
    /// Unknown error
    Unknown,
}

impl ErrorCode {
    /// Parse an error code from a response
    pub fn from_message(msg: &str) -> Self {
        if msg.starts_with("ENOENT") {
            Self::NotFound
        } else if msg.starts_with("EINVAL") {
            Self::InvalidArgument
        } else if msg.starts_with("EIO") {
            Self::IoError
        } else if msg.starts_with("ETOODEEP") {
            Self::TooDeep
        } else if msg.starts_with("ECYCLE") {
            Self::Cycle
        } else if msg.starts_with("ENOPARENT") {
            Self::NoParent
        } else {
            Self::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ok_response() {
        let data = b"\x1b_Gi=42;OK\x1b\\";
        let resp = Response::parse(data).unwrap();
        assert!(resp.is_ok());
        assert_eq!(resp.image_id, Some(42));
    }

    #[test]
    fn test_parse_ok_response_with_placement() {
        let data = b"\x1b_Gi=42,p=7;OK\x1b\\";
        let resp = Response::parse(data).unwrap();
        assert!(resp.is_ok());
        assert_eq!(resp.image_id, Some(42));
        assert_eq!(resp.placement_id, Some(7));
    }

    #[test]
    fn test_parse_error_response() {
        let data = b"\x1b_Gi=42;ENOENT:Image not found\x1b\\";
        let resp = Response::parse(data).unwrap();
        assert!(resp.is_error());
        assert_eq!(resp.image_id, Some(42));
        assert!(resp.error.unwrap().contains("Not found"));
    }

    #[test]
    fn test_parse_response_with_image_number() {
        let data = b"\x1b_Gi=99,I=13;OK\x1b\\";
        let resp = Response::parse(data).unwrap();
        assert!(resp.is_ok());
        assert_eq!(resp.image_id, Some(99));
        assert_eq!(resp.image_number, Some(13));
    }
}
