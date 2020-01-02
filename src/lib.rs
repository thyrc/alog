//! `alog` is a simple log file anonymizer.
//!
//! # About
//!
//! In fact `alog` just replaces the first *word*[^1] on every line of any UTF-8 stream with a customizable string.
//!
//! So "log file anonymizer" might be a bit of an overstatement, but `alog` can be used to (very efficiently) replace the $remote_address
//! part in many access log formats, e.g. Nginx' default combined access_log.
//!
//! By default any parseable $remote_address is replaced by it's *localhost* representation,
//! * any valid IPv4 address is replaced by '127.0.0.1',
//! * any valid IPv6 address is replaced by '::1'
//! * and any String (what might be a domain name) with 'localhost'.
//!
//! For now this only works on UTF-8 input streams, which is not much of an problem on most Linux systems, but still is a restriction not present in the
//! <10 line Perl script `alog` was supposed to replace.
//!
//! [^1]: Any first substring separated by a `' '` (Space) from the remainder of the line.
//!
//! ## Personal data in server logs
//!
//! The default configuration of popular web servers including Apache Web Server and Nginx collect and store at least two of the following three types of logs:
//!
//! 1. access logs
//! 2. error logs (including processing-language logs like PHP)
//! 3. security audit logs
//!
//! All of these logs contain personal information by default. IP addresses are specifically defined as personal data by the GDPR.
//! The logs can also contain usernames if your web service uses them as part of their URL structure, and even the referral information
//! that’s logged by default **can** contain personal information (e.g. unintended collection of sensitive data).
//!
//! So keep in mind, that just removing the IP / `$remote_host` part might not be enough to fully anonymize any given log file.

use std::fs::File;
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::net;
use std::path::Path;
use std::process;

/// INPUT / OUTPUT config
#[derive(Debug)]
pub struct IOConfig {
    input: Option<Vec<String>>,
    output: Option<String>,
}

/// Collection of replacement strings
#[derive(Debug)]
pub struct Config {
    ipv4: String,
    ipv6: String,
    host: String,
}

/// defaults to `None` for both input and output
impl Default for IOConfig {
    fn default() -> Self {
        IOConfig {
            input: None,
            output: None,
        }
    }
}

/// defaults to an equivalent of *localhost*
impl Default for Config {
    fn default() -> Self {
        Config {
            ipv4: "127.0.0.1".to_string(),
            ipv6: "::1".to_string(),
            host: "localhost".to_string(),
        }
    }
}

impl Config {
    /// Get IPv4 replacement value
    pub fn get_ipv4_value(&self) -> &String {
        &self.ipv4
    }

    /// Get IPv6 replacement value
    pub fn get_ipv6_value(&self) -> &String {
        &self.ipv6
    }

    /// Get string replacement value
    pub fn get_host_value(&self) -> &String {
        &self.host
    }

    /// Set IPv4 replacement `String`
    pub fn set_ipv4_value(&mut self, ipv4: &str) {
        self.ipv4 = ipv4.to_string();
    }

    /// Set IPv6 replacement `String`
    pub fn set_ipv6_value(&mut self, ipv6: &str) {
        self.ipv6 = ipv6.to_string();
    }

    /// Set `hostname` replacement `String`
    pub fn set_host_value(&mut self, host: &str) {
        self.host = host.to_string();
    }
}

impl IOConfig {
    /// Get input / reader names, if any (defaults to `None`)
    pub fn get_input(&self) -> Option<&Vec<String>> {
        self.input.as_ref()
    }

    /// Get output / writer name (defaults to `None`)
    pub fn get_output(&self) -> Option<&String> {
        self.output.as_ref()
    }

    /// Add input `Path`
    pub fn push_input(&mut self, i: &str) {
        if let Some(input) = &mut self.input {
            input.push(i.to_string());
        } else {
            self.input = Some(vec![]);
            self.push_input(i);
        }
    }

    /// Set output `Path`
    pub fn set_output(&mut self, output: &str) {
        self.output = Some(output.to_string());
    }
}

