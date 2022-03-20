use std::env;
use std::error::Error;
use std::io;
use std::io::Write;
use std::process;

mod scene;

use rustventure::Config;
use scene::{Effect, Scene};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).unwrap_or_else(|err| {
        eprintln!("Invalid arguments: {}", err);
        process::exit(2);
    });

    let mut scene = Scene::load(config.scenepath)?;

    print!("{}", scene.description());
    io::stdout().flush()?;

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        if io::stdin().read_line(&mut input)? == 0 {
            println!();
            break;
        }

        if let Some(a) = scene.get_action(input.trim()) {
            match a.effect() {
                Effect::Output(s) => println!("{}", s),
                Effect::Change(s) => {
                    scene = scene.load_next(s)?;
                    print!("{}", scene.description());
                    io::stdout().flush()?;
                }
            }
        }
    }

    Ok(())
}
