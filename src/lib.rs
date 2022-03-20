use std::path::PathBuf;

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
