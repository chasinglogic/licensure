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

const DEFAULT_PATTERNS: &'static str = "(\\.gitignore|.*lock|\\.licensure\\.yml)";

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
                break;
            }

            if !cwd.pop() {
                break;
            }
        }

        if cwd.exists() {
            return Some(cwd);
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
        .version("0.1.0")
        .author("Mathew Robinson <chasinglogic@gmail.com>")
        .about(
            "
Manage FOSS licenses in your projects.

Copyright (C) 2018 Mathew Robinson <chasinglogic@gmail.com>

This program is free software: you can redistribute it and/or modify
it under the terms of the Apache Version 2.0 License

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.

You should have recieved a copy of the license with this software if
not you can view it here: https://www.apache.org/licenses/LICENSE-2.0",
        )
        .subcommand(
            SubCommand::with_name("license")
                .arg(Arg::with_name("short").short("s").long("short"))
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
                        .help("SPDX license identifier to license files with."),
                )
                .arg(
                    Arg::with_name("author")
                        .short("a")
                        .long("author")
                        .help("Full name of copyright owner / source code author.")
                )
                .arg(
                    Arg::with_name("email")
                        .short("e")
                        .long("email")
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
