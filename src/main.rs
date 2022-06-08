use clap::Parser;
use std::io;
use std::process;

use rustventure::Config;

fn main() {
    let config = Config::parse();

    let stdin = io::stdin();
    let mut input = stdin.lock();
    let mut stdout = io::stdout();

    if let Err(err) = rustventure::run(config, &mut input, &mut stdout) {
        eprintln!("Error: {}", err);
        process::exit(1);
    }
}
