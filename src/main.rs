use std::io::{self, Read};
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
        if worker_count > 100 {
            return Err("Worker count cannot exceed 100.");
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
    banner: Option<String>,
}

fn scan_port(ip: IpAddr, port: u16, timeout_time: Duration) -> ScanResult {
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

fn scan_ports(ip: IpAddr, ports: Vec<u16>, scan_config: &Config) -> Vec<ScanResult> {
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
                        local_results.push(scan_port(ip, n, timeout_time)); // return expect result type of port + port status
                    }
                    None => break, // break from the thread
                }
            }

            local_results
        }));
    }

    let mut result_list = vec![];

    // join joinhandles and retrieve the local result vector from each thread
    for handle in results {
        result_list.extend(handle.join().unwrap()); // store results in a master result list by extending instead of pushing
    }

    result_list.sort_by_key(|res| res.port_number); // sort the list based on port number

    result_list
}

fn print_results(ip: IpAddr, results: &[ScanResult]) {
    // status report for each port
    // destructure it to match only on status
    for result in results {
        match &result.port_status {
            PortStatus::Open => {
                println!("{ip}:{} is OPEN", result.port_number);

                if let Some(banner) = &result.banner {
                    println!("  Banner: {}", banner.trim());
                }
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

    let number_of_threads = num_workers.parse().unwrap();

    if number_of_threads > ports.len() {
        panic!("You cannot spawn more threads than there are ports.");
    }

    let scan_config = match Config::new(
        number_of_threads,
        Duration::from_secs(timeout_dur.parse().unwrap()),
    ) {
        Ok(x) => x,
        Err(e) => {
            panic!("Config error: {}", e);
        }
    };

    let result_list = scan_ports(ip, ports, &scan_config);

    print_results(ip, &result_list);
}
