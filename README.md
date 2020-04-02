alog
====

[![Build Status](https://travis-ci.com/thyrc/alog.svg?branch=master)](https://travis-ci.com/thyrc/alog) [![GitHub license](https://img.shields.io/github/license/thyrc/alog.svg)](https://github.com/thyrc/alog/blob/master/LICENSE)

`alog` is a simple log file anonymizer.

About
-----

In fact `alog` just replaces the first *word*[^1] on every line of any input stream with a
customizable string.

So "log file anonymizer" might be a bit of an overstatement, but `alog` can be used to (very
efficiently) replace the $remote_addr part in many access log formats, e.g. Nginx' default
combined log format:

```text
log_format combined '$remote_addr - $remote_user [$time_local] '
                    '"$request" $status $body_bytes_sent '
                    '"$http_referer" "$http_user_agent"';
```

By default any parseable $remote_addr is replaced by it's *localhost* representation,

* any valid IPv4 address is replaced by '127.0.0.1',
* any valid IPv6 address is replaced by '::1' and
* any String (what might be a domain name) with 'localhost'.

Lines without a $remote_addr part will remain unchanged (but can be skipped with
[`alog::Config::set_skip()`] set to `true`).

[^1]: Any first substring *or* (zero width) anchor `^` separated by a `b' '` (Space) from the 
remainder of the line.

Building alog
=============

With version 0.3 `[features]` where added, so that the library crate won't pull unneeded
dependencies anymore.

Commandline Tool
----------------

To build the `alog` commandline tool you now have to expicitly add `--features`.


```shell
cargo build --features alog-cli
```
or 

```shell
cargo build --all-features
```

Usage
=====

Commandline tool
----------------

Run cli-tool with `--help`.

```shell
./target/release/alog --help
```

Library
-------

Calling `run()`

```rust
extern crate alog;

fn main() {
    let mut io_conf = alog::IOConfig::default();
    let mut conf = alog::Config::default();

    io_conf.push_input("/tmp/test.log");
    conf.set_ipv4_value("0.0.0.0");

    alog::run(&conf, &io_conf);
}
```

or `run_raw()`

```rust
extern crate alog;

use std::io::Cursor;

fn main() {
    let mut buffer = vec![];

    alog::run_raw(
        &alog::Config {
            ipv4: "XXX",
            ..Default::default()
        },
        Cursor::new(b"8.8.8.8 test line"),
        &mut buffer,
    );

    assert_eq!(buffer, b"XXX test line");
}
```

Project status
--------------

`alog` started as a replacement for a <10 line Perl script running on an old backup host.
So nothing shiny.. but it helped me learning some Rust (and crates.io) basics.

With version 0.5.6 I consider `alog` feature complete. It doesn't do much, but it does it 
quite well. At some point I might re-use this crate to add parallel processing or try
harder to actually anonymize data. But for now, this is it.

I will still fix bugs when (and if) I find them, so `alog` is now passively-maintained.
