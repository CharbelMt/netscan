use std::io;
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
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

    // end results for each thread
    let mut results: Vec<JoinHandle<Vec<(u16, PortStatus)>>> = vec![];

    // ports to be scanned; 1 -> 10k
    let mut ports: Vec<u16> = vec![];

    for port in start_port..=end_port {
        ports.push(port);
    }

    // create the shared Arc queue
    let work_queue = Arc::new(Mutex::new(ports));

    // create 100 workers
    for _ in 1..=100 {
        let clone = Arc::clone(&work_queue);

        // push join handles into results
        results.push(thread::spawn(move || {
            let mut local_results = vec![];

            // thread loop
            loop {
                // mutex guard lock and unlock after extracting port
                let port = {
                    let mut guard = clone.lock().unwrap();
                    guard.pop()
                };

                // match on Option<u16> for port
                match port {
                    Some(n) => {
                        let res = scan_port(ip, n);
                        local_results.push((n, res)); // return expect result type of port + port status
                    }
                    None => break, // break from the thread
                }
            }

            local_results
        }));
    }

    let mut result_list = vec![];

    // join joinhandles and retrieve the local result vector from each thread
    for i in results {
        result_list.extend(i.join().unwrap()); // store results in a master result list by extending instead of pushing
    }

    result_list.sort_by_key(|res| res.0); // sort the list based on port number

    // status report for each port
    // destructure it to match only on status
    for (port, status) in result_list {
        match status {
            PortStatus::Open => {
                println!("{ip}:{port} is OPEN");
            }
            PortStatus::Closed => {
                println!("{ip}:{port} is CLOSED");
            }
            PortStatus::TimedOut => {
                println!("{ip}:{port} TIMEDOUT");
            }
            PortStatus::Error(e) => {
                println!("{ip}:{port} error: {e}");
            }
        }
    }
}
