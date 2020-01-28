extern crate clap;
use clap::{App, Arg};

fn main() {
    let default_config = alog::Config::default();
    let mut config = alog::Config::default();
    let mut ioconfig = alog::IOConfig::default();

    let cli_arguments = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("Mangle common / combined logs")
        .arg(
            Arg::with_name("ipv4-replacement")
                .short("4")
                .long("ipv4-replacement")
                .value_name("ipv4-replacement")
                .default_value(&default_config.get_ipv4_value())
                .help("Sets IPv4 replacement string")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("ipv6-replacement")
                .short("6")
                .long("ipv6-replacement")
                .value_name("ipv6-replacement")
                .default_value(&default_config.get_ipv6_value())
                .help("Sets IPv6 replacement string")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("host-replacement")
                .long("host-replacement")
                .value_name("host-replacement")
                .default_value(&default_config.get_host_value())
                .help("Sets host replacement string")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("skip-invalid")
                .short("s")
                .long("skip-invalid")
                .value_name("skip-invalid")
                .help("Skip invalid lines")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("flush-line")
                .short("f")
                .long("flush-line")
                .value_name("flush-line")
                .help("Flush output on every line")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("FILE")
                .help("Sets output file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("input")
                .value_name("INPUT")
                .help("The input file(s) to use")
                .index(1)
                .multiple(true),
        )
        .get_matches();

    if let Some(ipv4) = cli_arguments.value_of("ipv4-replacement") {
        config.set_ipv4_value(ipv4);
    }

    if let Some(ipv6) = cli_arguments.value_of("ipv6-replacement") {
        config.set_ipv6_value(ipv6);
    }

    if let Some(host) = cli_arguments.value_of("host-replacement") {
        config.set_host_value(host);
    }

    if let Some(ipv4) = cli_arguments.value_of("ipv4-replacement") {
        config.set_ipv4_value(ipv4);
    }

    if cli_arguments.is_present("flush-line") {
        config.set_flush(true);
    }

    if cli_arguments.is_present("skip-invalid") {
        config.set_skip(true);
    }

    if let Some(output) = cli_arguments.value_of_os("output") {
        ioconfig.set_output_os(output);
    }

    if let Some(input) = cli_arguments.values_of_os("input") {
        for file in input {
            ioconfig.push_input_os(file);
        }
    }

    alog::run(&config, &ioconfig);
}
