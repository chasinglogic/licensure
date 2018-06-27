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


pub trait Comment {
    fn comment(&self, text: &str) -> String;
}

pub struct BlockComment {
    start: String,
    end: String,
    per_line: Option<Box<Comment>>,
}

impl BlockComment {
    fn new(start: &str, end: &str) -> BlockComment {
        BlockComment {
            start: String::from(start),
            end: String::from(end),
            per_line: None,
        }
    }

    fn with_per_line(mut self, per_line: &str) -> BlockComment {
        self.per_line = Some(Box::new(LineComment::new(per_line).skip_trailing_lines()));
        self
    }
}

impl Comment for BlockComment {
    fn comment(&self, text: &str) -> String {
        let mut new_text = self.start.clone();

        match self.per_line {
            Some(ref commenter) => {
                let commented_text = commenter.comment(text);
                new_text.push_str(&commented_text);
            }
            None => new_text.push_str(text),
        };

        new_text.push_str(&self.end);
        new_text
    }
}

pub struct LineComment {
    character: String,
    no_trailing_lines: bool,
}

impl LineComment {
    fn new(character: &str) -> LineComment {
        LineComment {
            character: String::from(character),
            no_trailing_lines: false,
        }
    }

    fn skip_trailing_lines(mut self) -> LineComment {
        self.no_trailing_lines = true;
        self
    }
}

impl Comment for LineComment {
    fn comment(&self, text: &str) -> String {
        let local_copy = text.to_string().clone();
        let lines = local_copy.split("\n");
        let mut new_text = "".to_string();
        for line in lines {
            new_text.push_str(&match line {
                "" => format!("{}\n", self.character),
                _ => format!("{} {}\n", self.character, line),
            });
        }

        if !self.no_trailing_lines {
            new_text.push_str("\n\n");
        }

        new_text
    }
}

pub fn get_commenter(ftype: &str) -> Box<Comment> {
    match ftype {
        "rs" => Box::new(LineComment::new("//")),
        "js" => Box::new(LineComment::new("//")),
        "go" => Box::new(LineComment::new("//")),
        "html" => Box::new(BlockComment::new("<!--\n", "-->")),
        "cpp" => Box::new(BlockComment::new("/*\n", "*/").with_per_line("*")),
        "c" => Box::new(BlockComment::new("/*\n", "*/").with_per_line("*")),
        _ => Box::new(LineComment::new("#")),
    }
}

// get_filetype returns the filetype as expected by get_commenter
pub fn get_filetype(filename: &str) -> String {
    let iter = filename.split(".");
    match iter.last() {
        Some(s) => s.to_string(),
        None => "".to_string(),
    }
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
#


",
            get_commenter("py").comment(EX_TEXT)
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
*
*/",
            get_commenter("cpp").comment(EX_TEXT)
        )
    }

    #[test]
    fn test_get_filetype() {
        assert_eq!("py", get_filetype("test.py"))
    }
}
