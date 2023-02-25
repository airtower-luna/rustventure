//! Handle adventure metadata and support searching adventures in the
//! directory tree.

use lazy_static::lazy_static;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use yaml_rust::{Yaml, YamlLoader};

use crate::scene::Scene;

#[derive(Debug, PartialEq, Eq)]
pub struct Adventure {
    name: String,
    author: String,
    version: Option<String>,
    start: PathBuf,
}

// Maybe these macros aren't necessary, but they are good practice. :D

macro_rules! get_optional_field {
    ($hash:ident, $field:ident) => {{
        lazy_static! {
            static ref FIELD: Yaml = Yaml::from_str(stringify!($field));
        }
        $hash
            .get(&FIELD)
            .and_then(|f| f.as_str())
            .map(|f| f.to_string())
    }};
}

macro_rules! get_field {
    ($hash:ident, $field:ident) => {
        get_optional_field!($hash, $field).ok_or(stringify!(missing $field))
    };
}

impl TryFrom<&Path> for Adventure {
    type Error = Box<dyn Error>;

    fn try_from(p: &Path) -> Result<Self, Self::Error> {
        let s = fs::read_to_string(p)?;
        let docs = YamlLoader::load_from_str(&s)?;
        let about = docs
            .get(0)
            .ok_or("no data in file")?
            .as_hash()
            .ok_or("invalid data, must be hash")?;

        Ok(Adventure {
            name: get_field!(about, name)?,
            author: get_field!(about, author)?,
            version: get_optional_field!(about, version),
            start: {
                let mut path = p.to_path_buf();
                path.set_file_name(
                    get_optional_field!(about, start)
                        .unwrap_or_else(|| "start.scene".to_string()),
                );
                path
            },
        })
    }
}

impl fmt::Display for Adventure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "\"{}\" by {}{}",
            self.name,
            self.author,
            self.version
                .as_ref()
                .map_or_else(String::new, |s| format! {" (version {})", s})
        )
    }
}

impl Adventure {
    /// Load the start scene of the adventure, consuming `self` to
    /// avoid copying the `PathBuf`.
    pub fn start(self) -> Result<Scene, Box<dyn Error>> {
        Scene::load(self.start)
    }
}

/// Find adventures inside the given `dir`. Assumes that every
/// directory containing an `about.yaml` or `about.yml` file is an
/// adventure.
pub fn search(dir: &Path) -> Result<Vec<Adventure>, Box<dyn Error>> {
    let mut res = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            res.extend(search(&path)?);
            continue;
        }
        let name = entry.file_name();
        if name == "about.yml" || name == "about.yaml" {
            res.push(Adventure::try_from(&path as &Path)?);
        }
    }
    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kitten_adventure() -> Adventure {
        let start: PathBuf =
            [env!("CARGO_MANIFEST_DIR"), "resources", "kitten.scene"]
                .iter()
                .collect();
        Adventure {
            name: "A cuddly kitten".to_string(),
            author: "Fiona".to_string(),
            version: Some("1.0".to_string()),
            start,
        }
    }

    #[test]
    fn load_adventure() {
        let path: PathBuf =
            [env!("CARGO_MANIFEST_DIR"), "resources", "about.yaml"]
                .iter()
                .collect();
        let about = Adventure::try_from(&path as &Path).unwrap();
        assert_eq!(about, kitten_adventure());
        assert_eq!(
            format!("{}", about),
            "\"A cuddly kitten\" by Fiona (version 1.0)"
        );
        let scene = about.start().unwrap();
        assert_eq!(
            format!("{}", scene).trim(),
            "There's a little kitten in front of you!"
        );
    }

    #[test]
    fn format_partial() {
        let about = Adventure {
            name: "Test Adventure".to_string(),
            author: "Me".to_string(),
            version: None,
            start: PathBuf::from("test.scene"),
        };
        assert_eq!(format!("{}", about), "\"Test Adventure\" by Me");
    }

    #[test]
    fn search_adventure() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let adventures = search(&dir as &Path).unwrap();
        assert_eq!(adventures, vec![kitten_adventure()]);
    }
}
