alog
====

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

[^1]: Any first substring separated by a `b' '` (Space) from the remainder of the line.

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
    $ cargo build --all-features
```

Project status
--------------

`alog` started as a replacement for a <10 line Perl script running on an old backup host.
So nothing shiny.. but it now serves as a journeyman's piece.

Along the way I expect the API to change quite a bit. I will update the README / Documentation
when things quiet down a bit, but util then (maybe a 1.x release) I will add features, move parts
around and fix bugs when (and if) I find them.
