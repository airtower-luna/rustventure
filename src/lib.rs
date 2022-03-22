//! A tiny text adventure engine made as an exercise for learning
//! Rust.

use std::error::Error;
use std::io::{BufRead, Write};
use std::path::PathBuf;

pub mod scene;

use scene::{Effect, Scene};

/// Runtime configuration data
pub struct Config {
    /// Path of the initial scene file to load
    pub scenepath: PathBuf,
}

impl Config {
    /// Create [`Config`] from command line arguments. The second item
    /// in the iterable (after the binary name) must be the path to
    /// the initial scene file.
    pub fn new<T>(args: T) -> Result<Config, &'static str>
    where
        T: IntoIterator<Item = String>,
    {
        let path = match args.into_iter().nth(1) {
            Some(p) => p,
            None => return Err("no scene file"),
        };
        let scenepath = PathBuf::from(path);
        Ok(Config { scenepath })
    }
}

/// Run a game based on the given [`Config`].
///
/// # Arguments
///
/// * `config` - Runtime configuration as returned by [`Config::new()`]
/// * `input` - Source of user input, e.g. stdin
/// * `output` - Destination for output to the user, e.g. stdout
///
/// The last two arguments exist primarily to make the function
/// testable, but could also be used to implement some other user
/// interface.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kitten() {
        let path: PathBuf =
            [env!("CARGO_MANIFEST_DIR"), "resources", "kitten.scene"]
                .iter()
                .collect();
        let args: Vec<String> =
            vec!["test".to_string(), path.to_str().unwrap().to_string()];
        let config = Config::new(args).unwrap();

        let input = b"meow\nhug\npet";
        let mut slice = &input[..];
        let mut output = Vec::new();

        run(config, &mut slice, &mut output).unwrap();
        assert_eq!(
            vec![
                "There's a little kitten in front of you!",
                "> \"Meow!\" =^.^=",
                "> *purr*",
                "There's a kitten purring in your arms!",
                "> *purr, purr*",
                "> ",
            ],
            String::from_utf8(output)
                .unwrap()
                .lines()
                .collect::<Vec<&str>>()
        );
    }
}
