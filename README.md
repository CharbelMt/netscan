# netscan

A multithreaded TCP connect port scanner and passive banner grabber written in Rust.

`netscan` scans an IPv4 address, IPv6 address, or hostname for TCP ports, optionally reads banners from successful connections, and reports results as human-readable text or JSON. It uses a bounded pool of OS threads and a shared work queue; it does **not** use Tokio or raw sockets.

> **Authorized use only:** Scan hosts you own or have explicit permission to test. For local testing, use `127.0.0.1` or a test VM.

## Features

- TCP connect scanning with configurable connection timeout
- IPv4, IPv6, and hostname input (resolved once before scanning)
- Individual ports, inclusive ranges, and comma-separated port specifications
- Bounded worker pool (1–100 OS threads)
- Passive banner grabbing with a separate read timeout and a 1,024-byte read limit
- Lossy UTF-8 decoding and escaping of terminal control characters in banners
- Open, closed, timed-out, and error status classification
- Sorted scan results and deduplicated port specifications
- Human-readable output by default; pretty-printed JSON via `--json`
- Optional inclusion of non-open ports via `--all`
- Structured validation errors using `thiserror` and application-level error propagation using `anyhow`

## Requirements

- Rust and Cargo with support for Rust edition 2024

## Build

From the project root:

```powershell
cargo build --release
```

Run from source:

```powershell
cargo run -- 127.0.0.1
```

Or run the Windows release binary:

```powershell
.\target\release\netscan.exe 127.0.0.1
```

## CLI usage

```text
netscan [OPTIONS] <IP>
```

`<IP>` is the positional target argument; it accepts an IPv4 address, IPv6 address, or resolvable hostname.

| Argument / option                       | Description                                                     | Default  |
| --------------------------------------- | --------------------------------------------------------------- | -------- |
| `<IP>`                                  | Target IP address or hostname (required)                        | —        |
| `-p, --ports <PORTS>`                   | Ports and inclusive ranges, e.g. `22,80,100-105`                | `1-1024` |
| `-w, --workers <WORKERS>`               | Maximum scanning workers (1–100)                                | `100`    |
| `-t, --timeout <TIMEOUT>`               | TCP connection timeout, milliseconds                            | `1000`   |
| `-b, --banner-timeout <BANNER_TIMEOUT>` | Banner read timeout, milliseconds                               | `500`    |
| `--json`                                | Output a single JSON document instead of terminal text          | Off      |
| `--all`                                 | Include closed, timed-out, and errored ports in detailed output | Off      |
| `-h, --help`                            | Show command-line help                                          | —        |

### Examples (PowerShell)

Scan a few local ports:

```powershell
cargo run -- 127.0.0.1 -p 22,80,443
```

Scan a hostname with a limited worker count:

```powershell
cargo run -- localhost -p 8000-8010 -w 4 -t 1500
```

Display every port status:

```powershell
cargo run -- 127.0.0.1 -p 1-10 --all
```

Produce JSON, including non-open ports:

```powershell
cargo run --quiet -- 127.0.0.1 -p 1-10 --json --all
```

Validate the JSON in PowerShell:

```powershell
cargo run --quiet -- 127.0.0.1 -p 1-10 --json | ConvertFrom-Json
```

## Output

By default, the CLI prints open ports, any banners it receives, and a summary of **all** scanned ports. With `--all`, it also prints individual closed, timed-out, and errored ports.

With `--json`, the CLI emits a JSON object with the resolved target IP, status totals, and a `results` array. `--all` controls which ports appear in `results`, **not** which ports contribute to the totals.

Illustrative JSON structure (statuses depend on your environment):

```json
{
	"ip": "127.0.0.1",
	"open": 1,
	"closed": 1,
	"timedout": 0,
	"errors": 0,
	"results": [
		{
			"port": 8080,
			"status": "Open",
			"banner": null,
			"error": null
		}
	]
}
```

The example shows default filtering: one closed port contributes to the `closed` count but is absent from `results`. With `--all`, that port would also appear in the array. Status strings currently use `Open`, `Closed`, `TimedOut`, and `Error`. An error entry can include its I/O error message in the `error` field.

## How it works

1. **Parse arguments:** `clap` reads the target, port specification, timeouts, worker count, and output flags.
2. **Resolve target:** `target.rs` accepts numeric IPs directly or resolves a hostname using `ToSocketAddrs`, selecting one address for the scan.
3. **Validate input:** Port specifications are expanded, sorted, and deduplicated. Scanner configuration validates worker limits and nonzero timeouts.
4. **Distribute ports:** The scanner shares a `Vec<u16>` work queue through `Arc<Mutex<_>>`. Each worker removes a port while holding the lock, then releases it before performing network I/O.
5. **Connect and classify:** `TcpStream::connect_timeout` attempts a TCP connection. Results are classified as open, closed, timed out, or another I/O error.
6. **Read optional banner:** After a successful connection, `banner.rs` performs a passive, timeout-limited read and sanitizes any received text. No protocol-specific request is sent.
7. **Collect and report:** Worker-local results are merged and sorted by port. `report.rs` calculates totals, applies `--all` filtering, and prints terminal text or serialized JSON.

### Source layout

| File             | Responsibility                                                |
| ---------------- | ------------------------------------------------------------- |
| `src/main.rs`    | Clap CLI, orchestration, application error propagation        |
| `src/lib.rs`     | Library module exports                                        |
| `src/scanner.rs` | Scanner configuration, worker pool, TCP status classification |
| `src/target.rs`  | Port parsing and hostname/IP resolution                       |
| `src/banner.rs`  | Passive banner reads and sanitization                         |
| `src/report.rs`  | Summary counts, result filtering, text/JSON reporting         |

The scanner returns structured `ScanResult` values rather than printing directly. The reporting layer handles presentation separately.

## Development checks

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

For a smoke test, start a local TCP listener on a known port and verify that it is reported as open in both default and JSON modes. Banner contents are only present if the service sends data without requiring a request.

## Limitations

- **TCP connect only:** No UDP, SYN/raw-socket scanning, or packet capture.
- **Passive banners:** Services that wait for a client request may not return a banner.
- **Single banner read:** At most 1,024 bytes are read per open connection; a banner timeout does not make the port closed.
- **One resolved address:** Hostnames are resolved once and one returned IP address is scanned, not every DNS result.
- **Blocking concurrency:** The scanner uses a bounded OS-thread worker pool, not async I/O.
- **Environment-dependent results:** Firewall rules, local networking, and remote behavior affect whether a port is refused, times out, or produces another error.
- **No service fingerprinting:** A received banner is unverified text, not a confirmed service or version identification.

## Roadmap

**v0.2.0:** Hostname resolution, banner sanitization, structured errors, and JSON output, built on the existing thread-pool scanner.

**Next:** Further scanner robustness and integration tests, followed by consideration of Tokio once the blocking implementation is well understood and verified.

## License

MIT, See LICENSE
