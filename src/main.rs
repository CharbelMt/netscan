use clap::Parser;
use std::time::Duration;

use netscan::report::report;
use netscan::scanner::{Config, scan_ports};
use netscan::target::{parse_ports, resolve_target};

/// An asynchronous TCP port scanner with banner grabbing.
#[derive(Parser)]
struct Args {
    /// Target hostname or IPv4/IPv6 address.
    ip: String,

    /// Ports to scan (e.g. 22,80,100-105).
    #[arg(short = 'p', long, default_value = "1-1024")]
    ports: String,

    /// Maximum number of concurrent port scans (1-100).
    #[arg(short = 'c', long, default_value_t = 100)]
    concurrency: usize,

    /// TCP connection timeout per port, in milliseconds.
    #[arg(short = 't', long, default_value_t = 1000)]
    timeout: u64,

    /// Banner read timeout per open port, in milliseconds.
    #[arg(short = 'b', long, default_value_t = 500)]
    banner_timeout: u64,

    /// Output scan results as JSON.
    #[arg(long)]
    json: bool,

    /// Include closed, timed-out, and errored ports.
    #[arg(long)]
    all: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // get input from program run (ex. cargo run -- 127.0.0.1 --ports 22,80,100-105 --concurrency 4 --timeout 1000)
    // (short form: cargo run -- 127.0.0.1 -p 22,80,100-105 -c 4 -t 1000)
    let args = Args::parse();

    let resolved_ip = resolve_target(&args.ip)?;

    // ports to be scanned
    let ports: Vec<u16> = parse_ports(&args.ports)?;

    let scan_config = Config::new(
        args.concurrency,
        Duration::from_millis(args.timeout),
        Duration::from_millis(args.banner_timeout),
    )?;

    let result_list = scan_ports(resolved_ip, ports, &scan_config).await;

    report(resolved_ip, &result_list, args.json, args.all)?;
    Ok(())
}
