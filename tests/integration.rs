extern crate alog;

use std::io::Cursor;

fn main() {
    let mut buffer = vec![];
    alog::run_raw(&alog::Config{ ipv4:"XXX", ..Default::default() }, Cursor::new(b"8.8.8.8 test line"), &mut buffer);

    assert_eq!(buffer, b"127.0.0.1 test line");
}
