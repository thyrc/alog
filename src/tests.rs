use super::*;

#[test]
fn run_raw_function() {
    use std::io::Cursor;
    let line = Cursor::new(b"8.8.8.8 XxX");
    let mut buffer = vec![];

    run_raw(&Config::default(), line, &mut buffer).unwrap();
    assert_eq!(buffer, b"127.0.0.1 XxX");
}

#[test]
fn replace_ipv4() {
    use std::io::Cursor;
    let mut buffer = Cursor::new(vec![]);
    let log = Box::new("8.8.8.8 - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes());
    let local_log = "127.0.0.1 - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes();

    replace_remote_address(&Config::default(), log, &mut buffer).unwrap();
    assert_eq!(&buffer.into_inner(), &local_log);
}

#[test]
fn replace_ipv6() {
    use std::io::Cursor;
    let mut buffer = Cursor::new(vec![]);
    let log = Box::new("2a00:1450:4001:81b::2004 - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes());
    let local_log = "::1 - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes();

    replace_remote_address(&Config::default(), log, &mut buffer).unwrap();
    assert_eq!(&buffer.into_inner(), &local_log);
}

#[test]
fn replace_host() {
    use std::io::Cursor;
    let mut buffer = Cursor::new(vec![]);
    let log = Box::new("google.com - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes());
    let local_log = "localhost - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes();

    replace_remote_address(&Config::default(), log, &mut buffer).unwrap();
    assert_eq!(&buffer.into_inner(), &local_log);
}

#[test]
fn clear_authuser() {
    use std::io::Cursor;
    let mut buffer = Cursor::new(vec![]);
    let log = Box::new("8.8.8.8 - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes());
    let local_log = "127.0.0.1 - - [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes();

    let mut conf = Config::default();
    conf.set_authuser(true);

    replace_remote_address(&conf, log, &mut buffer).unwrap();
    assert_eq!(&buffer.into_inner(), &local_log);
}

#[test]
fn replace_custom_ipv4() {
    use std::io::Cursor;
    let mut buffer = Cursor::new(vec![]);
    let log = Box::new("8.8.8.8 - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes());
    let local_log = "custom_ipv4 - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes();

    let mut conf = Config::default();
    conf.set_ipv4_value("custom_ipv4");

    replace_remote_address(&conf, log, &mut buffer).unwrap();
    assert_eq!(&buffer.into_inner(), &local_log);
}

#[test]
fn replace_custom_ipv6() {
    use std::io::Cursor;
    let mut buffer = Cursor::new(vec![]);
    let log = Box::new("2a00:1450:4001:81b::2004 - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes());
    let local_log = "custom_ipv6 - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes();

    let mut conf = Config::default();
    conf.set_ipv6_value("custom_ipv6");

    replace_remote_address(&conf, log, &mut buffer).unwrap();
    assert_eq!(&buffer.into_inner(), &local_log);
}

#[test]
fn replace_custom_host() {
    use std::io::Cursor;
    let mut buffer = Cursor::new(vec![]);
    let log = Box::new("google.com - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes());
    let local_log = "custom_host - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes();

    let mut conf = Config::default();
    conf.set_host_value("custom_host");

    replace_remote_address(&conf, log, &mut buffer).unwrap();
    assert_eq!(&buffer.into_inner(), &local_log);
}

#[test]
fn notrim_and_auth() {
    use std::io::Cursor;
    let mut buffer = Cursor::new(vec![]);
    let log = Box::new(" 8.8.8.8 - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes());
    let local_log = "localhost - - [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes();

    let mut conf = Config::default();
    conf.set_trim(false);
    conf.set_authuser(true);

    replace_remote_address(&conf, log, &mut buffer).unwrap();
    assert_eq!(&buffer.into_inner(), &local_log);
}

#[test]
fn invalid_utf8() {
    use std::io::Cursor;
    let line = Cursor::new(vec![0, 159, 146, 150, 32, 88, 120, 88]);
    let mut buffer = vec![];

    run_raw(&Config::default(), line, &mut buffer).unwrap();
    assert_eq!(buffer, b"localhost XxX");
}

#[test]
fn valid_utf8() {
    use std::io::Cursor;
    let line = Cursor::new(vec![240, 159, 146, 150, 32, 88, 120, 88]);
    let mut buffer = vec![];

    run_raw(&Config::default(), line, &mut buffer).unwrap();
    assert_eq!(buffer, b"localhost XxX");
}

#[test]
fn bmsearch() {
    let hay = b"8.8.8.8 - frank proxy 8.8.8.8 direct 8.8.8.8";
    let mat = hay.bmsearch(b"8.8.8.8");

    assert_eq!(Some(vec![0, 22, 37]), mat);
}

#[test]
fn kmpsearch() {
    let hay = b"8.8.8.8 - frank proxy 8.8.8.8 direct 8.8.8.8";
    let mat = hay.bmsearch(b"8.8.8.8");

    assert_eq!(Some(vec![0, 22, 37]), mat);
}

#[test]
fn research() {
    let hay = b"8.8.8.8 - frank proxy 8.8.8.8 direct 8.8.8.8";
    let mat = hay.bmsearch(b"8.8.8.8");

    assert_eq!(Some(vec![0, 22, 37]), mat);
}

#[test]
fn windowsearch() {
    let hay = b"8.8.8.8 - frank proxy 8.8.8.8 direct 8.8.8.8";
    let mat = hay.windowsearch(b"8.8.8.8");

    assert_eq!(Some(vec![0, 22, 37]), mat);
}

#[test]
fn thorough() {
    use std::io::Cursor;
    let mut buffer = Cursor::new(vec![]);
    let log = Box::new("8.8.8.8 - frank proxy 8.8.8.8 direct 8.8.8.8".as_bytes());
    let local_log = b"127.0.0.1 - frank proxy 127.0.0.1 direct 127.0.0.1";

    let mut conf = Config::default();
    conf.set_thorough(true);

    replace_remote_address(&conf, log, &mut buffer).unwrap();
    assert_eq!(&buffer.into_inner(), &local_log);
}

#[test]
fn thorough_non_overlapping() {
    use std::io::Cursor;
    let mut buffer = Cursor::new(vec![]);
    let log = Box::new("8.8.8.8 - frank proxy 8.8.8.8.8.8".as_bytes());
    let local_log = b"127.0.0.1 - frank proxy 127.0.0.1.8.8";

    let mut conf = Config::default();
    conf.set_thorough(true);

    replace_remote_address(&conf, log, &mut buffer).unwrap();
    assert_eq!(&buffer.into_inner(), &local_log);
}
