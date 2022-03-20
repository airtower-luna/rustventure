use std::error::Error;
use std::io::{BufRead, Write};
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

pub fn run<R, W>(
    config: Config,
    input: &mut R,
    output: &mut W,
) -> Result<(), Box<dyn Error>>
where
    R: BufRead,
    W: Write,
{
    let mut scene = Scene::load(config.scenepath)?;

    write!(output, "{}", scene.description())?;
    output.flush()?;

    loop {
        write!(output, "> ")?;
        output.flush()?;

        let mut line = String::new();
        if input.read_line(&mut line)? == 0 {
            write!(output, "\n")?;
            break;
        }

        if let Some(a) = scene.get_action(line.trim()) {
            match a.effect() {
                Effect::Output(s) => write!(output, "{}\n", s)?,
                Effect::Change(s) => {
                    scene = scene.load_next(s)?;
                    write!(output, "{}", scene.description())?;
                    output.flush()?;
                }
            }
        }
    }

    Ok(())
}
