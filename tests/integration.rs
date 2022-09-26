use std::io::Cursor;

#[test]
fn main() {
    let mut buffer = vec![];
    alog::run_raw(
        &alog::Config {
            ipv4: "XXX",
            ..Default::default()
        },
        Cursor::new(b"8.8.8.8 test line"),
        &mut buffer,
    )
    .unwrap();

    assert_eq!(buffer, b"XXX test line");
}
