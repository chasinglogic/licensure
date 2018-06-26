pub trait Comment {
    fn comment(&self, text: &str) -> String;
}

pub struct BlockComment {
    start: String,
    end: String,
    per_line: Option<String>,
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
        self.per_line = Some(String::from(per_line));
        self
    }
}

impl Comment for BlockComment {
    fn comment(&self, text: &str) -> String {
        let mut new_text = self.start.clone();

        match self.per_line {
            Some(ref comment_char) => {
                let lc = LineComment::new(comment_char).comment_empty_lines();
                let commented_text = lc.comment(text);
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
    skip_empty_lines: bool,
}

impl LineComment {
    fn new(character: &str) -> LineComment {
        LineComment {
            character: String::from(character),
            skip_empty_lines: true,
        }
    }

    fn comment_empty_lines(mut self) -> LineComment {
        self.skip_empty_lines = false;
        self
    }
}

impl Comment for LineComment {
    fn comment(&self, text: &str) -> String {
        let local_copy = text.to_string().clone();
        let lines = local_copy.split("\n");
        let mut new_text = "".to_string();
        for line in lines {
            let new_line;

            match line {
                "" if self.skip_empty_lines => continue,
                "" => new_line = format!("{}\n", self.character),
                _ => new_line = format!("{} {}\n", self.character, line),
            };

            new_text.push_str(&new_line);
        }

        new_text
    }
}

pub fn get_commenter(ftype: &str) -> Box<Comment> {
    match ftype {
        "rs" => Box::new(LineComment::new("//")),
        "js" => Box::new(LineComment::new("//")),
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
