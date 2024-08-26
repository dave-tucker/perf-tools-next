# perf-rs

A clean slate implementation of the `perf` command in Rust.

## Why?

The `perf` command is a powerful tool for profiling and tracing applications.
It is is a tool that has a 15 year history and has been developed by many contributors.

There are many useful features in `perf`, but equally there are many seldom used features in `perf` which adds to the projects complexity and maintenance burden.

By reimplementing `perf` in Rust, we can start with a clean slate and only implement the features that are most useful to users. This will allow us to
provide a more focused and user-friendly tool.

## Goals

- [ ] Implement a subset of the `perf` command in Rust
  - [ ] Implement the CLI using `clap`
  - [ ] Implement the TUI using `ratatui`
- [ ] Demonstrate how moving to Rust improves the developer experience and makes
      the codebase easier to maintain and extend.
- [ ] Provide a reference implementation for a userspace tool written in Rust,
      as a peer to the recently added "Rust for Linux" support.
- [ ] Show how safe rust can avoid some commonly reported bugs in `perf`

## Building perf-rs

### Prerequisites

1. Rust
2. To have run `make headers_install` in the Linux kernel source directory

### Building

To build `perf-rs`, you will need to have Rust installed.
You can install Rust by following the instructions at [rustup.rs](https://rustup.rs/).

Once you have Rust installed, you can build `perf-rs` by running:

```sh
cargo build
```

This will build the `perf-rs` binary in the `target/debug` directory.

## Running perf-rs

To run `perf-rs`, you can use the following command:

```sh
cargo run --bin perf-rs -- <perf-rs-args>
```

Should you need to run `perf-rs` with elevated privileges, you can use `sudo`.
Please note you must run `cargo build` before each invocation to ensure your
changes have been incorporated into the binary.

```sh
cargo build && sudo ./target/debug/perf-rs <perf-rs-args>
```
