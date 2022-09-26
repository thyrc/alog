# alog

[![Crates.io](https://img.shields.io/crates/v/alog.svg)](https://crates.io/crates/alog)
[![CI](https://github.com/thyrc/alog/workflows/Rust/badge.svg)](https://github.com/thyrc/alog/actions?query=workflow%3ARust)
[![Documentation](https://docs.rs/alog/badge.svg)](https://docs.rs/alog)
[![GitHub license](https://img.shields.io/github/license/thyrc/alog.svg)](https://github.com/thyrc/alog/blob/master/LICENSE)

`alog` is a simple log file anonymizer.

## About

In fact by default `alog` just replaces the first *word* on every line of any input stream
with a customizable string.

With version 0.6

* you can (at a substantial cost of CPU cycles) replace the `$remote_user` with '-' as well and
* by default any leading Spaces or Tabs will be removed from every line before replacing any `$remote_addr`.

So "log file anonymizer" might be a bit of an overstatement, but `alog` can be used to (very
efficiently) replace the `$remote_addr` part in many access log formats, e.g. Nginx' default
combined log format:

```text
log_format combined '$remote_addr - $remote_user [$time_local] '
                    '"$request" $status $body_bytes_sent '
                    '"$http_referer" "$http_user_agent"';
```

By default any parseable `$remote_addr` is replaced by it's *localhost* representation,

* any valid IPv4 address is replaced by *127.0.0.1*,
* any valid IPv6 address is replaced by *::1* and
* any String (what might be a domain name) with *localhost*.

Lines without a `$remote_addr` part will remain unchanged (but can be skipped).

## Building alog

With version 0.3 `[features]` where added, so that the library crate won't pull unneeded
dependencies anymore.

### Commandline Tool

To build the `alog` commandline tool you now have to expicitly add `--features`.

```shell
cargo build --features alog-cli
```

or

```shell
cargo build --all-features
```

## Usage

### Commandline tool

Run cli-tool with `--help`.

```shell
./target/release/alog --help
```

### Library

Calling `run()`

```rust
fn main() {
    let mut io_conf = alog::IOConfig::default();
    let mut conf = alog::Config::default();

    io_conf.push_input("/tmp/test.log");
    conf.set_ipv4_value("0.0.0.0");

    if let Err(e) = alog::run(&conf, &io_conf) {
        eprintln!("{}", e);
    }

}
```

or `run_raw()`

```rust
use std::io::Cursor;

fn main() {
    let mut buffer = vec![];

    if let Err(e) = alog::run_raw(
        &alog::Config {
            ipv4: "XXX",
            ..Default::default()
        },
        Cursor::new(b"8.8.8.8 test line"),
        &mut buffer,
    ) {
        eprintln!("{}", e);
    }

    assert_eq!(buffer, b"XXX test line");
}
```

## About `Config::authuser`

With version 0.6 `alog` can be used to replace the `$remote_user` field with '-', but this
feature comes with a couple of peculiarities.

This feature should work fine with standard Common / Combined Log formatted files, but...

* There will be a significant hit on performance (synthetic benchmarking suggests ~625MB/s
  instead of ~1100MB/s on my machine, but still better than Perl's ~115MB/s ;)
* Used with `Config::trim` set to `false` and malformatted files the performance hit will be
  even worse and removal of the `$remote_user` field will fail altogether if no `$time_local`
  field is found.
* The `$time_local` field is expected to start with '[' followed by a decimal number. E.g.:
  "[10/Oct/2000:13:55:36 -0700]"
* There is an optimization in place to reduce the performance hit with real-life log files,
  but this leads to `$remote_user` fields *starting* with "- [" _not_ being replaced! So in
  
  `"8.8.8.8 - - [frank] [10/Oct/2000:13:55:36 -0700] GET /apache_pb.gif HTTP/1.0 200 2326"`

  "frank" will still be "frank". This optimization can be disabled.

## Project status

`alog` started as a replacement for a <10 line Perl script running on an old backup host.
So nothing shiny.. but it helped me learning some Rust (and crates.io) basics.

With version 0.6 `alog` is feature complete. It doesn't do much, but it does it quite well.
At some point I might re-use this crate and try harder to actually anonymize data. But for
now, this is it.

I will still fix bugs when (and if) I find them, so `alog` is now passively-maintained.
