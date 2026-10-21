use serde::Serialize;
use std::net::IpAddr;

use crate::scanner::PortStatus;
use crate::scanner::ScanResult;

// Serializable representation of a complete scan
// Summary counters include every scanned port, even if some are hidden from output
#[derive(Serialize)]
struct ScanReport {
    ip: String,
    open: usize,
    closed: usize,
    timedout: usize,
    errors: usize,
    results: Vec<PortReport>,
}

// Serializable representation of an individual port
// Status uses a static string literal to avoid allocating a String for each port
// Banner and error are optional because they are not available for every status
#[derive(Serialize)]
pub struct PortReport {
    port: u16,
    status: &'static str,
    banner: Option<String>,
    error: Option<String>,
}

/// Builds a scan report and prints it in human-readable or JSON format
///
/// `json` selects the output format
/// `show_all` controls whether non-open ports appear in the detailed results
/// Summary counters always include all scanned ports
pub fn report(
    ip: IpAddr,
    results: &[ScanResult],
    json: bool,
    show_all: bool,
) -> anyhow::Result<()> {
    // Initialize the report before processing individual scan results
    let mut scan_report = ScanReport {
        ip: ip.to_string(),
        open: 0,
        closed: 0,
        timedout: 0,
        errors: 0,
        results: vec![],
    };

    for result in results {
        // Match by reference to avoid moving the status out of the borrowed ScanResult
        // Each arm updates its summary counter and returns a status/error tuple
        let (status, error) = match &result.port_status {
            PortStatus::Open => {
                scan_report.open += 1;

                // Suppress human-readable output when JSON mode is enabled
                if !json {
                    println!("{ip}:{} is OPEN", result.port_number);

                    if let Some(banner) = &result.banner {
                        println!("  Banner: {}", banner.trim());
                    }
                }

                ("Open", None)
            }

            PortStatus::Closed => {
                scan_report.closed += 1;

                if show_all && !json {
                    println!("{ip}:{} is CLOSED", result.port_number);
                }

                ("Closed", None)
            }

            PortStatus::TimedOut => {
                scan_report.timedout += 1;

                if show_all && !json {
                    println!("{ip}:{} TIMEDOUT", result.port_number);
                }

                ("TimedOut", None)
            }

            PortStatus::Error(e) => {
                scan_report.errors += 1;

                if show_all {
                    if !json {
                        println!("{ip}:{} error: {e}", result.port_number);
                    }

                    // Convert the I/O error into an owned string for serialization
                    ("Error", Some(e.to_string()))
                } else {
                    // Avoid allocating an error message for a result that will be omitted
                    ("Error", None)
                }
            }
        };

        // Include all ports with --all; otherwise include only open ports.
        if show_all || matches!(&result.port_status, PortStatus::Open) {
            scan_report.results.push(PortReport {
                port: result.port_number,
                status,

                // Clone the optional banner because results is a borrowed slice
                // and PortReport needs to own its banner data
                banner: result.banner.clone(),
                error,
            });
        }
    }

    // Output exactly one format so JSON remains valid for external tools
    if !json {
        println!(
            "Summary: {} open, {} closed, {} timed out, {} errors",
            scan_report.open, scan_report.closed, scan_report.timedout, scan_report.errors
        )
    } else {
        // Serialize the completed report; propagate serialization errors to the caller
        println!("{}", serde_json::to_string_pretty(&scan_report)?);
    }

    Ok(())
}
