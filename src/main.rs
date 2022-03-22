use std::env;
use std::io;
use std::process;

use rustventure::Config;

fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Invalid arguments: {}", err);
        process::exit(2);
    });

    let stdin = io::stdin();
    let mut input = stdin.lock();
    let mut stdout = io::stdout();

    if let Err(err) = rustventure::run(config, &mut input, &mut stdout) {
        eprintln!("Error: {}", err);
        process::exit(1);
    }
}
