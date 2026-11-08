use std::io::Write;
use std::net::TcpListener;
use std::time::Duration;

use netscan::scanner::{Config, PortStatus, scan_ports};

#[tokio::test]
async fn test_tcp_listener() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();

    let address = listener.local_addr().unwrap();

    let port = address.port();

    let res = scan_ports(
        address.ip(),
        vec![port],
        &Config::new(1, Duration::from_millis(1000), Duration::from_millis(500)).unwrap(),
    )
    .await;

    assert_eq!(res.len(), 1);
    assert!(matches!(res[0].port_status, PortStatus::Open));

    drop(listener);

    let res2 = scan_ports(
        address.ip(),
        vec![port],
        &Config::new(1, Duration::from_millis(1000), Duration::from_millis(500)).unwrap(),
    )
    .await;

    assert_eq!(res2.len(), 1);
    assert!(matches!(
        res2[0].port_status,
        PortStatus::Closed | PortStatus::TimedOut
    ));
}

#[tokio::test]
async fn test_concurrent_port_scanning() {
    let listener1 = TcpListener::bind("127.0.0.1:0").unwrap();
    let port1 = listener1.local_addr().unwrap().port();

    let listener2 = TcpListener::bind("127.0.0.1:0").unwrap();
    let port2 = listener2.local_addr().unwrap().port();

    // set up a closed port by binding and immediately dropping
    let closed_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let closed_port = closed_listener.local_addr().unwrap().port();
    drop(closed_listener);

    let banner1 = b"Hello from TEST server\r\n";
    // spawn server for listener 1
    let handle1 = std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener1.accept() {
            stream.write_all(banner1).unwrap();
        }
    });

    let mut unsorted_ports = vec![port2, closed_port, port1];
    unsorted_ports.sort();
    unsorted_ports.reverse();
    let total_ports = unsorted_ports.len();

    let config = Config::new(2, Duration::from_millis(500), Duration::from_millis(500)).unwrap();

    let results = scan_ports("127.0.0.1".parse().unwrap(), unsorted_ports, &config).await;

    handle1.join().expect("Server thread 1 panicked");

    assert_eq!(results.len(), total_ports);
    for window in results.windows(2) {
        assert!(
            window[0].port_number < window[1].port_number,
            "Results must be sorted by port: {} came before {}",
            window[0].port_number,
            window[1].port_number
        );
    }

    let expected_banner1 = "Hello from TEST server\\r\\n";

    for res in results {
        if res.port_number == port1 {
            assert!(
                matches!(res.port_status, PortStatus::Open),
                "Port {} should be open",
                res.port_number
            );
            assert_eq!(res.banner.as_deref(), Some(expected_banner1));
        } else if res.port_number == port2 {
            assert!(
                matches!(res.port_status, PortStatus::Open),
                "Port {} should be open",
                res.port_number
            );
            assert_eq!(res.banner, None);
        } else if res.port_number == closed_port {
            assert!(
                matches!(res.port_status, PortStatus::Closed | PortStatus::TimedOut),
                "Port {} should be closed or timed out",
                res.port_number
            );
        } else {
            panic!("{} is an invalid port for this tests", res.port_number)
        }
    }
}
