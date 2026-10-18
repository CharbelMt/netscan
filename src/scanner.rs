use std::io::{self, Error};
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use thiserror::Error;

use crate::banner::grab_banner;

pub struct ScanResult {
    pub port_number: u16,
    pub port_status: PortStatus,
    pub banner: Option<String>,
}

pub enum PortStatus {
    Open,
    Closed,
    TimedOut,
    Error(Error),
}

pub struct Config {
    worker_count: usize,
    timeout: Duration,
    banner_timeout: Duration,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Cannot spawn 0 workers")]
    ZeroWorkers,

    #[error("Cannot spawn more than 100 workers")]
    TooManyWorkers,

    #[error("Connection timeout cannot be 0")]
    ZeroConnectionTimeout,

    #[error("Banner read timeout cannot be 0")]
    ZeroBannerTimeout,
}

impl Config {
    pub fn new(
        worker_count: usize,
        timeout: Duration,
        banner_timeout: Duration,
    ) -> Result<Self, ConfigError> {
        // safety checks for worker count and timeout
        if worker_count == 0 {
            return Err(ConfigError::ZeroWorkers);
        }
        if worker_count > 100 {
            return Err(ConfigError::TooManyWorkers);
        }
        if timeout == Duration::ZERO {
            return Err(ConfigError::ZeroConnectionTimeout);
        }
        if banner_timeout == Duration::ZERO {
            return Err(ConfigError::ZeroBannerTimeout);
        }

        Ok(Self {
            worker_count,
            timeout,
            banner_timeout,
        })
    }
}

pub fn scan_ports(ip: IpAddr, ports: Vec<u16>, scan_config: &Config) -> Vec<ScanResult> {
    let worker_count = scan_config.worker_count.min(ports.len());
    let timeout_time = scan_config.timeout;
    let banner_timeout = scan_config.banner_timeout;

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
                        local_results.push(scan_port(ip, n, timeout_time, banner_timeout)); // return expect result type of port + port status
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

fn scan_port(
    ip: IpAddr,
    port: u16,
    connection_timeout: Duration,
    banner_timeout: Duration,
) -> ScanResult {
    let address = SocketAddr::new(ip, port); // create socket address from ip + port

    // match on successful connection
    match TcpStream::connect_timeout(&address, connection_timeout) {
        // if successful connection
        Ok(mut stream) => {
            let banner = grab_banner(&mut stream, banner_timeout);

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
