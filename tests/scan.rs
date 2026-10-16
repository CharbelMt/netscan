use std::net::TcpListener;
use std::time::Duration;

use netscan::scanner::{Config, PortStatus, scan_ports};

#[test]
fn test_tcp_listener() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();

    let address = listener.local_addr().unwrap();

    let port = address.port();

    let res = scan_ports(
        address.ip(),
        vec![port],
        &Config::new(1, Duration::from_millis(1000), Duration::from_millis(500)).unwrap(),
    );

    assert_eq!(res.len(), 1);
    assert!(matches!(res[0].port_status, PortStatus::Open));

    drop(listener);

    let res2 = scan_ports(
        address.ip(),
        vec![port],
        &Config::new(1, Duration::from_millis(1000), Duration::from_millis(500)).unwrap(),
    );

    assert_eq!(res2.len(), 1);
    assert!(matches!(
        res2[0].port_status,
        PortStatus::Closed | PortStatus::TimedOut
    ));
}
