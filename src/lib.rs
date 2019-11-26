use std::env;
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};
use std::net;
use std::process;

fn replace_remote_address<W: Write>(reader: Box<dyn BufRead>, mut w: W) {
    for buffer in reader.lines() {
        match buffer {
            Ok(line) => {
                let v: Vec<&str> = line.splitn(2, ' ').collect();
                match v.len() {
                    1 => writeln!(&mut w, "{}", line).unwrap(),
                    2 => {
                        let (remote_addr, log) = (&v[0], &v[1]);
                        match remote_addr.parse::<net::Ipv4Addr>() {
                            Ok(_) => writeln!(&mut w, "127.0.0.1 {}", log).unwrap(), 
                            Err(_) => match remote_addr.parse::<net::Ipv6Addr>() {
                                Ok(_) => writeln!(&mut w, "::1 {}", log).unwrap(),
                                Err(_) => writeln!(&mut w, "localhost {}", log).unwrap(),
                            },
                        }
                    }
                    _ => writeln!(&mut w, "{}", line).unwrap(),
                };
            }
            Err(err) => { eprintln!("Error reading from reader: {}", err);
                          process::exit(1);
            },
        }
    }
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
            replace_remote_address(reader, &mut output);
        }
    } else {
        let reader: Box<dyn BufRead> = Box::new(BufReader::new(io::stdin()));
        replace_remote_address(reader, &mut output);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_ipv4() {
        use std::io::Cursor;
        let mut buff = Cursor::new(vec![0; 512]);
        let log = Box::new("8.8.8.8 - - [21/Oct/2019:12:27:25 +0200] \"GET / HTTP/1.1\" 200 46948 \"-\" \"Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)\" \"-\"".as_bytes());

        replace_remote_address(log, &mut buff);
        assert_eq!(&buff.get_ref()[..10], &[49, 50, 55, 46, 48, 46, 48, 46, 49, 32]);
    }

    #[test]
    fn replace_ipv6() {
        use std::io::Cursor;
        let mut buff = Cursor::new(vec![0; 512]);
        let log = Box::new("2a00:1450:4001:81b::2004 - - [21/Oct/2019:12:27:25 +0200] \"GET / HTTP/1.1\" 200 46948 \"-\" \"Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)\" \"-\"".as_bytes());

        replace_remote_address(log, &mut buff);
        assert_eq!(&buff.get_ref()[..4], &[58, 58, 49, 32]);
    }

    #[test]
    fn replace_hostname() {
        use std::io::Cursor;
        let mut buff = Cursor::new(vec![0; 512]);
        let log = Box::new("www.google.com - - [21/Oct/2019:12:27:25 +0200] \"GET / HTTP/1.1\" 200 46948 \"-\" \"Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)\" \"-\"".as_bytes());

        replace_remote_address(log, &mut buff);
        assert_eq!(&buff.get_ref()[..10], &[108, 111, 99, 97, 108, 104, 111, 115, 116, 32]);
    }
}
