# licensure
A software license management CLI

## Table of Contents

- [Installation](#installation)
- [Usage](#usage)
  - [Example](#example)
  - [Supported Filetypes](#supported-filetypes)
- [Contributing](#contributing)
- [License](#license)

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

Make sure that you have a working Rust environment. Instructions for setting
one up can be found here: [https://rustup.rs](https://rustup.rs) you can then
install licensure from source with the following commands:

```bash
git clone https://github.com/chasinglogic/licensure
cd licensure
cargo install --path .
```

If you need to update cargo you can then run the appropriate `cargo install`
command with the `--force` flag.

## Usage

### Example

For now licensure only has one sub command: `license`. This command adds license
headers to source files and comments them. For example given the file
`test.py` which has the following contents:

```python
print("Hello world!")
```

If we run the following command:

```
licensure license --author 'Mathew Robinson' --email 'chasinglogic@gmail.com' --ident GPL-3.0 test.py
```

The resulting file will be:

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

Licensure does some naive string comparison to determine if the header already
exists. If you try to run licensure against a file which already has the
generated license header you'll get this message:

```
chasinglogic@Mathews-MBP:~ $ licensure license --author 'Mathew Robinson' --email 'chasinglogic@gmail.com' --ident GPL-3.0 test.py
Licensing file: test.py
test.py already licensed
chasinglogic@Mathews-MBP:~ $
```

This makes it safe and convenient to run `licensure license --project` on the
same project multiple times.

### Supported Filetypes

Licensure does it's best to determine the comment character based on filetype
but will always fall back to "python style" comments if it can't figure it out
since that is the most comment comment style. As of today we explicitly support
the following file types:

- js
- rs
- go
- c
- cpp
- html
- css
- py
- sh
- bash

Adding new file types is trivial and would make for a [great first pull
request](#contributing) if you're interested in adding your favorite programming
language.

### Help

For reference the full help output has been placed below. If you're having any
problems running licensure please open an
[issue](https://github.com/chasinglogic/licensure/issues).

```text
licensure 0.1.0
Mathew Robinson <chasinglogic@gmail.com>

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
along with this program.  If not, see <http://www.gnu.org/licenses/>.

USAGE:
    licensure [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    help       Prints this message or the help of the given subcommand(s)
    license    Apply license headers to source files
```

License help: 

```text
licensure-license
Apply license headers to source files

USAGE:
    licensure license [FLAGS] [FILES]...

FLAGS:
    -a, --author     Full name of copyright owner / source code author.
    -e, --email      Email of the copyright owner / source code author.
    -h, --help       Prints help information
    -i, --ident      SPDX license identifier to license files with.
    -p, --project    When specified will license the current project files as returned by git ls-files
    -V, --version    Prints version information

ARGS:
    <FILES>...    Files to license, ignored if --project is supplied
```

## Contributing

1. Fork it!
2. Create your feature branch: `git checkout -b my-new-feature`
3. Commit your changes: `git commit -am 'Add some feature'`
4. Push to the branch: `git push origin my-new-feature`
5. :fire: Submit a pull request :D :fire:

## License

This code is distributed under the GNU General Public License

```
    Copyright (C) 2017 Mathew Robinson

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
