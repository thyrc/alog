use std::env;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};
use std::net;
use std::process;

fn replace_remote_address<W: Write>(r: Box<dyn BufRead>, mut w: W) -> Result<(), Box<dyn Error>> {
    for buffer in r.lines() {
        match buffer {
            Ok(line) => {
                let v: Vec<&str> = line.splitn(2, ' ').collect();
                match v.len() {
                    1 => writeln!(&mut w, "{}", line)?,
                    2 => {
                        let (remote_addr, log) = (&v[0], &v[1]);
                        match remote_addr.parse::<net::Ipv4Addr>() {
                            Ok(_) => writeln!(&mut w, "127.0.0.1 {}", log)?,
                            Err(_) => match remote_addr.parse::<net::Ipv6Addr>() {
                                Ok(_) => writeln!(&mut w, "::1 {}", log)?,
                                Err(_) => writeln!(&mut w, "localhost {}", log)?,
                            },
                        }
                    }
                    _ => writeln!(&mut w, "{}", line)?,
                };
            },
            Err(e) => { eprintln!("Error reading from reader: {}", e);
                        process::exit(1);
            },
        }
    }
    Ok(())
}

pub fn run() {
    let args: Vec<_> = env::args().collect();
    let mut output = std::io::stdout();
    if args.len() > 1 {
        for arg in &args[1..] {
            let f = OpenOptions::new().read(true).open(arg);
            let f = match f {
                Ok(file) => file,
                Err(_) => { eprintln!("Error reading file '{}'.", arg);
                            process::exit(1);
                }
            };
            let reader: Box<dyn BufRead> = Box::new(BufReader::new(f));
            match replace_remote_address(reader, &mut output) {
                Ok(_) => (),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
    } else {
        let reader: Box<dyn BufRead> = Box::new(BufReader::new(io::stdin()));
        match replace_remote_address(reader, &mut output) {
            Ok(_) => (),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_ipv4() {
        use std::io::Cursor;
        let mut buff = Cursor::new(vec![0; 512]);
        let log = Box::new("8.8.8.8 - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes());
        let local_log = "127.0.0.1 - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes();

        replace_remote_address(log, &mut buff).unwrap();
        assert!(&buff.get_ref().starts_with(&local_log));
    }

    #[test]
    fn replace_ipv6() {
        use std::io::Cursor;
        let mut buff = Cursor::new(vec![0; 512]);
        let log = Box::new("2a00:1450:4001:81b::2004 - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes());
        let local_log = "::1 - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes();

        replace_remote_address(log, &mut buff).unwrap();
        assert!(&buff.get_ref().starts_with(&local_log));
    }

    #[test]
    fn replace_hostname() {
        use std::io::Cursor;
        let mut buff = Cursor::new(vec![0; 512]);
        let log = Box::new("google.com - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes());
        let local_log = "localhost - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes();

        replace_remote_address(log, &mut buff).unwrap();
        assert!(&buff.get_ref().starts_with(&local_log));
    }
}
