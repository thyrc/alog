//! `alog` is a simple log file anonymizer.
//!
//! ## About
//!
//! In fact `alog` just replaces the first *word*[^1] on every line of any input stream with a
//! customizable string.
//!
//! So "log file anonymizer" might be a bit of an overstatement, but `alog` can be used to (very
//! efficiently) replace the $remote_addr part in many access log formats, e.g. Nginx' default
//! combined log format:
//!
//! ```text
//! log_format combined '$remote_addr - $remote_user [$time_local] '
//!                     '"$request" $status $body_bytes_sent '
//!                     '"$http_referer" "$http_user_agent"';
//! ```
//!
//! By default any parseable $remote_addr is replaced by it's *localhost* representation,
//!
//! * any valid IPv4 address is replaced by '127.0.0.1',
//! * any valid IPv6 address is replaced by '::1' and
//! * any String (what might be a domain name) with 'localhost'.
//!
//! Lines without a $remote_addr part will remain unchanged (but can be skipped with
//! [`alog::Config::set_skip()`] set to `true`).
//!
//! [^1]: Any first substring separated by a `b' '` (Space) from the remainder of the line.
//!
//! ### Personal data in server logs
//!
//! The default configuration of popular web servers including Apache Web Server and Nginx collect
//! and store at least two of the following three types of logs:
//!
//! 1. access logs
//! 2. error logs (including processing-language logs like PHP)
//! 3. security audit logs
//!
//! All of these logs contain personal information by default. IP addresses are specifically
//! defined as personal data by the [GDPR].  The logs can also contain usernames if your web
//! service uses them as part of their URL structure, and even the referral information that’s
//! logged by default **can** contain personal information (or other sensitive data).
//!
//! So keep in mind, just removing the IP / `$remote_addr` part might not be enough to fully
//! anonymize any given log file.
//!
//! [GDPR]: https://gdpr.eu/article-4-definitions/
//! [`alog::Config::set_skip()`]: ./struct.Config.html#method.set_skip

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

/// Collection of replacement strings / config flags
#[derive(Debug)]
pub struct Config {
    ipv4: String,
    ipv6: String,
    host: String,
    skip: bool,
    flush: bool,
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
            skip: false,
            flush: false,
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

    /// Get `skip` value
    pub fn get_skip(&self) -> bool {
        self.skip
    }

