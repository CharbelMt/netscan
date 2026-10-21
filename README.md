# netscan

[![CI](https://github.com/<you>/netscan/actions/workflows/ci.yml/badge.svg)](https://github.com/<you>/netscan/actions/workflows/ci.yml)

A multithreaded TCP connect port scanner and passive banner grabber, written in Rust.

`netscan` takes an IPv4 address, IPv6 address, or hostname, tries to open TCP connections to a set of ports, classifies each result (open, closed, timed out, or error), reads a service banner from open ports when the service sends one, and reports the results as terminal text or JSON. It was built as a learning project to practice systems-level Rust and to make the mechanics of a connect scan explicit: concurrency, timeouts, error handling, and treating network data as untrusted input.

> **Authorized use only.** Scan hosts and networks you own or have explicit permission to test. Good targets are `127.0.0.1`, your own lab machines, or a VM you control. Unauthorized scanning may violate policy or law in your jurisdiction.

## Features

- TCP connect scanning with a configurable connection timeout
- IPv4, IPv6, and hostname targets (a hostname is resolved once, before scanning)
- Single ports, inclusive ranges, and comma-separated combinations (`22,80,100-105`)
- Bounded worker pool of 1 to 100 OS threads
- Passive banner grabbing with its own read timeout and a 1,024-byte read limit
- Banners are treated as untrusted: control characters are escaped before they reach the terminal
- Open, closed, timed-out, and error classification
- Text output by default, or a single JSON document with `--json`
- Typed validation errors (`thiserror`) and clean error exits (`anyhow`)

## Build

Requires a Rust toolchain that supports edition 2024.

```bash
cargo build --release
./target/release/netscan 127.0.0.1
```

On Windows (PowerShell):

```powershell
cargo build --release
.\target\release\netscan.exe 127.0.0.1
```

Or run from source: `cargo run -- 127.0.0.1`

## Usage

```text
netscan [OPTIONS] <TARGET>
```

| Option | Description | Default |
| --- | --- | --- |
| `<TARGET>` | IPv4 address, IPv6 address, or hostname | required |
| `-p, --ports <PORTS>` | Ports and inclusive ranges, e.g. `22,80,100-105` | `1-1024` |
| `-w, --workers <N>` | Maximum number of worker threads (1 to 100) | `100` |
| `-t, --timeout <MS>` | TCP connection timeout in milliseconds | `1000` |
| `-b, --banner-timeout <MS>` | Banner read timeout in milliseconds | `500` |
| `--json` | Print one JSON document instead of text | off |
| `--all` | Also list closed, timed-out, and errored ports | off |
| `-h, --help` | Show help | |

Examples:

```bash
# Default scan of the first 1024 ports
netscan 127.0.0.1

# Specific ports and a range, fewer workers
netscan 127.0.0.1 -p 22,80,443,8000-8010 -w 20

# A hostname, showing every port status
netscan localhost -p 1-100 --all

# JSON, piped to another tool
netscan 127.0.0.1 -p 1-1024 --json | jq '.results'
```

## Output

By default the scanner prints each open port (with its banner, if one was received) and a summary that counts every scanned port.

<!-- Replace this block with real output from your own run -->

```text
127.0.0.1:8000 is OPEN
127.0.0.1:9000 is OPEN
  Banner: SSH-2.0-test
Summary: 2 open, 1 closed, 0 timed out, 0 errors
```

With `--all`, closed, timed-out, and errored ports are listed too. The summary is the same either way.

### JSON

`--json` prints exactly one JSON document to stdout and nothing else. Errors go to stderr with a non-zero exit code, so the output is safe to pipe into other tools.

<!-- Replace this block with real output from your own run -->

```json
{
  "target": "127.0.0.1",
  "ip": "127.0.0.1",
  "open": 2,
  "closed": 1,
  "timed_out": 0,
  "errors": 0,
  "results": [
    { "port": 8000, "status": "open", "banner": null, "error": null },
    { "port": 9000, "status": "open", "banner": "SSH-2.0-test", "error": null }
  ]
}
```

- `target` is what you typed and `ip` is the address that was scanned.
- The totals always include every scanned port. `--all` only controls which ports appear in `results`.
- `status` is one of `open`, `closed`, `timed_out`, or `error`. An `error` entry carries the I/O error message in `error`.
- Banners appear in JSON as the received text. Serde escapes the JSON string itself. Escaping for the terminal is applied only to text output.

## How it works

### What a connect scan observes

A connect scan asks the operating system to complete a normal TCP handshake with each port. The result depends on how the target answers:

| What happens | Reported as |
| --- | --- |
| Handshake completes (SYN, SYN-ACK, ACK) | `open` |
| Target replies with a reset (RST) | `closed` |
| No reply before the timeout | `timed_out` (often a firewall silently dropping packets, or a host that is down) |
| Another failure, such as network unreachable | `error` |

Because it uses ordinary sockets, it needs no raw-socket access or elevated privileges. The trade-off is that every probe completes a full connection, which is slower than a half-open SYN scan and shows up in the target's logs.

### Pipeline

1. **Parse arguments.** `clap` reads the target, port specification, timeouts, worker count, and output flags.
2. **Validate input.** The port specification is expanded, sorted, and deduplicated. Configuration rejects zero workers, more than 100 workers, and zero timeouts.
3. **Resolve the target.** Numeric addresses are used directly. A hostname is resolved once with `ToSocketAddrs` and one returned address is scanned.
4. **Distribute ports.** Ports sit in a shared `Arc<Mutex<Vec<u16>>>` work queue. Each worker locks it only long enough to pop one port, then releases the lock before any network I/O.
5. **Connect and classify.** `TcpStream::connect_timeout` attempts the connection and the result is mapped to a status.
6. **Read a banner.** After a successful connection, one passive read runs under the banner timeout. Nothing is sent to the service.
7. **Collect and report.** Each worker returns its own results. The results are joined, merged, and sorted by port, and the reporting layer prints text or JSON.

The scanner returns structured `ScanResult` values and never prints. Presentation lives entirely in `report.rs`.

## Design decisions

- **Bounded worker pool instead of a thread per port.** The first version spawned one OS thread per port. That does not scale: large ranges run into thread and open-file limits. A fixed pool pulling from a shared queue caps concurrency and keeps the design simple.
- **Per-worker result vectors.** Each worker collects results privately and hands them back through `join()`. The only shared state is the work queue, and its lock is held only for a single `pop`.
- **Passive banners.** The scanner connects and listens. This works for services that speak first (SSH, FTP, SMTP) and produces nothing for services that wait for a request (plain HTTP). Sending protocol probes is out of scope.
- **Separate connect and banner timeouts.** A silent open port would otherwise hold a worker for the full connect timeout. The banner read has its own, shorter timeout.
- **Banners are untrusted input.** The text comes from a host you do not control. Control characters, including terminal escape sequences, are escaped before printing so a hostile service cannot rewrite the terminal output or forge extra result lines.
- **Typed errors.** Port parsing and configuration return typed errors, and `main` propagates them with `anyhow`, so bad input produces a one-line message and exit code 1 instead of a panic.
- **JSON built from separate output types.** `io::Error` cannot be serialized, so the report converts scan results into dedicated serializable structs and the scanner itself stays free of serde.

## Security considerations

- **Authorization.** Scanning systems you do not own or have permission to test may be illegal. Keep tests to localhost and lab machines.
- **Detectability.** Connect scans complete full connections and are visible in service and firewall logs.
- **Untrusted data.** Banners are attacker-controlled text. They are escaped for terminal output and never executed or interpreted.
- **No elevated privileges.** The tool uses ordinary TCP sockets only.

## Platform notes

On Windows, a refused connection to a local port can take longer than the default 1000 ms timeout, so closed ports may be reported as `timed_out`. If you see this, raise the timeout (for example `-t 3000`).

## Project layout

| File | Responsibility |
| --- | --- |
| `src/main.rs` | CLI definition and orchestration |
| `src/lib.rs` | Library module exports |
| `src/scanner.rs` | Configuration, worker pool, TCP status classification |
| `src/target.rs` | Port parsing and hostname/IP resolution |
| `src/banner.rs` | Passive banner reads |
| `src/report.rs` | Counting, filtering, and text/JSON output |
| `tests/scan.rs` | Loopback integration test |

## Development

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

The test suite covers the port parser, the banner sanitizer, and a loopback integration test that scans a listener it creates. CI runs the same checks on every push.

## Limitations

- **TCP connect only.** No UDP, SYN, or raw-socket scanning, and no packet capture.
- **Passive banners only.** Services that wait for a client request return no banner.
- **One banner read.** At most 1,024 bytes are read per open connection.
- **One resolved address.** A hostname resolves once and a single address is scanned, not every DNS result.
- **Blocking threads.** Concurrency uses OS threads, not async I/O, so very high concurrency is limited by thread and file-descriptor limits.
- **Worker panics abort the scan.** A panic inside a worker thread propagates when results are joined.
- **Environment-dependent results.** Firewalls and network conditions affect whether a port is refused, times out, or errors.
- **No service fingerprinting.** A banner is unverified text, not a confirmed service or version.

## Roadmap

- **v0.3.0 (planned):** replace the thread pool with Tokio tasks and a semaphore, keep the same CLI and tests, and publish a before-and-after benchmark.

## License

MIT. See [LICENSE](LICENSE).
