use std::net::IpAddr;

use crate::scanner::PortStatus;
use crate::scanner::ScanResult;

struct Summary {
    open: u16,
    closed: u16,
    timedout: u16,
    errors: u16,
}

pub fn report(ip: IpAddr, results: &[ScanResult], show_all: bool) {
    let mut summary = Summary {
        open: 0,
        closed: 0,
        timedout: 0,
        errors: 0,
    };

    // status report for each port
    // destructure it to match only on status
    for result in results {
        match &result.port_status {
            PortStatus::Open => {
                summary.open += 1;

                println!("{ip}:{} is OPEN", result.port_number);

                if let Some(banner) = &result.banner {
                    println!("  Banner: {}", banner.trim());
                }
            }
            PortStatus::Closed => {
                summary.closed += 1;

                if show_all {
                    println!("{ip}:{} is CLOSED", result.port_number);
                }
            }
            PortStatus::TimedOut => {
                summary.timedout += 1;

                if show_all {
                    println!("{ip}:{} TIMEDOUT", result.port_number);
                }
            }
            PortStatus::Error(e) => {
                summary.errors += 1;

                if show_all {
                    println!("{ip}:{} error: {e}", result.port_number);
                }
            }
        }
    }

    println!(
        "Summary: {} open, {} closed, {} timed out, {} errors",
        summary.open, summary.closed, summary.timedout, summary.errors
    )
}