    /// Get `flush` value
    pub fn get_flush(&self) -> bool {
        self.flush
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

    /// Set `flush` field
    pub fn set_flush(&mut self, b: bool) {
        self.flush = b;
    }

    /// Set `skip` field
    pub fn set_skip(&mut self, b: bool) {
        self.skip = b;
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

/// Reads lines from `reader`, if there is a '*first word*' (any String separated from the
/// remainder of the line by a b' ' (Space) byte) this word will be replaced
///
/// Any word, that can be parsed as * [`std::net::Ipv4Addr`] will be replaced with
/// [`alog::Config::get_ipv4_value()`], * [`std::net::Ipv6Addr`] will be replaced with
/// [`alog::Config::get_ipv6_value()`], * any other *word* will be replaced with
/// [`alog::Config::get_host_value()`].
///
/// Any line without a 'first word' will be written as is if [`alog::Config::get_skip()`] returns
/// `false` (default), or will be skipped otherwise.
///
/// ## Errors
///
/// This function will return an I/O error if the underlying reader or writer returns an error.
///
/// [`alog::Config::get_ipv4_value()`]: ./struct.Config.html#method.get_ipv4_value
/// [`alog::Config::get_ipv6_value()`]: ./struct.Config.html#method.get_ipv6_value
/// [`alog::Config::get_host_value()`]: ./struct.Config.html#method.get_host_value
fn replace_remote_address<R: BufRead, W: Write>(
    config: &Config,
    mut reader: R,
    mut writer: W,
) -> Result<(), std::io::Error> {
    let mut buf = vec![];

    'lines: loop {
        buf.clear();
        let bytes_read = reader.read_until(b'\n', &mut buf)?;
        match bytes_read {
            0 => break,
            _ => {
                for (i, byte) in buf.iter().enumerate() {
                    if *byte == b' ' {
                        if let Ok(_) = String::from_utf8_lossy(&buf[..i]).parse::<net::Ipv4Addr>() {
                            write!(&mut writer, "{}", config.get_ipv4_value())?;
                        } else if let Ok(_) =
                            String::from_utf8_lossy(&buf[..i]).parse::<net::Ipv6Addr>()
                        {
                            write!(&mut writer, "{}", config.get_ipv6_value())?;
                        } else {
                            write!(&mut writer, "{}", config.get_host_value())?;
                        }
                        writer.write(&buf[i..])?;
                        if config.get_flush() == true {
                            writer.flush()?;
                        }
                        continue 'lines;
                    }
                }
                if config.get_skip() != true {
                    writer.write(&buf)?;
                    if config.get_flush() == true {
                        writer.flush()?;
                    }
                }
            }
        };
    }
    writer.flush()?;
    Ok(())
}

/// Creates a reader (defaults to [`std::io::Stdin`]) and writer (defaults to [`std::io::Stdout`])
/// from [`alog::IOConfig`], passes both along with the [`alog::Config`] struct to actually replace
/// any first *word* in `reader` with strings stored in [`alog::Config`].
///
/// ## Errors
///
/// Exits when [`alog::IOConfig::get_output()`] already exists or the new reader / writer retruns
/// an error.
///
/// [`alog::Config`]: ./struct.Config.html
/// [`alog::IOConfig`]: ./struct.IOConfig.html
/// [`alog::IOConfig::get_output()`]: ./struct.IOConfig.html#method.get_output
/// [`std::io::Stdin`]: https://doc.rust-lang.org/std/io/struct.Stdin.html
/// [`std::io::Stdout`]: https://doc.rust-lang.org/std/io/struct.Stdout.html
/// [`std::net::Ipv4Addr`]: https://doc.rust-lang.org/std/net/struct.Ipv4Addr.html
/// [`std::net::Ipv6Addr`]: https://doc.rust-lang.org/std/net/struct.Ipv6Addr.html
pub fn run(ioconfig: &IOConfig, config: &Config) {
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
        None => Box::new(BufWriter::new(io::stdout())),
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
            if let Err(e) = replace_remote_address(config, reader, &mut writer) {
                eprintln!("Error: {}", e);
                if let Some(output) = ioconfig.get_output() {
                    std::fs::remove_file(Path::new(output)).unwrap();
                }
                process::exit(1);
            }
        }
    } else {
        let reader: Box<dyn BufRead> = Box::new(BufReader::new(io::stdin()));
        if let Err(e) = replace_remote_address(config, reader, &mut writer) {
            eprintln!("Error: {}", e);
            if let Some(output) = ioconfig.get_output() {
                std::fs::remove_file(Path::new(output)).unwrap();
            }
            process::exit(1);
        }
    }
}

/// Like [`alog::run`] but will let you pass your own `reader` and `writer`. Replacement strings
/// and config flags will still be read from [`alog::Config`] though.
///
/// ## Example
/// ```
/// use alog::{Config, run_raw};
/// use std::io::Cursor;
///
/// let line = Cursor::new(b"8.8.8.8 XxX");
/// let mut buffer = vec![];
///
/// run_raw(&Config::default(), line, &mut buffer);
/// assert_eq!(buffer, b"127.0.0.1 XxX");
/// ```
///
/// To read from Stdin and write to Stdout:
///
/// ```no_run
/// use alog::{Config, run_raw};
/// use std::io::{self, BufReader, BufWriter};
///
/// // Consider wrapping io::stdout in BufWriter
/// run_raw(&Config::default(), BufReader::new(io::stdin()), io::stdout());
/// ```
/// ## Errors
///
/// Exits when the new reader or writer retruns an error.
///
/// [`alog::run`]: ./fn.run.html
/// [`alog::Config`]: ./struct.Config.html
pub fn run_raw<R: BufRead, W: Write>(config: &Config, reader: R, mut writer: W) {
    if let Err(e) = replace_remote_address(config, reader, &mut writer) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_raw_function() {
        use std::io::Cursor;
        let line = Cursor::new(b"8.8.8.8 XxX");
        let mut buffer = vec![];

        run_raw(&Config::default(), line, &mut buffer);
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
}
