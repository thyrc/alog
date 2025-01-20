//! `alog` is a simple log file anonymizer.
//!
//! ## About
//!
//! In fact by default `alog` just replaces the first *word* on every line of any input stream
//! with a customizable string.
//!
//! With version 0.6 you can (at a substantial cost of CPU cycles) replace the `$remote_user`
//! with `"-"` ([`Config::authuser`] set to `true`) as well. Defaults to `false`.
//!
//! With [`Config::trim`] set to `false` the first *word* can be the (zero width)
//! anchor ^ or a single `b' '` (Space) separated by a `b' '` from the remainder of the line.
//! This was the default behaviour prior to version 0.6.
//!
//! So "log file anonymizer" might be a bit of an overstatement, but `alog` can be used to (very
//! efficiently) replace the `$remote_addr` part in many access log formats, e.g. Nginx' default
//! combined log format:
//!
//! ```text
//! log_format combined '$remote_addr - $remote_user [$time_local] '
//!                     '"$request" $status $body_bytes_sent '
//!                     '"$http_referer" "$http_user_agent"';
//! ```
//!
//! By default any parseable `$remote_addr` is replaced by it's *localhost* representation,
//!
//! * any valid IPv4 address is replaced by `127.0.0.1`,
//! * any valid IPv6 address is replaced by `::1` and
//! * any String (what might be a domain name) with `localhost`.
//!
//! Lines without a 'first word' will remain unchanged (but can be skipped with [`Config::skip`]
//! set to `true`).
//!
//! Starting with version 0.6 all Space and Tabulator (`b'\t'`) and from version 0.7 on all
//! [ASCII whitespace](https://infra.spec.whatwg.org/#ascii-whitespace) characters will be removed
//! from the beginning of each line before replacing any `$remote_addr` by default.
//! To switch back to the previous behaviour just set [`Config::trim`] to `false`.
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
//! So keep in mind, just removing the IP / `$remote_addr` or `$remote_user` part might not be
//! enough to fully anonymize any given log file.
//!
//! [GDPR]: https://gdpr.eu/article-4-definitions/

use std::cmp::Ordering;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::{fmt, net};

use regex::bytes::Regex;

#[macro_use(lazy_static)]
extern crate lazy_static;

lazy_static! {
    // $remote_user *can* contain whitespaces, so we search for the 'next'
    // field (`$time_local`) instead
    static ref RE: Regex = Regex::new(" \\[[0-9]{1,2}/").unwrap();
}

#[allow(dead_code)]
trait Replace {
    fn replace(&self, old: &[u8], new: &[u8]) -> Vec<u8>;
    fn kmpsearch(&self, pattern: &[u8]) -> Option<Vec<usize>>;
    fn bmsearch(&self, pattern: &[u8]) -> Option<Vec<usize>>;
    fn prefix_table(pattern: &[u8]) -> Vec<isize>;
    fn bad_char_table(pattern: &[u8]) -> [usize; 256];
}

impl Replace for [u8] {
    fn replace(&self, old: &[u8], new: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.len());
        let mut i = 0;

        if let Some(matches) = self.bmsearch(old) {
            for m in matches {
                result.extend_from_slice(&self[i..m]);
                result.extend_from_slice(new);
                i = m + old.len();
            }
            result.extend_from_slice(&self[i..]);
        } else {
            return self.to_vec();
        }

        result
    }

    #[allow(clippy::cast_sign_loss)]
    fn kmpsearch(&self, pattern: &[u8]) -> Option<Vec<usize>> {
        let m = self.len();
        let n = pattern.len();

        let mut i = 0;
        let mut j = 0;

        if pattern.is_empty() || n > m {
            return None;
        }

        let table = Self::prefix_table(pattern);

        let mut indices = Vec::new();

        while i < m {
            while j >= 0 && self[i] != pattern[j as usize] {
                j = table[j as usize];
            }

            i += 1;
            j += 1;
            if (j as usize) == n {
                indices.push(i - n);
                j = table[j as usize];
            }
        }

        if indices.is_empty() {
            None
        } else {
            Some(indices)
        }
    }

    fn bmsearch(&self, pattern: &[u8]) -> Option<Vec<usize>> {
        let m = self.len();
        let n = pattern.len();

        if pattern.is_empty() || n > m {
            return None;
        }

        let table = Self::bad_char_table(pattern);

        let mut indices = Vec::new();

        let mut i = 0;
        while i < m - n {
            let mut j = n - 1;
            while pattern[j] == self[i + j] {
                if j == 0 {
                    indices.push(i);
                    break;
                }
                j -= 1;
            }
            i += table[self[i + n - 1] as usize];
        }

        if indices.is_empty() {
            None
        } else {
            Some(indices)
        }
    }

    #[allow(clippy::cast_sign_loss)]
    fn prefix_table(pattern: &[u8]) -> Vec<isize> {
        let mut i = 0;
        let mut j: isize = -1;
        let mut table = vec![0; pattern.len() + 1];
        table[i] = j;
        while i < pattern.len() {
            while j >= 0 && pattern[j as usize] != pattern[i] {
                j = table[j as usize];
            }

            i += 1;
            j += 1;
            table[i] = j;
        }
        table
    }

    fn bad_char_table(pattern: &[u8]) -> [usize; 256] {
        let n = pattern.len();
        let mut table = [n; 256];

        for (i, &c) in pattern.iter().enumerate().take(n - 1) {
            table[c as usize] = n - i - 1;
        }

        table
    }
}

