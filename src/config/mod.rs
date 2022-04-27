use std::env;
use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::process;

use regex::RegexSet;
use serde::Deserialize;

mod comment;
mod default;
mod license;

pub use default::DEFAULT_CONFIG;

use crate::comments::Comment;
use crate::config::comment::get_filetype;
use crate::config::comment::Config as CommentConfig;
use crate::config::license::Config as LicenseConfig;
use crate::template::Template;

fn def_change_in_place() -> bool {
    false
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "def_change_in_place")]
    pub change_in_place: bool,
    pub excludes: RegexList,
    pub licenses: LicenseConfigList,
    pub comments: CommentConfigList,
}

impl Config {
    pub fn add_exclude(&mut self, pat: &str) {
        self.excludes.add_exclude(pat);
    }
}

#[derive(Deserialize)]
#[serde(from = "Vec<String>")]
pub struct RegexList {
    regex: RegexSet,
}

impl RegexList {
    pub fn is_match(&self, s: &str) -> bool {
        self.regex.is_match(s)
    }

    pub fn add_exclude(&mut self, pat: &str) {
        let mut old_pats = Vec::from(self.regex.patterns());
        let mut new_pats = vec![pat.to_string()];
        new_pats.append(&mut old_pats);
        self.regex = match RegexSet::new(&new_pats) {
            Ok(r) => r,
            Err(e) => {
                println!("Failed to compile exclude pattern: {}", e);
                process::exit(1);
            }
        };
    }
}

impl From<Vec<String>> for RegexList {
    fn from(rgxs: Vec<String>) -> RegexList {
        RegexList {
            regex: match RegexSet::new(&rgxs) {
                Ok(r) => r,
                Err(e) => {
                    println!("Failed to compile exclude pattern: {}", e);
                    process::exit(1);
                }
            },
        }
    }
}

#[derive(Deserialize)]
#[serde(from = "Vec<CommentConfig>")]
pub struct CommentConfigList {
    cfgs: Vec<CommentConfig>,
}

impl From<Vec<CommentConfig>> for CommentConfigList {
    fn from(cfgs: Vec<CommentConfig>) -> CommentConfigList {
        CommentConfigList { cfgs }
    }
}

impl CommentConfigList {
    pub fn get_commenter(&self, filename: &str) -> (CommentConfig, Box<dyn Comment>) {
        let file_type = get_filetype(filename);

        for c in &self.cfgs {
            if c.matches(file_type) {
                let cfg = c.clone();
                return (cfg, c.commenter());
            }
        }

        let def = CommentConfig::default();
        (def, CommentConfig::default().commenter())
    }
}

#[derive(Deserialize)]
#[serde(from = "Vec<LicenseConfig>")]
pub struct LicenseConfigList {
    cfgs: Vec<LicenseConfig>,
}

impl LicenseConfigList {
    pub async fn get_template(&self, filename: &str) -> Option<Template> {
        for cfg in &self.cfgs {
            if cfg.file_is_match(filename) {
                return Some(cfg.get_template().await);
            }
        }

        None
    }
}

impl From<Vec<LicenseConfig>> for LicenseConfigList {
    fn from(cfgs: Vec<LicenseConfig>) -> LicenseConfigList {
        LicenseConfigList { cfgs }
    }
}

pub fn xdg_config_dir() -> Option<PathBuf> {
    match env::var("XDG_CONFIG_HOME") {
        Ok(d) => Some(PathBuf::from(d)),
        Err(_) => match env::var("HOME") {
            Ok(home) => {
                let mut home_dir = PathBuf::from(home);
                home_dir.push(".config");
                Some(home_dir)
            }
            Err(_) => None,
        },
    }
}

/// Walk up from the current working directory searching for
/// the first .licensure.yml config file available else find the
/// global config file.
fn find_config_file() -> Option<PathBuf> {
    if let Ok(mut cwd) = env::current_dir() {
        loop {
            cwd.push(".licensure.yml");
            if cwd.exists() {
                return Some(cwd);
            }

            // Pop the .licensure.yml file we added
            cwd.pop();

            // Move up a directory checking if we have hit root yet
            if !cwd.pop() {
                break;
            }
        }
    }

    if let Some(mut global) = xdg_config_dir() {
        global.push(".licensure");
        global.push("config.yml");
        if global.exists() {
            return Some(global);
        }
    }

    None
}

pub fn load_config() -> Result<Config, io::Error> {
    match find_config_file() {
        Some(path) => {
            let f = File::open(path.clone())?;
            match serde_yaml::from_reader(f) {
                Ok(c) => Ok(c),
                Err(e) => Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Invalid YAML in {}: {}", path.display(), e),
                )),
            }
        }
        None => Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Config file not found",
        )),
    }
}
