use lazy_static::lazy_static;
use regex::Regex;
use std::io::{self, Read, Write};
use termios::{tcsetattr, Termios, CREAD, ECHO, ICANON, TCSANOW, VMIN, VTIME};

/// Represents an RGB color with 8-bit channels.
#[derive(Debug, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// A guard to ensure the terminal is restored to its original state.
struct RawModeGuard {
    original_termios: Termios,
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        // Restore the original terminal settings when the guard goes out of scope.
        let _ = tcsetattr(libc::STDIN_FILENO, TCSANOW, &self.original_termios);
    }
}

/// Queries the terminal for its background color using an OSC 11 sequence.
///
/// Returns `Ok(None)` if the terminal does not respond within the timeout.
pub fn get_terminal_background_color() -> Result<Option<Rgb>, Box<dyn std::error::Error>> {
    // Get the file descriptor for stdin.
    let stdin_fd = libc::STDIN_FILENO;

    // Get the original terminal settings.
    let original_termios = Termios::from_fd(stdin_fd)?;

    // Set up a guard to restore the terminal settings on drop.
    let _guard = RawModeGuard { original_termios };

    // Create a new Termios struct for raw mode.
    let mut raw_termios = original_termios;

    // Disable canonical mode, echo, and signal generation.
    raw_termios.c_lflag &= !(ICANON | ECHO);
    // Ensure read is enabled.
    raw_termios.c_cflag |= CREAD;

    // Set a read timeout. This is the crucial part.
    // VMIN = 0, VTIME > 0: read() will block for at most VTIME * 0.1 seconds.
    // If no data is available, it returns 0.
    raw_termios.c_cc[VMIN] = 0;
    // We set a timeout of 100ms (1 decisecond).
    raw_termios.c_cc[VTIME] = 1;

    // Apply the raw mode settings.
    tcsetattr(stdin_fd, TCSANOW, &raw_termios)?;

    // --- Send the query to the terminal ---
    let mut stdout = io::stdout();
    // OSC 11 query for background color: \x1b]11;?\x07
    stdout.write_all(b"\x1b]11;?\x07")?;
    stdout.flush()?;

    // --- Read the response ---
    let mut response_bytes = Vec::new();
    // stdin().read_to_end will now respect the VTIME timeout.
    io::stdin().read_to_end(&mut response_bytes)?;

    // The guard will automatically restore the terminal settings here.

    // --- Parse the response ---
    if response_bytes.is_empty() {
        return Ok(None); // Timeout, no response
    }

    let response_str = String::from_utf8_lossy(&response_bytes);

    lazy_static! {
        // Regex to parse: `\x1b]11;rgb:rrrr/gggg/bbbb\x07`
        static ref RE: Regex = Regex::new(
            r"\x1b]11;rgb:(?P<r>[0-9a-fA-F]{4})/(?P<g>[0-9a-fA-F]{4})/(?P<b>[0-9a-fA-F]{4})"
        ).unwrap();
    }

    if let Some(caps) = RE.captures(&response_str) {
        let r16 = u16::from_str_radix(&caps["r"], 16)?;
        let g16 = u16::from_str_radix(&caps["g"], 16)?;
        let b16 = u16::from_str_radix(&caps["b"], 16)?;

        // The terminal returns 16-bit colors. We need to scale them down to 8-bit.
        // The correct way is to divide by 257, because 65535 (0xffff) / 257 = 255.
        // A simple bitshift `>> 8` is a common and usually acceptable alternative.
        let rgb = Rgb {
            r: (r16 / 257) as u8,
            g: (g16 / 257) as u8,
            b: (b16 / 257) as u8,
        };
        Ok(Some(rgb))
    } else {
        Ok(None) // Response was not in the expected format
    }
}
