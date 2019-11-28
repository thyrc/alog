use std::env;
use std::process;
use alog::{Config, run};

fn main() {
    let mut config = Config::new();

    config.parse_args(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    run(config);
}
