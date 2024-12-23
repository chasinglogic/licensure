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
use regex::Regex;

pub fn remove_column_wrapping(string: &str) -> String {
    // Some license headers come pre-wrapped to a column width.
    // This regex replacement undoes the column-width wrapping
    // while preserving intentional line breaks / empty lines.
    let re = Regex::new(r"(?P<char>.)\n").unwrap();
    re.replace_all(string, "$char ").replace(" \n", "\n\n")
}

#[cfg(test)]
mod tests {
    use crate::utils::remove_column_wrapping;

    #[test]
    fn test_remove_column_wrapping() {
        let content = "\
some wrapped
text to unwrap.

The line above
is an intentional
line break.

So is this.";

        let expected = "some wrapped text to unwrap.\n\nThe line above \
        is an intentional line break.\n\nSo is this.";
        assert_eq!(expected, remove_column_wrapping(content))
    }
}
