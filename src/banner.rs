use std::io::Read;
use std::net::TcpStream;
use std::time::Duration;

pub fn grab_banner(stream: &mut TcpStream, timeout: Duration) -> Option<String> {
    let mut buffer = [0u8; 1024]; // memory buffer to store bytes with data

    // set read timeout with the configured timeout for banner read
    match stream.set_read_timeout(Some(timeout)) {
        Ok(_) => {
            // try to extract banner bytes and data
            match stream.read(&mut buffer) {
                // if 0 bytes of banner data, return None
                Ok(0) => None,

                // if n bytes, construct a string from the buffer
                Ok(n) => {
                    let text = String::from_utf8_lossy(&buffer[..n]).into_owned(); // convert from a slice of bytes to a String

                    Some(sanitize_banner(&text))
                }

                // read error, return None
                Err(_) => None,
            }
        }

        // timeout configuration failure
        Err(_) => None,
    }
}

fn sanitize_banner(banner_text: &str) -> String {
    let mut sanitized_banner = String::new();

    for char in banner_text.chars() {
        if char.is_control() {
            sanitized_banner.extend(char.escape_default());
        } else {
            sanitized_banner.push(char);
        }
    }

    sanitized_banner
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn printable_text() {
        assert_eq!(sanitize_banner("héllo world!"), "héllo world!");
    }

    #[test]
    fn whitespace_controls() {
        assert_eq!(sanitize_banner("hello\r\nworld\t!"), "hello\\r\\nworld\\t!")
    }

    #[test]
    fn ansi_escape_sequence() {
        assert_eq!(sanitize_banner("\u{1b}[31mHELLO"), "\\u{1b}[31mHELLO");
    }
}
