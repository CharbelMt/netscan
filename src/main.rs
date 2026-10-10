use std::io;
use std::net::IpAddr;
use std::time::Duration;

use netscan::report::report;
use netscan::scanner::{Config, scan_ports};

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

    report(ip, &result_list);
}
