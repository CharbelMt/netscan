use std::io::Error;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

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
}

impl Config {
    pub fn new(worker_count: usize, timeout: Duration) -> Result<Self, &'static str> {
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

pub fn scan_ports(ip: IpAddr, ports: Vec<u16>, scan_config: &Config) -> Vec<ScanResult> {
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
                        local_results.push(grab_banner(ip, n, timeout_time)); // return expect result type of port + port status
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
