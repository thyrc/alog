use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::net;
use std::process;

pub struct Config<'a> {
    ipv4_replacement: &'a str,
    ipv6_replacement: &'a str,
    hostname_replacement: &'a str,

    input_files: Vec<String>,
}

impl<'a> Config<'a> {
    pub fn new() -> Config<'a> {
        let ipv4_replacement = "127.0.0.1";
        let ipv6_replacement = "::1";
        let hostname_replacement = "localhost";

        let input_files = vec![];

        Config { ipv4_replacement, ipv6_replacement, hostname_replacement, input_files }
    }

    pub fn parse_args(&mut self, args: env::Args) -> Result<(), &'static str>{
        for arg in args.skip(1) {
            self.input_files.push(arg.to_string());
        }
        Ok(())
    }
}

fn replace_remote_address<W: Write>(c: &Config, r: Box<dyn BufRead>, mut w: W) -> Result<(), std::io::Error> {
    for buffer in r.lines() {
        match buffer {
            Ok(line) => {
                let v: Vec<&str> = line.splitn(2, ' ').collect();
                match v.len() {
                    1 => writeln!(&mut w, "{}", line)?,
                    2 => {
                        let (remote_addr, log) = (&v[0], &v[1]);
                        match remote_addr.parse::<net::Ipv4Addr>() {
                            Ok(_) => writeln!(&mut w, "{} {}", c.ipv4_replacement, log)?,
                            Err(_) => match remote_addr.parse::<net::Ipv6Addr>() {
                                Ok(_) => writeln!(&mut w, "{} {}", c.ipv6_replacement, log)?,
                                Err(_) => writeln!(&mut w, "{} {}", c.hostname_replacement, log)?,
                            },
                        }
                    }
                    _ => writeln!(&mut w, "{}", line)?,
                };
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

pub fn run(config: Config) {
    let mut output = std::io::stdout();

    if config.input_files.len() > 0 {
        for arg in &config.input_files {
            let f = File::open(arg);
            let f = match f {
                Ok(file) => file,
                Err(_) => {
                    eprintln!("Error reading file '{}'.", arg);
                    process::exit(1);
                }
            };
            let reader: Box<dyn BufRead> = Box::new(BufReader::new(f));
            if let Err(e) = replace_remote_address(&config, reader, &mut output) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
    } else {
        let reader: Box<dyn BufRead> = Box::new(BufReader::new(io::stdin()));
        if let Err(e) = replace_remote_address(&config, reader, &mut output) {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_ipv4() {
        use std::io::Cursor;
        let mut buff = Cursor::new(vec![]);
        let log = Box::new("8.8.8.8 - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes());
        let local_log = "127.0.0.1 - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes();

        replace_remote_address(&Config::new(), log, &mut buff).unwrap();
        assert!(&buff.get_ref().starts_with(&local_log));
    }

    #[test]
    fn replace_ipv6() {
        use std::io::Cursor;
        let mut buff = Cursor::new(vec![]);
        let log = Box::new("2a00:1450:4001:81b::2004 - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes());
        let local_log = "::1 - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes();

        replace_remote_address(&Config::new(), log, &mut buff).unwrap();
        assert!(&buff.get_ref().starts_with(&local_log));
    }

    #[test]
    fn replace_hostname() {
        use std::io::Cursor;
        let mut buff = Cursor::new(vec![]);
        let log = Box::new("google.com - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes());
        let local_log = "localhost - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes();

        replace_remote_address(&Config::new(), log, &mut buff).unwrap();
        assert!(&buff.get_ref().starts_with(&local_log));
    }
}
