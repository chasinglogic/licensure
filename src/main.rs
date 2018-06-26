extern crate chrono;
extern crate clap;
extern crate licensure;

use chrono::prelude::*;
use clap::{App, Arg, SubCommand};

struct Config {
    author: String,
    email: String,
    year: String,
    ident: String,
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
            let files: Vec<String>;

            if args.is_present("project") {
                files = licensure::utils::get_project_files();
            } else {
                files = args
                    .values_of("FILES")
                    .expect("ERROR: Must provide files to license either as args or via --project")
                    .map(str::to_string)
                    .collect();
            }

            let now;
            let year_string;
            let config = Config {
                author: args
                    .value_of("author")
                    .expect("--author is a required flag"),
                email: args.value_of("email").expect("--email is a required flag"),
                ident: args.value_of("ident").expect("--ident is a required flag"),
                year: if let Some(year) = args.value_of("year") {
                    year
                } else {
                    now = Local::now();
                    year_string = now.year().to_string();
                    &year_string
                },
            };

            for file in files {
                println!("Licensing file: {}", file);
                licensure::ops::license_file(
                    &file,
                    &config.ident,
                    &config.author,
                    &config.email,
                    &config.year,
                );
            }
        }
        _ => println!("ERROR: Unknown command"),
    }
}
