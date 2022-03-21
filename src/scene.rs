use lazy_static::lazy_static;
use regex::Regex;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

#[derive(Debug)]
pub struct Scene {
    path: PathBuf,
    description: String,
    actions: Vec<Action>,
}

impl Scene {
    pub fn load(path: PathBuf) -> Result<Scene, Box<dyn Error>> {
        let f = File::open(&path)?;
        let mut reader = BufReader::new(f);

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

    pub fn get_action(&self, input: &str) -> Option<&Action> {
        for a in &self.actions {
            if a.expression.is_match(input) {
                return Some(&a);
            }
        }
        None
    }

    pub fn load_next(&self, name: &str) -> Result<Scene, Box<dyn Error>> {
        let mut path = self.path.clone();
        path.set_file_name(format!("{}.scene", name));
        Ok(Scene::load(path)?)
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Debug)]
pub struct Action {
    expression: Regex,
    effect: Effect,
}

impl Action {
    fn from(line: &str) -> Result<Action, Box<dyn Error>> {
        lazy_static! {
            static ref ACTION_RE: Regex =
                Regex::new(r"^!(\w+):(.*)\s->\s(\w+)\s(.*)$").unwrap();
        }
        let c = ACTION_RE
            .captures(&line)
            .ok_or(format!("invalid action line: {}", line))?;
        let kind = &c[1];
        let expression = &c[2];
        let action = &c[3];
        let argument = &c[4];

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

    pub fn effect(&self) -> &Effect {
        &self.effect
    }
}

#[derive(Debug, PartialEq)]
pub enum Effect {
    Output(String),
    Change(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kitten_scene() -> Scene {
        let p: PathBuf =
            [env!("CARGO_MANIFEST_DIR"), "resources", "kitten.scene"]
                .iter()
                .collect();
        Scene::load(p).unwrap()
    }

    #[test]
    fn parse_action() {
        let a = Action::from("!kw:meow -> print \"Meow!\" =^.^=").unwrap();
        assert_eq!(a.effect, Effect::Output("\"Meow!\" =^.^=".to_string()));
        assert_eq!(a.expression.as_str(), r"^meow$");
        assert!(a.expression.is_match("meow"));
    }

    #[test]
    #[should_panic(expected = "invalid action line:")]
    fn load_invalid_action() {
        Action::from("Meow, I'm a little kitten!").unwrap();
    }

    #[test]
    fn load_scene() {
        let s = kitten_scene();
        assert_eq!(
            s.description().trim(),
            "There's a little kitten in front of you!"
        );
        assert!(s.get_action("bark").is_none());
        assert_eq!(
            s.get_action("meow").unwrap().effect,
            Effect::Output("\"Meow!\" =^.^=".to_string())
        );
        assert_eq!(
            s.get_action("hug").unwrap().effect,
            Effect::Change("cuddle_cat".to_string())
        );
    }

    #[test]
    fn change_scene() {
        let mut s = kitten_scene();
        let a = s.get_action("hug").unwrap();
        assert_eq!(a.effect, Effect::Change("cuddle_cat".to_string()));
        match a.effect() {
            Effect::Change(t) => s = s.load_next(t).unwrap(),
            _ => panic!("unexpected effect"),
        }
        assert_eq!(
            s.description().trim(),
            "*purr*\nThere's a kitten purring in your arms!"
        );
        assert_eq!(
            s.get_action("pet").unwrap().effect,
            Effect::Output("*purr, purr*".to_string())
        );
        assert_eq!(
            s.get_action("down").unwrap().effect,
            Effect::Change("kitten".to_string())
        );
        assert_eq!(
            s.get_action("set down").unwrap().effect,
            Effect::Change("kitten".to_string())
        );
        assert_eq!(
            s.get_action("release").unwrap().effect,
            Effect::Change("kitten".to_string())
        );
    }
}
