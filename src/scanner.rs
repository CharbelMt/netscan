use std::io::{self, Error};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;
use tokio::time::timeout;

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

pub async fn scan_ports(ip: IpAddr, ports: Vec<u16>, scan_config: &Config) -> Vec<ScanResult> {
    let task_count = scan_config.worker_count.min(ports.len());
    let timeout_time = scan_config.timeout;
    let banner_timeout = scan_config.banner_timeout;

    // create a semaphore to manage concurrent worker execution limits
    let worker_capacity = Arc::new(Semaphore::new(task_count));

    // store join handles for each spawned port scanning task to await them later
    let mut results: Vec<JoinHandle<ScanResult>> = vec![];

    // iterate through all target ports by spawning tasks
    for current_port in ports {
        // concurrency permit
        let permit = worker_capacity.clone().acquire_owned().await.unwrap();

        // spawn an independent Tokio task to execute the port scan concurrently
        results.push(tokio::spawn(async move {
            let result_future = scan_port(ip, current_port, timeout_time, banner_timeout);
            let result = result_future.await;

            // release the semaphore permit back to the pool after scanning
            drop(permit);

            result
        }));
    }

    // await all spawned tasks and collect their respective port scan outcomes
    let mut result_list = vec![];

    for result in results {
        result_list.push(result.await.unwrap());
    }

    // sort the final results sequentially
    result_list.sort_by_key(|res| res.port_number);

    result_list
}

async fn scan_port(
    ip: IpAddr,
    port: u16,
    connection_timeout: Duration,
    banner_timeout: Duration,
) -> ScanResult {
    let address = SocketAddr::new(ip, port); // create socket address from ip + port

    // match on successful connection
    match timeout(connection_timeout, TcpStream::connect(&address)).await {
        // successful connection; port open
        Ok(Ok(mut stream)) => {
            let banner = grab_banner(&mut stream, banner_timeout).await;

            ScanResult {
                port_number: port,
                port_status: PortStatus::Open,
                banner,
            }
        }

        // unsuccessful connection; port closed but tokio didn't timeout
        Ok(Err(e)) if e.kind() == io::ErrorKind::ConnectionRefused => ScanResult {
            port_number: port,
            port_status: PortStatus::Closed,
            banner: None,
        },

        // unsuccessful connection; tcp timedout
        Ok(Err(e)) if e.kind() == io::ErrorKind::TimedOut => ScanResult {
            port_number: port,
            port_status: PortStatus::TimedOut,
            banner: None,
        },

        // unsuccessful connection; tokio didn't timeout
        Ok(Err(e)) => ScanResult {
            port_number: port,
            port_status: PortStatus::Error(e),
            banner: None,
        },

        // tokio timeout
        Err(_) => ScanResult {
            port_number: port,
            port_status: PortStatus::TimedOut,
            banner: None,
        },
    }
}
