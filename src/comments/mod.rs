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

pub use block_comment::BlockComment;
pub use line_comment::LineComment;

mod block_comment;
mod line_comment;

pub trait Comment {
    fn comment(&self, text: &str) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    const EX_TEXT: &'static str = "There once was a man
with a very nice cat
the cat wore a top hat
it looked super dapper
";

    #[test]
    fn test_comment_python() {
        assert_eq!(
            "# There once was a man
# with a very nice cat
# the cat wore a top hat
# it looked super dapper
",
            LineComment::new("#", None).comment(EX_TEXT)
        )
    }

    #[test]
    fn test_comment_python_w_trailing_lines() {
        assert_eq!(
            "# There once was a man
# with a very nice cat
# the cat wore a top hat
# it looked super dapper


",
            LineComment::new("#", None)
                .set_trailing_lines(2)
                .comment(EX_TEXT)
        )
    }

    #[test]
    fn test_comment_cpp() {
        assert_eq!(
            "/*
* There once was a man
* with a very nice cat
* the cat wore a top hat
* it looked super dapper
*/",
            BlockComment::new("/*\n", "*/", None)
                .with_per_line("*")
                .comment(EX_TEXT)
        )
    }

    #[test]
    fn test_comment_cpp_w_trailing_lines() {
        assert_eq!(
            "/*
* There once was a man
* with a very nice cat
* the cat wore a top hat
* it looked super dapper
*/

",
            BlockComment::new("/*\n", "*/", None)
                .with_per_line("*")
                .set_trailing_lines(2)
                .comment(EX_TEXT)
        )
    }

    #[test]
    fn test_comment_html() {
        assert_eq!(
            "<!--
There once was a man
with a very nice cat
the cat wore a top hat
it looked super dapper
-->",
            BlockComment::new("<!--\n", "-->", None).comment(EX_TEXT)
        )
    }
}
