use std::net::IpAddr;

use crate::scanner::PortStatus;
use crate::scanner::ScanResult;

pub fn report(ip: IpAddr, results: &[ScanResult]) {
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
