use clap::{builder::ValueParser, Arg, Command};
use std::ffi::OsString;
use std::path::Path;

#[allow(clippy::too_many_lines)]
fn main() {
    let default_config = alog::Config::default();
    let mut config = alog::Config::default();
    let mut ioconfig = alog::IOConfig::default();

    let cli_arguments = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("Mangle common / combined logs")
        .arg(
            Arg::new("ipv4-replacement")
                .short('4')
                .long("ipv4-replacement")
                .value_name("ipv4-replacement")
                .default_value(default_config.get_ipv4_value())
                .help("Sets IPv4 replacement string")
                .takes_value(true),
        )
        .arg(
            Arg::new("ipv6-replacement")
                .short('6')
                .long("ipv6-replacement")
                .value_name("ipv6-replacement")
                .default_value(default_config.get_ipv6_value())
                .help("Sets IPv6 replacement string")
                .takes_value(true),
        )
        .arg(
            Arg::new("host-replacement")
                .long("host-replacement")
                .value_name("host-replacement")
                .default_value(default_config.get_host_value())
                .help("Sets host replacement string")
                .takes_value(true),
        )
        .arg(
            Arg::new("skip-invalid")
                .short('s')
                .long("skip-invalid")
                .value_name("skip-invalid")
                .help("Skip invalid lines")
                .takes_value(false),
        )
        .arg(
            Arg::new("authuser")
                .short('a')
                .long("authuser")
                .value_name("authuser")
                .help("Clear authuser")
                .takes_value(false),
        )
        .arg(
            Arg::new("notrim")
                .short('n')
                .long("notrim")
                .value_name("no-trim")
                .help("Don't remove Space and Tab from the start of every line")
                .takes_value(false),
        )
        .arg(
            Arg::new("nooptimize")
                .long("no-optimize")
                .value_name("nooptimize")
                .help("Don't try to reduce performance hit with `--authuser`")
                .takes_value(false),
        )
        .arg(
            Arg::new("flush-line")
                .short('f')
                .long("flush-line")
                .value_name("flush-line")
                .help("Flush output on every line")
                .takes_value(false),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Sets output file")
                .value_parser(ValueParser::os_string())
                .takes_value(true),
        )
        .arg(
            Arg::new("input")
                .value_name("INPUT")
                .help("The input file(s) to use")
                .index(1)
                .value_parser(ValueParser::os_string())
                .multiple_values(true),
        )
        .get_matches();

    if let Some(ipv4) = cli_arguments
        .get_one::<String>("ipv4-replacement")
        .map(std::string::String::as_str)
    {
        config.set_ipv4_value(ipv4);
    }

    if let Some(ipv6) = cli_arguments
        .get_one::<String>("ipv6-replacement")
        .map(std::string::String::as_str)
    {
        config.set_ipv6_value(ipv6);
    }

    if let Some(host) = cli_arguments
        .get_one::<String>("host-replacement")
        .map(std::string::String::as_str)
    {
        config.set_host_value(host);
    }

    if let Some(ipv4) = cli_arguments
        .get_one::<String>("ipv4-replacement")
        .map(std::string::String::as_str)
    {
        config.set_ipv4_value(ipv4);
    }

    if cli_arguments.contains_id("flush-line") {
        config.set_flush(true);
    }

    if cli_arguments.contains_id("authuser") {
        config.set_authuser(true);
    }

    if cli_arguments.contains_id("notrim") {
        config.set_trim(false);
    }

    if cli_arguments.contains_id("nooptimize") {
        config.set_optimize(false);
    }

    if cli_arguments.contains_id("skip-invalid") {
        config.set_skip(true);
    }

    if let Some(output) = cli_arguments
        .get_one::<OsString>("output")
        .map(std::ffi::OsString::as_os_str)
    {
        ioconfig.set_output(Path::new(output));
    }

    if let Some(input) = cli_arguments.get_many::<OsString>("input") {
        for file in input.collect::<Vec<_>>() {
            ioconfig.push_input(file);
        }
    }

    alog::run(&config, &ioconfig);
}