#[derive(Debug)]
pub struct IOError {
    message: String,
}

impl fmt::Display for IOError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<io::Error> for IOError {
    fn from(error: io::Error) -> Self {
        IOError {
            message: error.to_string(),
        }
    }
}

/// INPUT / OUTPUT config
#[derive(Debug)]
pub struct IOConfig<'a> {
    /// List of input paths / files, e.g. `Some(vec![Path::new("/tmp/test1.log"), Path::new("/tmp/test2.log")])`
    /// If set to `None` the reader will read from Stdin.
    input: Option<Vec<&'a Path>>,
    /// Single output path / file
    /// If set to `None` the writer will write to Stdout.
    output: Option<&'a Path>,
}

/// Collection of replacement strings / config flags
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug)]
pub struct Config<'a> {
    /// IPv4-parseable `$remote_addr` replacement string
    pub ipv4: &'a str,
    /// IPv6-parseable `$remote_addr` replacement string
    pub ipv6: &'a str,
    /// `$remote_addr` replacement string
    pub host: &'a str,
    /// Skip lines w/o a `$remote_addr` part / first word
    pub skip: bool,
    /// Try to clear the authuser field
    pub authuser: bool,
    /// Trim spaces from the start of every line
    pub trim: bool,
    /// Replace all occurrences of `$remote_addr` in each
    /// line
    pub thorough: bool,
    /// Don't clear authuser fields starting with "- ["
    /// We assume these fields are already cleared.
    pub optimize: bool,
    /// Flush output after each line
    pub flush: bool,
}

/// defaults to `None` for both input and output
impl<'a> Default for IOConfig<'a> {
    fn default() -> Self {
        IOConfig {
            input: None,
            output: None,
        }
    }
}

/// `$remote_addr` replacements default to an equivalent of *localhost*
impl<'a> Default for Config<'a> {
    fn default() -> Self {
        Config {
            ipv4: "127.0.0.1",
            ipv6: "::1",
            host: "localhost",
            skip: false,
            authuser: false,
            trim: true,
            thorough: false,
            optimize: true,
            flush: false,
        }
    }
}

impl<'a> Config<'a> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get IPv4 replacement value
    #[must_use]
    pub fn get_ipv4_value(&self) -> &'a str {
        self.ipv4
    }

    /// Get IPv6 replacement value
    #[must_use]
    pub fn get_ipv6_value(&self) -> &'a str {
        self.ipv6
    }

    /// Get string replacement value
    #[must_use]
    pub fn get_host_value(&self) -> &'a str {
        self.host
    }

    /// Get `skip` value
    #[must_use]
    pub fn get_skip(&self) -> bool {
        self.skip
    }

    /// Get `authuser` value
    #[must_use]
    pub fn get_authuser(&self) -> bool {
        self.authuser
    }

    /// Get `trim` value
    #[must_use]
    pub fn get_trim(&self) -> bool {
        self.trim
    }
    /// Get `thorough` value
    #[must_use]
    pub fn get_thorough(&self) -> bool {
        self.thorough
    }

    /// Get `optimize` value
    #[must_use]
    pub fn get_optimize(&self) -> bool {
        self.optimize
    }

    /// Get `flush` value
    #[must_use]
    pub fn get_flush(&self) -> bool {
        self.flush
    }

    /// Set IPv4 replacement `String`
    pub fn set_ipv4_value(&mut self, ipv4: &'a str) {
        self.ipv4 = ipv4;
    }

    /// Set IPv6 replacement `String`
    pub fn set_ipv6_value(&mut self, ipv6: &'a str) {
        self.ipv6 = ipv6;
    }

    /// Set `hostname` replacement `String`
    pub fn set_host_value(&mut self, host: &'a str) {
        self.host = host;
    }

    /// Set `flush` field
    pub fn set_flush(&mut self, b: bool) {
        self.flush = b;
    }

    /// Set `authuser` field
    pub fn set_authuser(&mut self, b: bool) {
        self.authuser = b;
    }

    /// Set `trim` field
    pub fn set_trim(&mut self, b: bool) {
        self.trim = b;
    }
    /// Set `thorough` field
    pub fn set_thorough(&mut self, b: bool) {
        self.thorough = b;
    }

    /// Set `optimize` field
    pub fn set_optimize(&mut self, b: bool) {
        self.optimize = b;
    }

    /// Set `skip` field
    pub fn set_skip(&mut self, b: bool) {
        self.skip = b;
    }
}

