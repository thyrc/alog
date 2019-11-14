#!/bin/sh

RUSTFLAGS='-C target-feature=+crt-static' cargo build --release
strip --strip-unneeded -R .comment target/release/anon
