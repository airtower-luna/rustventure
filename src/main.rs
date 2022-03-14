use std::env;
use std::error::Error;
use std::io;
use std::io::Write;
use std::path::PathBuf;

mod scene;

use scene::{Effect, Scene};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    let scenepath = PathBuf::from(args.nth(1).ok_or("No scene file!")?);

    let mut scene = Scene::load(scenepath)?;

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
