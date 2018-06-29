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

extern crate clap;
extern crate licensure;
extern crate regex;
extern crate serde_yaml;

use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::Error;
use std::path::PathBuf;
use std::process;
use std::process::Command;

use clap::{App, Arg, SubCommand};
use regex::Regex;

use licensure::comments;
use licensure::licenses::Config;

const DEFAULT_PATTERNS: &'static str =
    "(\\.gitignore|.*lock|\\.licensure\\.yml|README.*|LICENSE.*)";

fn get_project_files() -> Box<Vec<String>> {
    match Command::new("git").arg("ls-files").output() {
        Ok(proc) => Box::new(
            String::from_utf8(proc.stdout)
                .unwrap()
                .split("\n")
                .filter(|s| *s != "")
                .map(str::to_string)
                .collect(),
        ),
        Err(e) => {
            println!("Failed to run git ls-files. Make sure you're in a git repo.");
            println!("{}", e);
            process::exit(1)
        }
    }
}

fn find_config_file() -> Option<PathBuf> {
    if let Ok(mut cwd) = env::current_dir() {
        loop {
            cwd.push(".git");
            if cwd.exists() {
                cwd.pop();
                cwd.push(".licensure.yml");
                return Some(cwd);
            }

            // Pop the .git directory we added
            cwd.pop();

            // Move up a directory checking if we have hit root yet
            if !cwd.pop() {
                break;
            }
        }
    }

    if let Some(mut global) = env::home_dir() {
        global.push(".licensure");
        global.push("config.yml");
        if global.exists() {
            return Some(global);
        }
    }

    None
}

fn license_file(filename: &str, uncommented: &str) -> Result<(), Error> {
    let mut f = File::open(filename)?;
    let mut content = String::new();
    f.read_to_string(&mut content)?;

    let filetype = comments::get_filetype(&filename);
    let commenter = comments::get_commenter(&filetype);
    let mut header = commenter.comment(uncommented);

    if content.contains(&header) {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("{} already licensed", filename),
        ));
    }

    header.push_str(&content);

    f = File::create(filename)?;
    f.write_all(header.as_bytes()).map(|_x| ())
}

fn main() {
    let matches = App::new("licensure")
        .version("0.1.2")
        .author("Mathew Robinson <chasinglogic@gmail.com>")
        .about(
            "
Manage licenses in your projects.

Copyright 2018 Mathew Robinson <chasinglogic@gmail.com>. All rights reserved.

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.")
        .subcommand(
            SubCommand::with_name("license")
                .about("Apply license headers to source files")
                .arg(
                    Arg::with_name("exclude")
                     .short("e")
                     .long("exclude")
                     .takes_value(true)
                     .value_name("REGEX")
                     .help("A regex which will be used to determine what files to ignore.")
                )
                .arg(
                    Arg::with_name("project")
                        .long("project")
                        .short("p")
                        .help("When specified will license the current project files as returned by git ls-files")
                )
                .arg(
                    Arg::with_name("ident")
                        .short("i")
                        .long("ident")
                        .takes_value(true)
                        .value_name("SPDX_IDENTIFIER")
                        .help("SPDX license identifier to license files with."),
                )
                .arg(
                    Arg::with_name("author")
                        .short("a")
                        .long("author")
                        .takes_value(true)
                        .value_name("AUTHOR_NAME")
                        .help("Full name of copyright owner / source code author.")
                )
                .arg(
                    Arg::with_name("email")
                        .short("m")
                        .long("email")
                        .takes_value(true)
                        .value_name("AUTHOR_EMAIL")
                        .help("Email of the copyright owner / source code author.")
                )
                .arg(
                    Arg::with_name("FILES")
                        .multiple(true)
                        .help("Files to license, ignored if --project is supplied"),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        ("license", Some(args)) => {
            let files: Vec<String> = if args.is_present("project") {
                *get_project_files()
            } else {
                args.values_of("FILES")
                    .expect("ERROR: Must provide files to license either as args or via --project")
                    .map(str::to_string)
                    .collect()
            };

            let mut config = match find_config_file() {
                Some(file) => {
                    let mut f = File::open(file).expect("FAIL");
                    let mut content = String::new();
                    f.read_to_string(&mut content).expect("FAIL");
                    serde_yaml::from_str(&content).unwrap()
                }
                None => Config::new("", ""),
            };

            if let Some(author) = args.value_of("author") {
                config.author = author.to_string();
            } else if &config.author == "" {
                println!("ERROR: --author is a required flag");
                process::exit(1);
            };

            if let Some(ident) = args.value_of("ident") {
                config.ident = ident.to_string();
            } else if &config.ident == "" {
                println!("ERROR: --ident is a required flag");
                process::exit(1);
            };

            if let Some(email) = args.value_of("email") {
                config = config.with_email(email);
            }

            if let Some(year) = args.value_of("year") {
                config = config.with_year(year);
            }

            let mut final_pat = DEFAULT_PATTERNS.to_string();
            if let Some(exclude) = args.value_of("exclude") {
                final_pat.push_str("|");
                final_pat.push_str(exclude);
            }

            let cfg_pat = config.exclude_pat();
            if cfg_pat != "" {
                final_pat.push_str("|");
                final_pat.push_str(&cfg_pat);
            }

            let rgx = match Regex::new(&final_pat) {
                Ok(r) => r,
                Err(e) => {
                    println!("ERROR: Unable to compile regex: {}", e);
                    process::exit(1);
                }
            };

            let header = config.render();
            for file in files {
                if rgx.is_match(&file) {
                    println!("Skipping excluded file: {}", file);
                    continue;
                }

                println!("Licensing file: {}", file);
                if let Err(err) = license_file(&file, &header) {
                    println!("{}", err);
                }
            }
        }
        _ => {
            println!("ERROR: Unknown command");
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_project_files() {
        assert!(get_project_files().len() != 0)
    }
}
