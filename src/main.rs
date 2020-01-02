use alog::{run, Config, IOConfig};

extern crate clap;
use clap::{App, Arg};

fn main() {
    let default_config = Config::default();
    let mut config = Config::default();
    let mut ioconfig = IOConfig::default();

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
                .help("Sets the input file(s) to use")
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

    if let Some(output) = cli_arguments.value_of("output") {
        ioconfig.set_output(output);
    }

    if let Some(input) = cli_arguments.values_of("input") {
        for file in input {
            ioconfig.push_input(&file);
        }
    }

    run(&ioconfig, &config);
}
