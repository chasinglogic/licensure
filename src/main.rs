// Copyright (C) 2024 Mathew Robinson <chasinglogic@gmail.com>
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, version 3.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with
// this program. If not, see <https://www.gnu.org/licenses/>.
//

extern crate chrono;
extern crate clap;
#[macro_use]
extern crate log;
extern crate regex;
extern crate serde;
extern crate serde_yaml;
extern crate textwrap;
extern crate ureq;

use std::fs::File;
use std::io::prelude::*;
use std::io::ErrorKind;
use std::path::Path;
use std::process;
use std::process::Command;

use chrono::offset::{Offset, Utc};
use clap::{App, Arg};

use config::DEFAULT_CONFIG;
use licensure::Licensure;

mod comments;
mod config;
mod licensure;
mod template;
mod utils;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const ABOUT: &str = env!("CARGO_PKG_DESCRIPTION");
const HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");

// FIXME: Possible that we should remove this functionality.
fn get_project_files() -> Vec<String> {
    let mut files = git_ls_files(Vec::new());

    let mut new_unstaged_files = git_ls_files(vec!["--others", "--exclude-standard"]);
    files.append(&mut new_unstaged_files);

    files
        .iter()
        .filter(|x| !Path::new(*x).is_symlink())
        .map(|x| x.to_string())
        .collect()
}

fn git_ls_files(extra_args: Vec<&str>) -> Vec<String> {
    match Command::new("git")
        .arg("ls-files")
        .args(extra_args)
        .output()
    {
        Ok(proc) => String::from_utf8(proc.stdout)
            .expect("git ls-files output was not UTF-8!")
            .split('\n')
            // git-ls still returns the removed files that are not committed, so we filter those out.
            .filter(|s| !s.is_empty() && Path::new(s).exists())
            .map(str::to_string)
            .collect(),
        Err(e) => {
            println!("Failed to run git ls-files. Make sure you're in a git repo.");
            println!("{}", e);
            process::exit(1)
        }
    }
}

fn main() {
    let matches = App::new("licensure")
        .version(VERSION)
        .author("Mathew Robinson <chasinglogic@gmail.com>")
        .about(
            format!(
                "{}

{}

More information is available at: {}",
                ABOUT,
                AUTHORS.replace(':', ", "),
                HOMEPAGE
            )
            .as_str(),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true),
        )
        .arg(Arg::with_name("in-place").short("i").long("in-place"))
        .arg(
            Arg::with_name("check")
                .long("check")
                .help("Checks if any file is not licensed with the given config"),
        )
        .arg(
            Arg::with_name("exclude")
                .short("e")
                .long("exclude")
                .takes_value(true)
                .value_name("REGEX")
                .help("A regex which will be used to determine what files to ignore."),
        )
        .arg(Arg::with_name("project").long("project").short("p").help(
            "When specified will license the current project files as returned by git ls-files",
        ))
        .arg(
            Arg::with_name("generate-config")
                .long("generate-config")
                .help("Generate a default licensure config file"),
        )
        .arg(
            Arg::with_name("FILES")
                .multiple(true)
                .help("Files to license, ignored if --project is supplied"),
        )
        .get_matches();

    match matches.occurrences_of("verbose") {
        0 => (),
        x => simplelog::SimpleLogger::init(
            if x > 3 {
                simplelog::LevelFilter::Trace
            } else if x > 2 {
                simplelog::LevelFilter::Debug
            } else {
                simplelog::LevelFilter::Info
            },
            simplelog::ConfigBuilder::new()
                .set_time_level(simplelog::LevelFilter::Debug)
                .set_thread_level(simplelog::LevelFilter::Debug)
                .set_target_level(simplelog::LevelFilter::Debug)
                .set_location_level(simplelog::LevelFilter::Trace)
                .set_time_offset(Utc.fix())
                .build(),
        )
        .unwrap(),
    };

    if matches.is_present("generate-config") {
        let mut f = match File::create(".licensure.yml") {
            Ok(f) => f,
            Err(e) => {
                println!("Unable to create .licensure.yml: {}", e);
                process::exit(1);
            }
        };

        if let Err(e) = f.write_all(DEFAULT_CONFIG.as_bytes()) {
            println!("Unable to write to .licensure.yml: {}", e);
            process::exit(1);
        }

        process::exit(0);
    }

    let files: Vec<String> = if matches.is_present("project") {
        get_project_files()
    } else {
        matches
            .values_of("FILES")
            .expect("ERROR: Must provide files to license either as matches or via --project")
            .map(str::to_string)
            .collect()
    };

    let mut config = match config::load_config() {
        Ok(c) => c,
        Err(e) => {
            if ErrorKind::NotFound == e.kind() {
                println!("No config file found, generate one with licensure --generate-config");
            } else {
                println!("Error loading config file: {}", e);
            }

            process::exit(1);
        }
    };

    if let Some(exclude) = matches.value_of("exclude") {
        config.add_exclude(exclude);
    }

    if matches.is_present("in-place") {
        config.change_in_place = true;
    }

    let licensure = Licensure::new(config).with_check_mode(matches.is_present("check"));
    match licensure.license_files(&files) {
        Err(e) => {
            println!("Failed to license files: {}", e);
            process::exit(1);
        }
        Ok(stats) => {
            if matches.is_present("check")
                && !(stats.files_not_licensed.is_empty()
                    && stats.files_needing_license_update.is_empty())
            {
                if !stats.files_needing_license_update.is_empty() {
                    eprintln!(
                        "The following {} files' licenses need to be updated",
                        stats.files_needing_license_update.len()
                    );
                    for file in stats.files_needing_license_update {
                        eprintln!("{}", file);
                    }
                }

                if !stats.files_not_licensed.is_empty() {
                    eprintln!(
                        "The following {} files were not licensed with the given config.",
                        stats.files_not_licensed.len()
                    );
                    for file in stats.files_not_licensed {
                        eprintln!("{}", file);
                    }
                }

                process::exit(1);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_project_files() {
        assert!(!get_project_files().is_empty())
    }
}