impl<'a> IOConfig<'a> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    /// Get input / reader names, if any (defaults to `None`)
    pub fn get_input(&self) -> Option<&Vec<&'a Path>> {
        self.input.as_ref()
    }

    #[must_use]
    /// Get output / writer name (defaults to `None`)
    pub fn get_output(&self) -> Option<&'a Path> {
        self.output
    }

    /// Add input `Path`
    pub fn push_input<P: AsRef<Path> + ?Sized>(&mut self, i: &'a P) {
        if let Some(input) = &mut self.input {
            input.push(i.as_ref());
        } else {
            self.input = Some(vec![]);
            self.push_input(i);
        }
    }

    /// Set output `Path`
    pub fn set_output(&mut self, o: &'a Path) {
        self.output = Some(o);
    }
}

/// Reads lines from `reader`, if there is a '*first word*' (any String separated from the
/// remainder of the line by a b' ' (Space) byte) this word will be replaced
///
/// Any word, that can be parsed as
/// * [`std::net::Ipv4Addr`] will be replaced with [`alog::Config::ipv4`],
/// * [`std::net::Ipv6Addr`] will be replaced with [`alog::Config::ipv6`],
/// * any other *word* will be replaced with [`alog::Config::host`].
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
) -> Result<(), io::Error> {
    let mut buf = vec![];
    let mut repl;

    'lines: loop {
        buf.clear();
        let bytes_read = reader.read_until(b'\n', &mut buf)?;
        if bytes_read == 0 {
            break;
        }

        if config.get_trim() {
            let s = buf
                .iter()
                .position(|&x| !x.is_ascii_whitespace())
                .unwrap_or(0);
            buf.drain(..s);
        }

        for (i, byte) in buf.iter().enumerate() {
            if byte.is_ascii_whitespace() {
                let needle = &String::from_utf8_lossy(&buf[..i]);
                repl = match needle {
                    s if s.parse::<net::Ipv4Addr>().is_ok() => config.get_ipv4_value(),
                    s if s.parse::<net::Ipv6Addr>().is_ok() => config.get_ipv6_value(),
                    s if s.is_empty() && config.get_skip() => continue 'lines,
                    _ => config.get_host_value(),
                };

                write!(&mut writer, "{repl}")?;

                let is_authuser = config.get_authuser();
                let is_thorough = config.get_thorough();
                let is_optimized = config.get_optimize() && buf.len() >= i + 6;

                if is_authuser {
                    if is_optimized && buf[i + 3..i + 6].iter().cmp(b"- [") == Ordering::Equal {
                        write_or_replace(&buf[i..], needle, repl, is_thorough, &mut writer)?;
                    } else if let Some(time_field) = RE.find_at(&buf, i) {
                        write!(&mut writer, " - -")?;
                        write_or_replace(
                            &buf[time_field.start()..],
                            needle,
                            repl,
                            is_thorough,
                            &mut writer,
                        )?;
                    } else {
                        write_or_replace(&buf[i..], needle, repl, is_thorough, &mut writer)?;
                    }
                } else if is_thorough {
                    write_or_replace(&buf[i..], needle, repl, true, &mut writer)?;
                } else {
                    writer.write_all(&buf[i..])?;
                }

                if config.get_flush() {
                    writer.flush()?;
                }

                continue 'lines;
            }
        }
    }

    writer.flush()?;
    Ok(())
}

