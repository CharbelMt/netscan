use clap::Parser;
use std::net::IpAddr;
use std::time::Duration;

use netscan::report::report;
use netscan::scanner::{Config, scan_ports};
use netscan::target::parse_ports;

/// A multithreaded TCP port scanner.
#[derive(Parser)]
struct Args {
    /// Target IPv4 or IPv6 address to scan.
    ip: IpAddr,

    /// Ports to scan, specified individually or as ranges (e.g. 22,80,100-105).
    #[arg(short = 'p', long, default_value = "1-1024")]
    ports: String,

    /// Maximum number of concurrent scanning workers (1-100).
    #[arg(short = 'w', long, default_value_t = 100)]
    workers: usize,

    /// Connection timeout per port, in milliseconds.
    #[arg(short = 't', long, default_value_t = 1000)]
    timeout: u64,

    /// Banner read timeout per port, in milliseconds.
    #[arg(short = 'b', long, default_value_t = 500)]
    banner_timeout: u64,

    /// Include closed, timed-out, and errored ports in the output.
    #[arg(long)]
    all: bool,
}

fn main() {
    // get input from program run (ex. cargo run -- 127.0.0.1 --ports 22,80,100-105 --workers 4 --timeout 1000)
    // (short form: cargo run -- 127.0.0.1 -p 22,80,100-105 -w 4 -t 1000)
    let args = Args::parse();

    // ports to be scanned
    let ports: Vec<u16> = match parse_ports(&args.ports) {
        Ok(x) => x,
        Err(e) => panic!("{}", e),
    };

    let scan_config = match Config::new(
        args.workers,
        Duration::from_millis(args.timeout),
        Duration::from_millis(args.banner_timeout),
    ) {
        Ok(x) => x,
        Err(e) => {
            panic!("Config error: {}", e);
        }
    };

    let result_list = scan_ports(args.ip, ports, &scan_config);

    report(args.ip, &result_list, args.all);
}
