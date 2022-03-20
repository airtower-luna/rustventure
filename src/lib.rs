use std::error::Error;
use std::io;
use std::io::Write;
use std::path::PathBuf;

mod scene;

use scene::{Effect, Scene};

pub struct Config {
    pub scenepath: PathBuf,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        let path = match args.get(1) {
            Some(p) => p,
            None => return Err("no scene file"),
        };
        let scenepath = PathBuf::from(path);
        Ok(Config { scenepath })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
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