fn write_or_replace<W: Write>(
    slice: &[u8],
    needle: &str,
    repl: &str,
    should_replace: bool,
    writer: &mut W,
) -> Result<(), io::Error> {
    if should_replace && !needle.is_empty() {
        writer.write_all(&slice.replace(needle.as_bytes(), repl.as_bytes()))?;
    } else {
        writer.write_all(slice)?;
    }
    Ok(())
}

/// Creates a reader (defaults to [`std::io::Stdin`]) and writer (defaults to [`std::io::Stdout`])
/// from [`alog::IOConfig`] and uses both along with [`alog::Config`] to actually replace
/// any first *word* in `reader` with strings stored in [`alog::Config`].
///
/// Appends data if the writer points to an existing, writeable file.
///
/// ## Errors
///
/// Returns an error if the new reader / writer retruns an error.
///
/// ## Example
///
/// ```no_run
/// alog::run(
///     &alog::Config {
///         host: "XXX",
///         ..Default::default()
///     },
///     &alog::IOConfig::default()
/// ).unwrap();
/// ```
///
/// [`alog::Config`]: ./struct.Config.html
/// [`alog::IOConfig`]: ./struct.IOConfig.html
/// [`alog::IOConfig::get_output()`]: ./struct.IOConfig.html#method.get_output
/// [`std::io::Stdin`]: https://doc.rust-lang.org/std/io/struct.Stdin.html
/// [`std::io::Stdout`]: https://doc.rust-lang.org/std/io/struct.Stdout.html
/// [`std::net::Ipv4Addr`]: https://doc.rust-lang.org/std/net/struct.Ipv4Addr.html
/// [`std::net::Ipv6Addr`]: https://doc.rust-lang.org/std/net/struct.Ipv6Addr.html
pub fn run(config: &Config, ioconfig: &IOConfig) -> Result<(), IOError> {
    // Set writer
    let stdout = io::stdout();
    let mut writer: Box<dyn Write> = match ioconfig.get_output() {
        Some(output) => {
            let f = match OpenOptions::new()
                .create(true)
                .append(true)
                .open(Path::new(output))
            {
                Ok(f) => f,
                Err(e) => {
                    return Err(IOError {
                        message: format!("Can not open output '{}': {e}", output.display()),
                    })
                }
            };
            Box::new(BufWriter::new(f)) as _
        }
        None => Box::new(BufWriter::new(stdout.lock())),
    };

    // Set reader
    if let Some(input) = ioconfig.get_input() {
        for arg in input {
            match File::open(Path::new(arg)) {
                Err(e) => {
                    return Err(IOError {
                        message: format!("Can not open input '{}': {e}", arg.display()),
                    })
                }
                Ok(f) => {
                    let reader: Box<dyn BufRead> = Box::new(BufReader::new(f));
                    if let Err(e) = replace_remote_address(config, reader, &mut writer) {
                        return Err(IOError {
                            message: e.to_string(),
                        });
                    }
                }
            }
        }
    } else {
        let stdin = io::stdin();
        let reader: Box<dyn BufRead> = Box::new(stdin.lock());
        if let Err(e) = replace_remote_address(config, reader, &mut writer) {
            return Err(IOError {
                message: e.to_string(),
            });
        }
    }

    Ok(())
}

/// Like [`alog::run`] but will let you pass your own `reader` and `writer`. Replacement strings
/// and config flags will still be read from [`alog::Config`].
///
/// ## Errors
///
/// Returns an error if the new reader or writer retruns an error.
///
/// ## Example
///
/// ```
/// use std::io::Cursor;
///
/// let line = Cursor::new(b"8.8.8.8 XxX");
/// let mut buffer = vec![];
///
/// alog::run_raw(&alog::Config::default(), line, &mut buffer).unwrap();
/// assert_eq!(buffer, b"127.0.0.1 XxX");
/// ```
///
/// To read from Stdin and write to Stdout:
///
/// ```no_run
/// use std::io::{self, BufReader, BufWriter};
///
/// // Consider wrapping io::stdout in BufWriter
/// let stdin = io::stdin();
/// let stdout = io::stdout();
/// alog::run_raw(&alog::Config::default(), stdin.lock(), stdout.lock()).unwrap();
/// ```
///
/// [`alog::run`]: ./fn.run.html
/// [`alog::Config`]: ./struct.Config.html
pub fn run_raw<R: BufRead, W: Write>(
    config: &Config,
    reader: R,
    mut writer: W,
) -> Result<(), IOError> {
    replace_remote_address(config, reader, &mut writer)?;
    Ok(())
}

#[cfg(test)]
mod tests {
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
}
