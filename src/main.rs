use std::env;
use std::process;
use alog::Config;

fn main() {
    let mut config = Config::new().unwrap_or_else(|err| {
        eprintln!("Problem generating default config: {}", err);
        process::exit(1);
    });

    config.parse_args(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    alog::run(config);
}
