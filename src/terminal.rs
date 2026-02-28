//! Terminal utilities for the Kitty graphics protocol

use crate::error::{Error, Result};
use std::io::{self, Read, Write};

/// Terminal window size information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowSize {
    /// Number of rows (lines)
    pub rows: u16,
    /// Number of columns (characters)
    pub cols: u16,
    /// Screen width in pixels
    pub width: u16,
    /// Screen height in pixels
    pub height: u16,
}

impl WindowSize {
    /// Get the cell width in pixels
    pub fn cell_width(&self) -> u16 {
        if self.cols > 0 {
            self.width / self.cols
        } else {
            0
        }
    }

    /// Get the cell height in pixels
    pub fn cell_height(&self) -> u16 {
        if self.rows > 0 {
            self.height / self.rows
        } else {
            0
        }
    }

    /// Calculate how many cells are needed for an image of given pixel dimensions
    pub fn cells_for_image(&self, img_width: u32, img_height: u32) -> (u32, u32) {
        let cell_w = self.cell_width() as u32;
        let cell_h = self.cell_height() as u32;

        if cell_w == 0 || cell_h == 0 {
            return (0, 0);
        }

        let cols = img_width.div_ceil(cell_w);
        let rows = img_height.div_ceil(cell_h);

        (cols, rows)
    }
}

#[cfg(unix)]
mod unix {
    use super::*;
    use libc::{STDOUT_FILENO, TIOCGWINSZ, ioctl, winsize};

    /// Get the terminal window size using TIOCGWINSZ ioctl
    pub fn get_window_size() -> Result<WindowSize> {
        unsafe {
            let mut ws: winsize = std::mem::zeroed();
            let result = ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut ws);

            if result == -1 {
                return Err(Error::Io(io::Error::last_os_error()));
            }

            Ok(WindowSize {
                rows: ws.ws_row,
                cols: ws.ws_col,
                width: ws.ws_xpixel,
                height: ws.ws_ypixel,
            })
        }
    }
}

#[cfg(not(unix))]
mod other {
    use super::*;

    /// Get the terminal window size (stub for non-Unix systems)
    pub fn get_window_size() -> Result<WindowSize> {
        Err(Error::protocol(
            "get_window_size is only supported on Unix systems",
        ))
    }
}

#[cfg(not(unix))]
pub use other::get_window_size;
#[cfg(unix)]
pub use unix::get_window_size;

/// Query the terminal for window size using CSI 14 t escape code
/// This works across more terminals but requires terminal interaction
pub fn query_window_size() -> Result<WindowSize> {
    let mut stdout = io::stdout();
    let mut stdin = io::stdin();

    // Save current terminal settings
    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        let fd = stdin.as_raw_fd();
        let mut termios = std::mem::MaybeUninit::uninit();
        if unsafe { libc::tcgetattr(fd, termios.as_mut_ptr()) } == 0 {
            let termios = unsafe { termios.assume_init() };
            let _ = unsafe { libc::tcsetattr(fd, libc::TCSAFLUSH, &termios) };
        }
    }

    // Send CSI 14 t query
    write!(stdout, "\x1b[14t")?;
    stdout.flush()?;

    // Read response: ESC [ 4 ; <height> ; <width> t
    let mut response = Vec::new();
    let mut buf = [0u8; 1];

    loop {
        let n = stdin.read(&mut buf)?;
        if n == 0 {
            break;
        }
        response.push(buf[0]);
        if buf[0] == b't' {
            break;
        }
        if response.len() > 100 {
            break; // Safety limit
        }
    }

    // Parse response
    let response_str = String::from_utf8(response).map_err(Error::from)?;
    parse_size_response(&response_str)
}

fn parse_size_response(response: &str) -> Result<WindowSize> {
    // Expected format: ESC[4;<height>;<width>t
    if !response.starts_with("\x1b[4;") {
        return Err(Error::InvalidResponse(response.to_string()));
    }

    let parts: Vec<&str> = response[4..].trim_end_matches('t').split(';').collect();
    if parts.len() < 2 {
        return Err(Error::InvalidResponse(response.to_string()));
    }

    let height: u16 = parts[0]
        .parse()
        .map_err(|_| Error::InvalidResponse(response.to_string()))?;
    let width: u16 = parts[1]
        .parse()
        .map_err(|_| Error::InvalidResponse(response.to_string()))?;

    // Get rows/cols using stty or default values
    let (rows, cols) = get_terminal_size_from_stty()?;

    Ok(WindowSize {
        rows,
        cols,
        width,
        height,
    })
}

