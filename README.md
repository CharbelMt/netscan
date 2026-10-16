# netscan

A multithreaded TCP connect port scanner and passive banner grabber written in Rust.

`netscan` scans an IPv4 or IPv6 address for open TCP ports, attempts to read a banner from successful connections, and prints a summary of the results. It uses a bounded pool of OS threads and a shared work queue.

## Features

- TCP connect scanning with configurable per-connection timeout
- IPv4 and IPv6 address input
- Individual ports, inclusive ranges, and comma-separated combinations
- Bounded concurrency (1-100 worker threads)
- Passive banner reads on open connections (up to 1,024 bytes)
- Open, closed, timed-out, and error status classification
- Sorted results, with duplicate ports removed
- Optional display of non-open results using `--all`

## Requirements

- Rust toolchain with Cargo (edition 2024 compatible)

## Build and run

From the project directory, build an optimized binary:

```powershell
cargo build --release
```

Run from source:

```powershell
cargo run -- 127.0.0.1
```

Or run the release binary directly on Windows (PowerShell):

```powershell
.\target\release\netscan.exe 127.0.0.1
```

## Usage

```text
netscan [OPTIONS] <IP>
```

The target must be a numeric IPv4 or IPv6 address. All other options are optional.

| Argument / option         | Description                                                                  | Default  |
| ------------------------- | ---------------------------------------------------------------------------- | -------- |
| `<IP>`                    | Target IPv4 or IPv6 address (required)                                       | None     |
| `-p, --ports <PORTS>`     | Comma-separated ports and/or inclusive ranges, e.g. `22,80,100-105`          | `1-1024` |
| `-w, --workers <WORKERS>` | Maximum number of worker threads (1-100)                                     | `100`    |
| `-t, --timeout <TIMEOUT>` | TCP connection timeout in milliseconds; also used as the banner read timeout | `1000`   |
| `--all`                   | Also print closed, timed-out, and errored ports                              | Off      |
| `-h, --help`              | Display command help                                                         | —        |

Examples (PowerShell):

```powershell
# Scan the default ports with default settings
cargo run -- 127.0.0.1

# Scan selected ports and an inclusive range
cargo run -- 127.0.0.1 -p 22,80,100-105 -w 4 -t 1000

# Display every port status
cargo run -- 127.0.0.1 -p 1-10 -w 4 -t 500 --all
```

## Example output

The following output was captured from a scan of localhost. Results depend on the services running and the host's networking configuration.

Command:

```powershell
cargo run -- 127.0.0.1 -p 1-100 -w 10 -t 500
```

Output:

```text
Summary: 0 open, 0 closed, 100 timed out, 0 errors
```

With `--all`:

```powershell
cargo run -- 127.0.0.1 -p 49150-49154 -w 5 -t 1000 --all
```

```text
127.0.0.1:49150 TIMEDOUT
127.0.0.1:49151 TIMEDOUT
127.0.0.1:49152 is OPEN
 Banner: SSH-2.0-netscan-test-server
127.0.0.1:49153 TIMEDOUT
127.0.0.1:49154 TIMEDOUT
Summary: 1 open, 0 closed, 4 timed out, 0 errors
```

By default, individual non-open ports are omitted, but the summary still counts them. Open ports are always printed; any captured banner is displayed beneath the open-port line. Results depend on services running and the host's networking configuration.

## How it works

1. **Parse input:** `clap` parses the target address and CLI options. The port parser expands ranges, sorts ports, and removes duplicates.
2. **Validate configuration:** The scanner rejects zero workers, more than 100 workers, and zero timeout. The effective number of workers is capped at the number of ports.
3. **Distribute work:** Ports are stored in a shared `Arc<Mutex<Vec<u16>>>`. Each worker locks the queue only long enough to remove a port, then releases the lock before network I/O.
4. **Connect and read:** Each worker calls `TcpStream::connect_timeout`. A successful connection is classified as open, after which the scanner attempts one passive read with a read timeout. Connection refusal is classified as closed; timeout and other I/O errors have separate statuses.
5. **Collect and report:** Each worker returns its own results. The main scanning function joins the threads, merges and sorts the results by port, and the reporting layer prints the output and status totals.

The core is organized into `scanner.rs` (configuration and worker pool), `target.rs` (port parsing), `banner.rs` (connections and passive reads), and `report.rs` (terminal output), with a CLI entry point in `main.rs`.

## Tests and checks

Run these commands from the project directory (PowerShell):

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

The test suite includes port-parsing unit tests and a local TCP listener integration test. The listener test checks that a listening port is detected as open and that the port is no longer reported as open after the listener is dropped.

## Known limitations

- **TCP connect only:** No UDP, SYN/raw-socket scanning, or packet capture.
- **Numeric IP addresses only:** Hostname/DNS resolution is not implemented.
- **Passive banners only:** The scanner does not send protocol-specific probes. Services that wait for a client request may produce no banner.
- **Limited banner read:** It performs a single read of up to 1,024 bytes. Read errors, including read timeouts, result in no banner while the successful TCP connection remains classified as open.
- **Blocking OS threads:** Concurrency uses a bounded worker pool, not async I/O. The maximum configured worker count is 100.
- **CLI error handling:** Invalid port specifications or scanner configuration currently cause a panic rather than a polished command-line error.
- **Environment-dependent results:** Firewall rules, network configuration, and remote behavior can affect whether connections are refused, time out, or fail with another I/O error.
- **No service identification:** A banner is raw text decoded with lossy UTF-8 conversion, not a verified service or version fingerprint.

## Authorized use

Only scan hosts and networks you own or are explicitly authorized to test. Scanning third-party systems without permission may violate policies or applicable law.
