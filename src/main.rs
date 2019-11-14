use std::env;
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader};
use std::net;

fn replace_remote_address(reader: Box<dyn BufRead>) {
    for buffer in reader.lines() {
        match buffer {
            Ok(line) => {
                // trim_end() gains ~10% speed
                let v: Vec<&str> = line.trim_end().splitn(2, ' ').collect();
                match v.len() {
                    1 => match line.len() {
                        _ => println!("{}", &v[0]),
                    },
                    2 => {
                        let (remote_addr, log) = (&v[0], &v[1]);
                        match remote_addr.parse::<net::Ipv4Addr>() {
                            Ok(_) => println!("127.0.0.1 {}", log),
                            Err(_) => match remote_addr.parse::<net::Ipv6Addr>() {
                                Ok(_) => println!("::1 {}", log),
                                Err(_) => println!("localhost {}", log),
                            },
                        }
                    }
                    _ => continue,
                }
            }
            Err(err) => { eprintln!("Error reading from reader: {}", err);
                          std::process::exit(1);
            },
        }
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() > 1 {
        for arg in &args[1..] {
            let f = OpenOptions::new().read(true).open(arg);
            let f = match f {
                Ok(file) => file,
                Err(_) => { eprintln!("Error reading file '{}'.", arg);
                            std::process::exit(1);
                }
            };
            let reader: Box<dyn BufRead> = Box::new(BufReader::new(f));
            replace_remote_address(reader);
        }
    } else {
        let reader: Box<dyn BufRead> = Box::new(BufReader::new(io::stdin()));
        replace_remote_address(reader);
    }

}
