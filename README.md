# netscan

[![CI](https://github.com/CharbelMt/netscan/actions/workflows/ci.yml/badge.svg)](https://github.com/CharbelMt/netscan/actions/workflows/ci.yml)

A command-line TCP port scanner and banner grabber written in Rust.

> **Status:** work in progress. Not yet usable.

## Ethical use

Only scan hosts you own or have explicit permission to scan. Good targets
for testing: `localhost`, `scanme.nmap.org` (provided by the Nmap project for
this purpose), or your own lab machines. Unauthorized scanning may be illegal
in your jurisdiction.

## Planned features

- TCP connect scan on a single port or a range
- Banner grabbing on open ports
- Concurrent scanning with Tokio
- Text and JSON output

## License

MIT
