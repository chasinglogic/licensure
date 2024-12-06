# licensure

A software license header and copyright mangement tool.

## Table of Contents

- [Why use Licensure?](#why-use-licensure)
- [Installation](#installation)
- [Usage](#usage)
  - [Example](#example)
  - [Supported Filetypes](#supported-filetypes)
- [Configuration](#configuration)
- [Contributing](#contributing)
- [License](#license)

## Why use Licensure?

According to the Apache Software Foundation: [License headers allow
someone examining the file to know the terms for the work, even when
it is distributed without the rest of the distribution. Without a
licensing notice, it must be assumed that the author has reserved all
rights, including the right to copy, modify, and
redistribute.](http://www.apache.org/legal/src-headers.html#faq-whyheader)

It's an easy extra step to make sure that no matter what file someone
is looking at in your software they are aware of their rights and
yours. However maintaining and updating a copyright notice in every
source file can be a cumbersome exercise in patience. This is where
Licensure comes in.

Licensure makes it easy to maintain and update your copyright notices
across a project.

## Installation

### Install from Release (WIP)

1. Navigate to [the Releases Page](https://github.com/chasinglogic/licensure/releases)
2. Find the tar ball for your platform / architecture. For example, on 64 bit
   Mac OSX, the archive is named `licensure_{version}_darwin_amd64.tar.gz`
3. Extract the tar ball
4. Put the licensure binary in your `$PATH`

### Install from Source

Licensure is available on crates.io and so can be installed with the following
command:

```
cargo install licensure
```

Make sure that you have a working Rust environment. Instructions for
setting one up can be found [on the rustup
website](https://rustup.rs). You can then install licensure from
source with the following commands:

```bash
git clone https://github.com/chasinglogic/licensure
cd licensure
cargo install --path .
```

If you need to update licensure via cargo you can then run the appropriate
`cargo install` command with the `--force` flag.

## Usage

Licensure searches for a config file `.licensure.yml` which will
inform it's operation. Licensure will not function without a config
file and if it cannot find one it will report this to you. 

The method for finding a config file [can be found in the
Configuration section of this
document.](#where-the-configuration-file-lives) For our purposes we
can generate a default config file with: `licensure
--generate-config`. This will write an example configuration file with
documentation comments into your current working directory. Note that
the default config file this generates is not functional immediately
as you must fill out the `licenses` section of the config. Licensure
does not want to make assumptions about the licensing of your project.

In the following examples we will assume your using a simple
configuration file such as the following:

```yaml
licenses:
  - files: any
    ident: GPL-3.0
    authors:
      - name: Mathew Robinson
        email: chasinglogic@gmail.com
    auto_template: true
    
comments:
  - columns: 80
    extension: any
    commenter:
      type: line
      comment_char: "#"
      trailing_lines: 0
```

With this config file if you create a source file named `test.py` with the contents:

```python
print("Hello world!")
```

We can then license this file with the command:

```
chasinglogic@galactica $ licensure test.py
```

Licensure will print out the licensed file as follows:

```python
# Copyright 2018 Mathew Robinson <chasinglogic@gmail.com>. All rights reserved.
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <http://www.gnu.org/licenses/>.
print("Hello World!")
```

If we want to update the contents `test.py` instead of just printing
out the licensed file content we can give Licensure the `--in-place`
(or shortened form `-i`) flag:

```
chasinglogic@galactica $ licensure --in-place test.py
```

Licensure does some naive string comparison to determine if the header already
exists. If you try to run licensure against a file which already has the
generated license header it will skip it. You can see what files were
skipped by running licensure with `--verbose`:

```
chasinglogic@galactica $ licensure --in-place --verbose test.py
test.py already licensed
chasinglogic@galactica $
```

This makes it safe and convenient to run `licensure --in-place
--project` on the same project multiple times.

## Configuration

Licensure requires the use of a configuration file. This section will
explain all of the options of a configuration file, their definitions,
and some explanation of how they work. If you're instead looking for
the quickest way to get up and running you can generate a config with
`licensure --generate-config` which will give you a default config
file with documentation comments.

### Where the Configuration File lives

The configuration file is written in yaml and is searched for by
climbing the directory tree, starting at the current working
directory, for a file named `.licensure.yml`. If it is not found the
global configuration file located at
`$XDG_CONFIG_HOME/licensure/config.yml` (where `$XDG_CONFIG_HOME` is
`$HOME/.config` by default) will be used.

This essentially means that subdirectories can have their own
licensure configs and the order of precedence is closest config file
to the current working directory.

### Top Level Configuration Options

The Configuration File only has a two top level options: `exclude`,
and `change_in_place`. The other top level keys in the config file are
referred to as [Configuration Sections](#configuration-sections) and
make up the bulk of Licensure configuration.

#### change\_in\_place. 

Takes a boolean indicating whether to change files in place when
licensure is run. If this is set to `false` then when this config file
is in use files will never be updated in place, even if `--in-place`
is provided via the command line.

**Example Configuration:**

```yaml
change_in_place: true
```

#### exclude

Takes a list of strings that will be compiled as regexes to filter out
files from licensing.  Excludes passed via the command line flag will
be joined with any found in a config file.

**Example Configuration:**

```yaml
# Regexes which if matched by a file path will always be excluded from
# getting a license header
excludes:
  - \.gitignore
  - .*lock
  - \.git/.*
  - \.licensure\.yml
  - README.*
  - LICENSE.*
  - .*\.(md|rst|txt)
```

### Configuration Sections

Currently Licensure has two configuration sections: `licenses` and
`comments`. They configure the two axes Licensure uses to generate a
license header.

#### licenses

The licenses section is a list of license configuration
objects. License configuration objects define what licenses should
apply to what files, the templates for those licenses, and the
copyright holders of those files. A license configuration object has
the following form:

```yaml
# Either a regex or the string "any" to determine to what files this
# license should apply. It is common for projects to have files
# under multiple licenses or with multiple copyright holders. This
# provides the ability to automatically license files correctly
# based on their file paths.
#
# If "any" is provided all files will match this license.
files: any

# The license identifier, a list of common identifiers can be
# found at: https://spdx.org/licenses/ but existence of the ident
# in this list it is not enforced unless auto_template is set to
# true.
ident: MIT

# A list of authors who hold copyright over these files
authors:
  # Provide either your full name or company name for copyright purposes
  - name: Your Name Here
    # Optionally provide email for copyright purposes
    # email: you@yourdomain.com

# The template that will be rendered to generate the header before
# comment characters are applied. Available variables are:
#  - [year]: substituted with the current year.
#  - [name of author]: Substituted with name of the author and email
#    if provided. If email is provided the output appears as Full
#    Name <email@example.com>. If multiple authors are provided the
#    list is concatenated together with commas.
template: |
  Copyright [year] [name of author]. All rights reserved. Use of
  this source code is governed by the [ident] license that can be
  found in the LICENSE file.

# If auto_template is true then the template configuration is ignored
# and the SPDX API will be queried with the ident value to
# automatically determine the license header template. auto_template
# works best with licenses that have a standardLicenseHeader field
# defined in their license info JSON, if it is not then we will use
# the full licenseText to generate the header which works fine for
# short licenses like MIT but can be quite lengthy for other licenses
# like BSD-4-Clause. The above default template is valid for most
# licenses and is recommended for MIT, and BSD licenses. Common
# licenses that work well with the auto_template feature are GPL
# variants, and the Apache 2.0 license.
#
# Important Note: this means the ident must be a valid SPDX identifier
# auto_template: true
```

A common licenses section would look like:

```yaml
licenses:
  - files: any
    authors:
      - name: Mathew Robinson
        email: chasinglogic@gmail.com
        ident: MIT
    template: |
      Copyright [year] [name of author]. All rights reserved. Use of
      this source code is governed by the [ident] license that can be
      found in the LICENSE file.
```

##### Year ranges

You can specify a year range for your copyright instead by using the start_year
configuration option. If provided Licensure will put a year range in your
copyright. An example with a `start_year` value of `2019` would look like so:

```
Copyright 2019, 2024 ...
```

The `end_year` can also be specified but if omitted will automatically be
updated to be the current year per your local system time. 

###### Automated year ranges

If you want per-file year ranges or just automated ones you can opt in with the
`use_dynamic_year_ranges: true` setting. This will use `git` commit information
to determine a files created and last updated year. It will then license that
file with a year range specific to it based on the `git` information.

#### comments

The comments section is a list of comment configuration
objects. Comment configuration objects define how files with certain
extensions will be commented. They can also define the column width to
wrap the generated license header at. A comment configuration object
has the following fields: `extensions` (or `extension`), `columns`,
`commenter`,

##### Columns Configuration

The `columns` key specifies to what width the license header should be wrapped. Common values include: `80`, `100`, `120`.

Example:

```yaml
columns: 80
```

##### Extension Configuration

The extensions (or singular extension) field defines which file
extensions to apply the commenter to. If extension is the string 
"any" then all extensions will match this comment configuration.

Example use of any:

```yaml
extension: any
```

Example definition of explicit extensions

```yaml
extensions:
  - js
  - rs
  - go
```

##### Files Configuration

An optional list of regular expressions which, if specified, restrict what files
this commenter applies to. This commenter will only apply to files which match
one of the given regular expressions.

##### Commenter Configuration

The commenter field defines the kind of commenter to
generate. There are two types of commenters: line and block.

A line commenter type will apply the `comment_char` to the beginning
of each line in the license header. It will then add empty newlines to
the end of the header equal to `trailing_lines`.

A block commenter type will add `start_block_char` as the first character
in the license header and add `end_block_char` as the last character
in the license header. When `per_line_char` is provided each line of
the header between the block start and end characters will be
line commented with the `per_line_char`.

If trailing_lines is omitted it's assumed to be 0.

###### Line Commenter Example

This is an example of a line commenter configuration. 

```yaml
commenter:
  type: line
  comment_char: "//"
  trailing_lines: 0
```

If this commenter is given the text:

```
A piece of text that
spans multiple lines
```

It would generate:

```
// A piece of text that 
// spans multiple lines
```

Note: when columns has a value the text may be re-wrapped to match the
column width.

###### Block Commenter Example

This is an example of a block commenter configuration. 

```yaml
commenter:
  type: block
  start_block_char: "/*\n"
  end_block_char: "*/"
  per_line_char: "*"
  trailing_lines: 0
```

If this commenter is given the text:

```
A piece of text that
spans multiple lines
```

It would generate:

```
/*
* A piece of text that 
* spans multiple lines
*/
```

Note: when columns has a value the text may be re-wrapped to match the
column width.

### A Complete Configuration Example 

The best up to date minimal example configuration is the one for
[licensure itself](https://github.com/chasinglogic/licensure/blob/master/.licensure.yml).

## Contributing

1. Fork it!
2. Create your feature branch: `git checkout -b my-new-feature`
3. Commit your changes: `git commit -am 'Add some feature'`
4. Push to the branch: `git push origin my-new-feature`
5. :fire: Submit a pull request :D :fire:

## License

This code is distributed under the GNU General Public License

```
    Copyright (C) 2024 Mathew Robinson

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
```
