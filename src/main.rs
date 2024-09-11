use std::{ffi::OsString, path::Path, process};

const HELP: &str = "\
Mangle common / combined logs

USAGE:
    alog [FLAGS] [OPTIONS] [INPUT]...

FLAGS:
    -a, --authuser        Clear authuser
    -f, --flush-line      Flush output on every line
        --no-optimize     Don't try to reduce performance hit with `--authuser`
    -n, --notrim          Don't remove Space and Tab from the start of every line
    -s, --skip-invalid    Skip invalid lines

    -h, --help            Print this message
    -V, --version         Print version information

OPTIONS:
        --host-replacement <host-replacement>    Sets host replacement string [default: localhost]
    -4, --ipv4-replacement <ipv4-replacement>    Sets IPv4 replacement string [default: 127.0.0.1]
    -6, --ipv6-replacement <ipv6-replacement>    Sets IPv6 replacement string [default: ::1]
    -o, --output <FILE>                          Sets output file

ARGS:
    <INPUT>...    The input file(s) to use";

fn main() -> Result<(), lexopt::Error> {
    use lexopt::prelude::*;

    let mut config = alog::Config::default();
    let mut ioconfig = alog::IOConfig::default();

    let mut host_replacement = config.get_host_value().to_string();
    let mut ipv4_replacement = config.get_ipv4_value().to_string();
    let mut ipv6_replacement = config.get_ipv6_value().to_string();

    let mut output: Option<OsString> = None;
    let mut input: Vec<OsString> = vec![];

    let mut parser = lexopt::Parser::from_env();

    while let Some(arg) = parser.next()? {
        match arg {
            Short('a') | Long("authuser") => config.set_authuser(true),
            Short('f') | Long("flush-line") => config.set_flush(true),
            Long("no-optimize") => config.set_optimize(false),
            Short('n') | Long("notrim") => config.set_trim(false),
            Short('s') | Long("skip-invalid") => config.set_skip(true),
            Long("host-replacement") => host_replacement = parser.value()?.string()?,
            Short('4') | Long("ipv4-replacement") => ipv4_replacement = parser.value()?.string()?,
            Short('6') | Long("ipv6-replacement") => ipv6_replacement = parser.value()?.string()?,
            Short('o') | Long("output") => output = Some(parser.value()?.parse()?),
            Value(f) => input.push(f),
            Short('h') | Long("help") => {
                println!("{HELP}");
                process::exit(0);
            }
            Short('V') | Long("version") => {
                println!("{} {}", env!("CARGO_BIN_NAME"), env!("CARGO_PKG_VERSION"));
                process::exit(0);
            }
            _ => return Err(arg.unexpected()),
        }
    }

    config.set_host_value(&host_replacement);
    config.set_ipv4_value(&ipv4_replacement);
    config.set_ipv6_value(&ipv6_replacement);

    let opath = output.unwrap_or_default();
    if !opath.is_empty() {
        ioconfig.set_output(Path::new(opath.as_os_str()));
    }

    for i in &input {
        ioconfig.push_input(i);
    }

    if let Err(e) = alog::run(&config, &ioconfig) {
        eprintln!("Error: {e}");
        process::exit(1);
    };

    Ok(())
}
