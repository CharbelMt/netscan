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

                    Some(text)
                }

                // read error, return None
                Err(_) => None,
            }
        }

        // timeout configuration failure
        Err(_) => None,
    }
}
