extern crate clap;
extern crate licensure;

use clap::{App, Arg};

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
        .setting(AppSettings::TrailingVarArg)
        .arg(Arg::with_name("project").long("project").short("p").help(
            "When specified will license the current project files
        as returned by git ls-files",
        ))
        .arg(Arg::with_name("short").short("s").long("short"))
        .arg(
            Arg::with_name("license")
                .short("l")
                .long("license")
                .help("SPDX license identifier to license files with."),
        )
        .arg(
            Arg::with_name("FILES")
                .multiple(true)
                .help("Files to license, ignored if --project is supplied"),
        )
        .get_matches();
}
