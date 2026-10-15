use std::io;
use std::net::IpAddr;
use std::time::Duration;

use netscan::report::report;
use netscan::scanner::{Config, scan_ports};
use netscan::target::parse_ports;

fn main() {
    // get user input
    let mut buffer = String::new();
    let mut buffer2 = String::new();

    println!("enter ip and port specification separated by space (e.g. 127.0.0.1 1,2,3,4-10): ");
    io::stdin().read_line(&mut buffer).unwrap();

    let scan_range: Vec<&str> = buffer.split_whitespace().collect();
    let [inputted_ip, port_specifications] = &scan_range[..] else {
        panic!("bad input")
    };

    println!("enter number of threads and timeout duration, separated by space: ");
    io::stdin().read_line(&mut buffer2).unwrap();

    let scanner_config: Vec<&str> = buffer2.split_whitespace().collect();
    let [num_workers, timeout_dur] = &scanner_config[..] else {
        panic!("bad input")
    };

    let ip: IpAddr = inputted_ip.parse().unwrap();

    // ports to be scanned
    let ports: Vec<u16> = match parse_ports(port_specifications) {
        Ok(x) => x,
        Err(e) => panic!("{}", e),
    };

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
