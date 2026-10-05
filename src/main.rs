use std::io;
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::thread::{self, JoinHandle};
use std::time::Duration;

enum PortStatus {
    Open,
    Closed,
    TimedOut,
    Error(io::Error),
}

fn scan_port(ip: IpAddr, port: u16) -> PortStatus {
    let address = SocketAddr::new(ip, port);
    let timeout = Duration::from_secs(3);

    match TcpStream::connect_timeout(&address, timeout) {
        Ok(_) => PortStatus::Open,

        Err(e) if e.kind() == io::ErrorKind::ConnectionRefused => PortStatus::Closed,

        Err(e) if e.kind() == io::ErrorKind::TimedOut => PortStatus::TimedOut,

        Err(e) => PortStatus::Error(e),
    }
}

fn main() {
    let mut buffer = String::new();

    println!("enter ip, start port and end port separated by space: ");

    io::stdin().read_line(&mut buffer).unwrap();

    let things: Vec<&str> = buffer.split_whitespace().collect();

    let [a, b, c] = &things[..] else {
        panic!("bad input")
    };

    let ip: IpAddr = a.parse().unwrap();
    let mut start_port: u16 = b.parse().unwrap();
    let mut end_port: u16 = c.parse().unwrap();

    if start_port > end_port {
        std::mem::swap(&mut start_port, &mut end_port);
    }

    let mut res: Vec<JoinHandle<(u16, PortStatus)>> = vec![];

    for port in start_port..=end_port {
        res.push(thread::spawn(move || (port, scan_port(ip, port))));
    }

    for i in res {
        match i.join().unwrap() {
            (port, PortStatus::Open) => {
                println!("{ip}:{port} is OPEN");
            }
            (port, PortStatus::Closed) => {
                println!("{ip}:{port} is CLOSED");
            }
            (port, PortStatus::TimedOut) => {
                println!("{ip}:{port} TIMEDOUT");
            }
            (port, PortStatus::Error(e)) => {
                println!("{ip}:{port} error: {e}");
            }
        }
    }
}
