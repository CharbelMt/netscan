use std::io::{self, Read};
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::time::Duration;

use crate::scanner::PortStatus;
use crate::scanner::ScanResult;

pub fn grab_banner(ip: IpAddr, port: u16, timeout_time: Duration) -> ScanResult {
    let address = SocketAddr::new(ip, port); // create socket address from ip + port

    // match on successful connection
    match TcpStream::connect_timeout(&address, timeout_time) {
        // if successful connection
        Ok(mut stream) => {
            let mut buffer = [0u8; 1024]; // memory buffer to store bytes with data

            stream
                .set_read_timeout(Some(timeout_time))
                .expect("read timeout error"); // set read timeout with the same given timeout for the connection

            // try to extract banner bytes and data
            let banner = match stream.read(&mut buffer) {
                // if 0 bytes of banner data, return None
                Ok(0) => None,

                // if n bytes, construct a string from the buffer
                Ok(n) => {
                    let text = String::from_utf8_lossy(&buffer[..n]).into_owned(); // convert from a slice of bytes to a String

                    Some(text)
                }

                Err(_) => None, //
            };

            ScanResult {
                port_number: port,
                port_status: PortStatus::Open,
                banner,
            }
        }

        Err(e) if e.kind() == io::ErrorKind::ConnectionRefused => ScanResult {
            port_number: port,
            port_status: PortStatus::Closed,
            banner: None,
        },

        Err(e) if e.kind() == io::ErrorKind::TimedOut => ScanResult {
            port_number: port,
            port_status: PortStatus::TimedOut,
            banner: None,
        },

        Err(e) => ScanResult {
            port_number: port,
            port_status: PortStatus::Error(e),
            banner: None,
        },
    }
}
