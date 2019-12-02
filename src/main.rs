use alog::run;

extern crate clap;
use clap::{App, Arg};

fn main() {
    let cli_arguments = App::new("alog")
                            .version("0.1.1")
                            .about("Mangle common / combined logs")
                            .arg(Arg::with_name("ipv4-replacement")
                                .short("4")
                                .long("ipv4-replacement")
                                .value_name("ipv4-replacement")
                                .help("Sets IPv4 replacement string")
                                .takes_value(true))
                            .arg(Arg::with_name("ipv6-replacement")
                                .short("6")
                                .long("ipv6-replacemment")
                                .value_name("ipv6-replacement")
                                .help("Sets IPv6 replacement string")
                                .takes_value(true))
                            .arg(Arg::with_name("hostname-replacement")
                                .long("hostname-replacemment")
                                .value_name("hostname-replacement")
                                .help("Sets hostname replacement string")
                                .takes_value(true))
                            .arg(Arg::with_name("INPUT")
                                .help("Sets the input file(s) to use")
                                .index(1)
                                .multiple(true))
                            .get_matches();

    run(&cli_arguments);
}
