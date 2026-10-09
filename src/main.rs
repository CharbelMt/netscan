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

struct Config {
    worker_count: usize,
    timeout: Duration,
}

impl Config {
    fn new(worker_count: usize, timeout: Duration) -> Result<Self, &'static str> {
        // safety checks for worker count and timeout
        if worker_count == 0 {
            return Err("Worker count cannot be 0");
        }
        if timeout == Duration::ZERO {
            return Err("Timeout time cannot be 0");
        }

        Ok(Self {
            worker_count,
            timeout,
        })
    }
}

struct ScanResult {
    port_number: u16,
    port_status: PortStatus,
}

fn scan_port(ip: IpAddr, port: u16, timeout_time: Duration) -> PortStatus {
    let address = SocketAddr::new(ip, port);

    match TcpStream::connect_timeout(&address, timeout_time) {
        Ok(_) => PortStatus::Open,

        Err(e) if e.kind() == io::ErrorKind::ConnectionRefused => PortStatus::Closed,

        Err(e) if e.kind() == io::ErrorKind::TimedOut => PortStatus::TimedOut,

        Err(e) => PortStatus::Error(e),
    }
}

fn main() {
    // get user input
    let mut buffer = String::new();
    let mut buffer2 = String::new();

    println!("enter ip, start port and end port separated by space: ");
    io::stdin().read_line(&mut buffer).unwrap();

    println!("enter number of threads and timeout duration, separated by space: ");
    io::stdin().read_line(&mut buffer2).unwrap();

    let scan_range: Vec<&str> = buffer.split_whitespace().collect();
    let scanner_config: Vec<&str> = buffer2.split_whitespace().collect();

    let [a, b, c] = &scan_range[..] else {
        panic!("bad input")
    };
    let [num_workers, timeout_dur] = &scanner_config[..] else {
        panic!("bad input")
    };

    let ip: IpAddr = a.parse().unwrap();
    let mut start_port: u16 = b.parse().unwrap();
    let mut end_port: u16 = c.parse().unwrap();

    // ensure start port is less than end port
    if start_port > end_port {
        std::mem::swap(&mut start_port, &mut end_port);
    }

    // ports to be scanned; 1 -> 10k
    let mut ports: Vec<u16> = vec![];

    for port in start_port..=end_port {
        ports.push(port);
    }

    let scan_config = match Config::new(
        num_workers.parse().unwrap(),
        Duration::from_secs(timeout_dur.parse().unwrap()),
    ) {
        Ok(x) => x,
        Err(e) => {
            panic!("Config error: {}", e);
        }
    };

    let worker_count = scan_config.worker_count;
    let timeout_time = scan_config.timeout;

    // create the shared Arc queue
    let work_queue = Arc::new(Mutex::new(ports));

    // end results for each thread
    let mut results: Vec<JoinHandle<Vec<ScanResult>>> = vec![];

    // create workers
    for _ in 1..=worker_count {
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
                        let res = scan_port(ip, n, timeout_time);
                        local_results.push(ScanResult {
                            port_number: n,
                            port_status: res,
                        }); // return expect result type of port + port status
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

    result_list.sort_by_key(|res| res.port_number); // sort the list based on port number

    // status report for each port
    // destructure it to match only on status
    for result in result_list {
        match result.port_status {
            PortStatus::Open => {
                println!("{ip}:{} is OPEN", result.port_number);
            }
            PortStatus::Closed => {
                println!("{ip}:{} is CLOSED", result.port_number);
            }
            PortStatus::TimedOut => {
                println!("{ip}:{} TIMEDOUT", result.port_number);
            }
            PortStatus::Error(e) => {
                println!("{ip}:{} error: {e}", result.port_number);
            }
        }
    }
}
