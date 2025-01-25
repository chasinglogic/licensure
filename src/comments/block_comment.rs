use crate::comments::line_comment::LineComment;

use super::Comment;

pub struct BlockComment {
    start: String,
    end: String,
    per_line: Option<Box<dyn Comment>>,
    trailing_lines: usize,
    cols: Option<usize>,
}

impl BlockComment {
    pub fn new(start: &str, end: &str, cols: Option<usize>) -> BlockComment {
        BlockComment {
            start: String::from(start),
            end: String::from(end),
            per_line: None,
            trailing_lines: 0,
            cols,
        }
    }

    pub fn set_trailing_lines(mut self, num_lines: usize) -> BlockComment {
        self.trailing_lines = num_lines;
        self
    }

    pub fn with_per_line(mut self, per_line: &str) -> BlockComment {
        self.per_line = Some(Box::new(
            LineComment::new(per_line, self.cols).skip_trailing_lines(),
        ));
        self
    }
}

impl Comment for BlockComment {
    fn comment(&self, text: &str) -> String {
        let mut new_text = self.start.clone();
        let wrapped_text;

        match self.per_line {
            Some(ref commenter) => {
                let commented_text = commenter.comment(text);
                new_text.push_str(&commented_text);
            }
            None => new_text.push_str(match self.cols {
                Some(cols) => {
                    wrapped_text = textwrap::fill(text, cols);
                    wrapped_text.as_str()
                }
                None => text,
            }),
        };

        new_text.push_str(&self.end);

        for _ in 0..self.trailing_lines {
            new_text.push('\n');
        }

        new_text
    }

    fn comment_width(&self) -> usize {
        if let Some(ref character) = self.per_line {
            character.comment_width()
        } else {
            0
        }
    }
}
