#!/bin/sh

if [[ "x$1" == "x--static" ]]; then
    RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --all-features
else
    cargo build --release --all-features
fi

strip --strip-unneeded -R .comment target/release/alog