/// Errors
///
/// This function has the same error semantics as `BufRead::read_line` and will also return an error if the read bytes are not valid UTF-8.
fn replace_remote_address<W: Write>(
    config: &Config,
    reader: Box<dyn BufRead>,
    mut writer: W,
) -> Result<(), std::io::Error> {
    for buffer in reader.lines() {
        match buffer {
            Ok(line) => {
                let v: Vec<&str> = line.splitn(2, ' ').collect();
                match v.len() {
                    // 1 => writeln!(&mut writer, "{}", line)?,
                    2 => {
                        let (remote_addr, log) = (&v[0], &v[1]);
                        match remote_addr.parse::<net::Ipv4Addr>() {
                            Ok(_) => writeln!(&mut writer, "{} {}", config.get_ipv4_value(), log)?,
                            Err(_) => match remote_addr.parse::<net::Ipv6Addr>() {
                                Ok(_) => {
                                    writeln!(&mut writer, "{} {}", config.get_ipv6_value(), log)?
                                }
                                Err(_) => {
                                    writeln!(&mut writer, "{} {}", config.get_host_value(), log)?
                                }
                            },
                        }
                    }
                    _ => writeln!(&mut writer, "{}", line)?,
                };
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

/// Creates a reader (defaults to `io::Stdin`) and writer (defaults to `io::Stdout`) from `IOConfig.reader` and `IOConfig.writer`, passes both along with the
/// `Config` struct to actually replace any first *word* in `reader` with `String`s in `Config.{ip4,ipv6,host}`.
///
/// Any word, that can be parsed as
/// * `net::Ipv4Addr` will be replaced with `Config.ip4`,
/// * `net::Ipv6Addr` will be replaced with `Config.ip6`,
/// * any other *word* will be replaced with `Config.host`.
///
/// # Errors
///
/// Exits when `IOConfig.output` already exists
/// Exits when `IOConfig.config` contains invalid UTF-8.
pub fn run(ioconfig: &IOConfig, repl: &Config) {
    // Set writer
    let mut writer: Box<dyn Write> = match ioconfig.get_output() {
        Some(output) => {
            let f = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(Path::new(output));
            let f = match f {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Error writing to file {}: {}.", output, e);
                    std::process::exit(1);
                }
            };
            Box::new(BufWriter::new(f)) as _
        }
        None => Box::new(std::io::stdout()) as _,
    };

    // Set reader
    if let Some(input) = ioconfig.get_input() {
        for arg in input {
            let f = File::open(Path::new(arg));
            let f = match f {
                Ok(file) => file,
                Err(e) => {
                    eprintln!("Error reading file '{}': {}.", arg, e);
                    if let Some(output) = ioconfig.get_output() {
                        std::fs::remove_file(Path::new(output)).unwrap();
                    }
                    process::exit(1);
                }
            };
            let reader: Box<dyn BufRead> = Box::new(BufReader::new(f));
            if let Err(e) = replace_remote_address(repl, reader, &mut writer) {
                eprintln!("Error: {}", e);
                if let Some(output) = ioconfig.get_output() {
                    std::fs::remove_file(Path::new(output)).unwrap();
                }
                process::exit(1);
            }
        }
    } else {
        let reader: Box<dyn BufRead> = Box::new(BufReader::new(io::stdin()));
        if let Err(e) = replace_remote_address(repl, reader, &mut writer) {
            eprintln!("Error: {}", e);
            if let Some(output) = ioconfig.get_output() {
                std::fs::remove_file(Path::new(output)).unwrap();
            }
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

        replace_remote_address(&Config::default(), log, &mut buff).unwrap();
        assert!(&buff.get_ref().starts_with(&local_log));
    }

    #[test]
    fn replace_ipv6() {
        use std::io::Cursor;
        let mut buff = Cursor::new(vec![]);
        let log = Box::new("2a00:1450:4001:81b::2004 - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes());
        let local_log = "::1 - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes();

        replace_remote_address(&Config::default(), log, &mut buff).unwrap();
        assert!(&buff.get_ref().starts_with(&local_log));
    }

    #[test]
    fn replace_host() {
        use std::io::Cursor;
        let mut buff = Cursor::new(vec![]);
        let log = Box::new("google.com - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes());
        let local_log = "localhost - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes();

        replace_remote_address(&Config::default(), log, &mut buff).unwrap();
        assert!(&buff.get_ref().starts_with(&local_log));
    }

    #[test]
    fn replace_custom_ipv4() {
        use std::io::Cursor;
        let mut buff = Cursor::new(vec![]);
        let log = Box::new("8.8.8.8 - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes());
        let local_log = "custom_ipv4 - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes();

        let mut conf = Config::default();
        conf.set_ipv4_value("custom_ipv4");

        replace_remote_address(&conf, log, &mut buff).unwrap();
        assert!(&buff.get_ref().starts_with(&local_log));
    }

    #[test]
    fn replace_custom_ipv6() {
        use std::io::Cursor;
        let mut buff = Cursor::new(vec![]);
        let log = Box::new("2a00:1450:4001:81b::2004 - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes());
        let local_log = "custom_ipv6 - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes();

        let mut conf = Config::default();
        conf.set_ipv6_value("custom_ipv6");

        replace_remote_address(&conf, log, &mut buff).unwrap();
        assert!(&buff.get_ref().starts_with(&local_log));
    }

    #[test]
    fn replace_custom_host() {
        use std::io::Cursor;
        let mut buff = Cursor::new(vec![]);
        let log = Box::new("google.com - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes());
        let local_log = "custom_host - frank [10/Oct/2000:13:55:36 -0700] \"GET /apache_pb.gif HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"".as_bytes();

        let mut conf = Config::default();
        conf.set_host_value("custom_host");

        replace_remote_address(&conf, log, &mut buff).unwrap();
        assert!(&buff.get_ref().starts_with(&local_log));
    }
}
