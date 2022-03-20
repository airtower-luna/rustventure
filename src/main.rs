use std::env;
use std::process;

use rustventure::Config;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).unwrap_or_else(|err| {
        eprintln!("Invalid arguments: {}", err);
        process::exit(2);
    });

    if let Err(err) = rustventure::run(config) {
        eprintln!("Error: {}", err);
        process::exit(1);
    }
}
