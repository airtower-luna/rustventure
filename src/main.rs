use lazy_static::lazy_static;
use regex::Regex;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug)]
struct Scene {
    path: PathBuf,
    description: String,
    actions: Vec<Action>,
}

impl Scene {
    fn load(path: PathBuf) -> Result<Scene, Box<dyn Error>> {
        let f = File::open(&path)?;
        let mut reader = io::BufReader::new(f);

        let mut desc = String::new();
        let mut actions = Vec::new();

        // Read the scene description: Everything until the first line
        // that can be parsed as an action.
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line)? == 0 {
                break;
            }
            match Action::from(line.trim()) {
                Ok(a) => {
                    actions.push(a);
                    break;
                }
                Err(_) => desc.push_str(&line),
            }
        }

        // Read remaining actions
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line)? == 0 {
                break;
            }
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            actions.push(Action::from(line)?);
        }

        Ok(Scene {
            path,
            description: desc,
            actions,
        })
    }

    fn get_action(&self, input: &str) -> Option<&Action> {
        for a in &self.actions {
            if a.expression.is_match(input) {
                return Some(&a);
            }
        }
        None
    }

    fn load_next(&self, name: &str) -> Result<Scene, Box<dyn Error>> {
        let mut path = self.path.clone();
        path.set_file_name(format!("{}.scene", name));
        Ok(Scene::load(path)?)
    }
}

#[derive(Debug)]
struct Action {
    expression: Regex,
    effect: Effect,
}

impl Action {
    fn from(line: &str) -> Result<Action, Box<dyn Error>> {
        lazy_static! {
            static ref ACTION_RE: Regex = Regex::new(r"^!(\w+):(.*)\s->\s(\w+)\s(.*)$").unwrap();
        }
        let c = ACTION_RE
            .captures(&line)
            .ok_or(format!("invalid action line: {}", line))?;
        // unwrap because the line above already fails if there's no match
        let kind = c.get(1).unwrap().as_str();
        let expression = c.get(2).unwrap().as_str();
        let action = c.get(3).unwrap().as_str();
        let argument = c.get(4).unwrap().as_str();

        let expr = if kind == "kw" {
            Regex::new(&format!("^{}$", regex::escape(&expression)))?
        } else {
            Regex::new(&expression)?
        };

        let effect = if action == "scene" {
            Effect::Change(argument.to_string())
        } else {
            Effect::Output(argument.to_string())
        };

        Ok(Action {
            expression: expr,
            effect,
        })
    }
}

#[derive(Debug)]
enum Effect {
    Output(String),
    Change(String),
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args();
    let scenepath = PathBuf::from(args.nth(1).ok_or("No scene file!")?);

    let mut scene = Scene::load(scenepath)?;

    print!("{}", &scene.description);
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
            match &a.effect {
                Effect::Output(s) => println!("{}", s),
                Effect::Change(s) => {
                    scene = scene.load_next(s)?;
                    print!("{}", &scene.description);
                    io::stdout().flush()?;
                }
            }
        }
    }

    Ok(())
}
