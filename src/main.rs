// Copyright 2018 Mathew Robinson <chasinglogic@gmail.com>. All rights reserved.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

extern crate chrono;
extern crate clap;
#[macro_use]
extern crate log;
extern crate regex;
extern crate reqwest;
extern crate serde;
extern crate serde_yaml;
extern crate textwrap;

mod comments;
mod config;
mod licensure;
mod template;

use std::fs::File;
use std::io::prelude::*;
use std::io::ErrorKind;
use std::process;
use std::process::Command;

use clap::{App, Arg};
use chrono::offset::{Offset, Utc};

use config::DEFAULT_CONFIG;
use futures::executor::block_on;
use licensure::Licensure;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const ABOUT: &str = env!("CARGO_PKG_DESCRIPTION");
const HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");

// FIXME: Possible that we should remove this functionality.
fn get_project_files() -> Vec<String> {
    match Command::new("git").arg("ls-files").output() {
        Ok(proc) => String::from_utf8(proc.stdout)
            .unwrap()
            .split('\n')
            .filter(|s| !s.is_empty())
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
                AUTHORS.replace(":", ", "),
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
            if x > 2 {
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

    let done = async {
        match Licensure::new(config).license_files(&files).await {
            Err(e) => {
                println!("Failed to license files: {}", e);
                process::exit(1);
            }
            Ok(files_not_licensed) => {
                if matches.is_present("check") && !files_not_licensed.is_empty() {
                    eprintln!("The following files were not licensed with the given config.");
                    for file in files_not_licensed {
                        eprintln!("{}", file);
                    }
                    process::exit(1);
                }
            }
        }
    };
    block_on(done);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_project_files() {
        assert!(get_project_files().len() != 0)
    }
}
