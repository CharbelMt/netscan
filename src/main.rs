use clap::Parser;
use std::net::IpAddr;
use std::time::Duration;

use netscan::report::report;
use netscan::scanner::{Config, scan_ports};
use netscan::target::parse_ports;

#[derive(Parser)]
struct Args {
    ip: IpAddr,
    #[arg(long)]
    ports: String,
    #[arg(long)]
    threads: usize,
    #[arg(long)]
    timeout: u64,
    #[arg(long)]
    all: bool,
}

fn main() {
    // get input from program run (ex. cargo run -- 127.0.0.1 --ports 22,80,100-105 --threads 4 --timeout 1)
    let args = Args::parse();

    // ports to be scanned
    let ports: Vec<u16> = match parse_ports(&args.ports) {
        Ok(x) => x,
        Err(e) => panic!("{}", e),
    };

    if args.threads > ports.len() {
        panic!("You cannot spawn more threads than there are ports.");
    }

    let scan_config = match Config::new(args.threads, Duration::from_secs(args.timeout)) {
        Ok(x) => x,
        Err(e) => {
            panic!("Config error: {}", e);
        }
    };

    let result_list = scan_ports(args.ip, ports, &scan_config);

    report(args.ip, &result_list, args.all);
}
