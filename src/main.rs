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
use std::io::ErrorKind;
use std::io::prelude::*;
use std::path::Path;
use std::process;
use std::process::Command;

use clap::Parser;

use config::DEFAULT_CONFIG;
use licensure::Licensure;

mod comments;
mod config;
mod licensure;
mod template;
mod utils;

// FIXME: Possible that we should remove this functionality.
fn get_project_files() -> Vec<String> {
    let mut files = git_ls_files(Vec::new());

    let mut new_unstaged_files = git_ls_files(vec!["--others", "--exclude-standard"]);
    files.append(&mut new_unstaged_files);

    // If there is a file symlink to outside the project directory we probably
    // don't want to modify it (it'd be surprising to have external
    // modifications), and if it's to within the project then we'll modify it
    // when we come across the "real" file. Furthermore, allowing symlinks adds
    // the possibility that we'll have ambiguity (or a it's-never-happy fight)
    // if the symlink has a different file extension than the file it points at.
    files.retain(|x| !Path::new(x).is_symlink());
    files
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

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(help = "Files to license, ignored if --project is supplied")]
    files: Vec<String>,

    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[arg(short, long)]
    in_place: bool,

    #[arg(short, long)]
    check: bool,

    #[arg(
        short,
        long,
        help = "A regex which will be used to determine what files to ignore."
    )]
    exclude: Option<String>,

    #[arg(
        short,
        long,
        help = "When specified will license the current project files as returned by git ls-files"
    )]
    project: bool,

    #[arg(short, long, help = "Generate a default licensure config file")]
    generate_config: bool,
}

fn main() {
    let matches = Cli::parse();

    match matches.verbose {
        0 => (),
        x => simplelog::SimpleLogger::init(
            if x >= 3 {
                simplelog::LevelFilter::Trace
            } else if x >= 2 {
                simplelog::LevelFilter::Debug
            } else {
                simplelog::LevelFilter::Info
            },
            simplelog::ConfigBuilder::new()
                .set_time_level(simplelog::LevelFilter::Debug)
                .set_thread_level(simplelog::LevelFilter::Debug)
                .set_target_level(simplelog::LevelFilter::Debug)
                .set_location_level(simplelog::LevelFilter::Trace)
                .build(),
        )
        .unwrap(),
    };

    if matches.generate_config {
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

    let files: Vec<String> = if matches.project {
        get_project_files()
    } else if matches.files.len() > 0 {
        matches.files
    } else {
        eprintln!("ERROR: Must provide files to license either as arguments or via --project");
        process::exit(10);
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

    if let Some(exclude) = matches.exclude {
        config.add_exclude(&exclude);
    }

    if matches.in_place {
        config.change_in_place = true;
    }

    let licensure = Licensure::new(config).with_check_mode(matches.check);
    match licensure.license_files(&files) {
        Err(e) => {
            println!("Failed to license files: {}", e);
            process::exit(1);
        }
        Ok(stats) => {
            if matches.check
                && !(stats.files_not_licensed.is_empty()
                    && stats.files_needing_license_update.is_empty())
            {
                print_files(
                    &stats.files_needing_license_update,
                    "The following files' licenses need to be updated",
                );

                print_files(
                    &stats.files_not_licensed,
                    "The following files were not licensed with the given config.",
                );

                print_files(
                    &stats.files_needing_commenter,
                    "The following files did not have a commenter with the given config.",
                );

                process::exit(1);
            }

            if print_files(
                &stats.files_needing_commenter,
                "The following files did not have a commenter with the given config.",
            ) {
                process::exit(1);
            };
        }
    }
}

/// Print the given list of files (if non-empty) with a message "The following X
/// Y" where X is the number of files to be printed and Y is the given message
/// parameter. Returns true if files were printed and false otherwise.
fn print_files(files: &Vec<String>, message: &str) -> bool {
    if !files.is_empty() {
        eprintln!("{} {} ", message, files.len());
        for file in files {
            eprintln!("{}", file);
        }
        true
    } else {
        false
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