#[cfg(unix)]
fn get_terminal_size_from_stty() -> Result<(u16, u16)> {
    use std::process::Command;

    let output = Command::new("stty").arg("size").output()?;

    if !output.status.success() {
        return Err(Error::Io(io::Error::other("stty size failed")));
    }

    let size_str = String::from_utf8_lossy(&output.stdout);
    let size_owned = size_str.into_owned();
    let parts: Vec<&str> = size_owned.split_whitespace().collect();

    if parts.len() < 2 {
        return Err(Error::InvalidResponse(size_owned));
    }

    let rows: u16 = parts[0]
        .parse()
        .map_err(|_| Error::InvalidResponse(size_owned.clone()))?;
    let cols: u16 = parts[1]
        .parse()
        .map_err(|_| Error::InvalidResponse(size_owned))?;

    Ok((rows, cols))
}

#[cfg(not(unix))]
fn get_terminal_size_from_stty() -> Result<(u16, u16)> {
    // Default values for non-Unix systems
    Ok((24, 80))
}

/// Check if the terminal supports the Kitty graphics protocol
pub fn check_protocol_support() -> Result<bool> {
    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;

        let stdin = io::stdin();
        let fd = stdin.as_raw_fd();

        // Check if stdin is a TTY
        if unsafe { libc::isatty(fd) } != 1 {
            // Not a TTY, can't reliably check - assume supported
            // This happens when running through cargo run or pipes
            return Ok(true);
        }

        let mut stdout = io::stdout();

        // Save original terminal settings
        let mut original_termios: libc::termios = unsafe { std::mem::zeroed() };
        if unsafe { libc::tcgetattr(fd, &mut original_termios) } != 0 {
            // Can't get terminal attributes - assume supported
            return Ok(true);
        }

        // Set terminal to raw mode
        let mut raw_termios = original_termios;
        unsafe { libc::cfmakeraw(&mut raw_termios) };
        if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &raw_termios) } != 0 {
            // Can't set raw mode - assume supported
            return Ok(true);
        }

        // Send query command
        // a=q means query, i=31 is image ID, s=1,v=1 is 1x1 pixel, f=24 is RGB format
        let _ = write!(stdout, "\x1b_Ga=q,i=31,s=1,v=1,f=24;AAAA\x1b\\");
        let _ = stdout.flush();

        // Read response with timeout
        let mut response = Vec::new();
        let mut buf = [0u8; 256];

        let start = std::time::Instant::now();

        loop {
            if start.elapsed() > std::time::Duration::from_millis(200) {
                break;
            }

            // Use select for timeout
            let mut tv = libc::timeval {
                tv_sec: 0,
                tv_usec: 50_000, // 50ms
            };

            // Set up fd_set for select
            let mut read_fds: libc::fd_set = unsafe { std::mem::zeroed() };
            unsafe { libc::FD_SET(fd, &mut read_fds) };

            let ready = unsafe {
                libc::select(
                    fd + 1,
                    &mut read_fds,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    &mut tv,
                )
            };

            if ready > 0 {
                let n = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
                if n > 0 {
                    response.extend_from_slice(&buf[..n as usize]);
                    // Check if we got the complete response (ends with ESC \)
                    if response.windows(2).any(|w| w == &[0x1b, b'\\']) {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        // Restore original terminal settings
        unsafe { libc::tcsetattr(fd, libc::TCSANOW, &original_termios) };

        let response_str = String::from_utf8_lossy(&response);

        // Check for valid Kitty graphics protocol response
        // Response format: ESC _ G i=31;OK ESC \
        let has_apc = response.windows(3).any(|w| w == &[0x1b, b'_', b'G']);
        let has_ok = response_str.contains("OK");
        let has_error = response_str.contains("ENO");

        Ok(has_apc && (has_ok || has_error))
    }

    #[cfg(not(unix))]
    {
        // On non-Unix systems, assume supported
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_size_calculation() {
        let ws = WindowSize {
            rows: 40,
            cols: 120,
            width: 1200,
            height: 800,
        };

        assert_eq!(ws.cell_width(), 10);
        assert_eq!(ws.cell_height(), 20);

        let (cols, rows) = ws.cells_for_image(100, 100);
        assert_eq!(cols, 10);
        assert_eq!(rows, 5);
    }

    #[test]
    fn test_window_size_edge_cases() {
        let ws = WindowSize {
            rows: 0,
            cols: 0,
            width: 0,
            height: 0,
        };

        assert_eq!(ws.cell_width(), 0);
        assert_eq!(ws.cell_height(), 0);
        assert_eq!(ws.cells_for_image(100, 100), (0, 0));
    }
}
