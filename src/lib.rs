#![doc = include_str!("../README.md")]

use std::error;
use std::fmt;
use std::io::{BufRead, Write};
use std::path::PathBuf;

use clap::{arg, command, ArgMatches};

pub mod adventure;
pub mod scene;

use scene::{Effect, Scene};

/// Runtime configuration data
pub struct Config {
    /// Path of the initial scene file to load, or directory to search
    /// for adventures
    pub scenepath: PathBuf,
}

impl Config {
    /// Create [`Config`] from command line arguments, or exit with
    /// help message and error code.
    pub fn parse() -> Config {
        let m = command!()
            .arg(
                arg!([scene] "scene file to load, or directory to search \
                              for adventures")
                .default_value("."),
            )
            .get_matches();
        Config::from(m)
    }
}

impl From<ArgMatches> for Config {
    fn from(m: ArgMatches) -> Self {
        Config {
            scenepath: PathBuf::from(m.value_of("scene").unwrap()),
        }
    }
}

#[derive(Debug)]
struct Error {
    msg: String,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

/// Run a game based on the given [`Config`].
///
/// # Arguments
///
/// * `config` - Runtime configuration as returned by [`Config::parse()`]
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
) -> Result<(), Box<dyn error::Error>>
where
    R: BufRead,
    W: Write,
{
    // If the configured path is a directory, search it for
    // adventures. Otherwise try to load it as a scene file.
    let mut scene = if config.scenepath.is_dir() {
        let mut adventures = adventure::search(&config.scenepath)?;
        if adventures.len() == 0 {
            return Err(Box::new(Error {
                msg: "no adventures found".to_string(),
            }) as Box<dyn error::Error>);
        } else if adventures.len() == 1 {
            let a = adventures.swap_remove(0);
            write!(output, "Starting adventure: {}\n\n", a)?;
            a.start()?
        } else {
            write!(output, "Please select an adventure by number:\n")?;
            for (i, a) in adventures.iter().enumerate() {
                write!(output, "{}: {}\n", i + 1, a)?;
            }
            let mut i: Option<usize> = None;
            let mut line = String::new();
            while i.is_none() {
                write!(output, "> ")?;
                output.flush()?;
                input.read_line(&mut line)?;
                i = line
                    .trim()
                    .parse()
                    .ok()
                    .filter(|i| i > &0 && i <= &adventures.len());
                if i.is_none() {
                    line.clear();
                    write!(
                        output,
                        "Please select a valid number (1 to {})!\n",
                        adventures.len()
                    )?;
                }
            }
            adventures.swap_remove(i.unwrap() - 1).start()?
        }
    } else {
        Scene::load(config.scenepath)?
    };

    write!(output, "{}", scene)?;
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
                    write!(output, "{}", scene)?;
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
        let config = Config { scenepath: path };

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
