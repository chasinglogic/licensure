use textwrap;

use super::Comment;

pub struct LineComment {
    character: String,
    trailing_lines: usize,
}

impl LineComment {
    pub fn new(character: &str) -> LineComment {
        LineComment {
            character: String::from(character),
            trailing_lines: 0,
        }
    }

    pub fn set_trailing_lines(mut self, num_lines: usize) -> LineComment {
        self.trailing_lines = num_lines;
        self
    }

    pub fn skip_trailing_lines(mut self) -> LineComment {
        self.trailing_lines = 0;
        self
    }
}

impl Comment for LineComment {
    fn comment(&self, text: &str, columns: Option<usize>) -> String {
        let local_copy = if let Some(cols) = columns {
            // Subtract two columns to account for the comment
            // character and space we will add later.
            textwrap::fill(text, if cols > 2 { cols - 2 } else { cols })
        } else {
            text.to_string()
        };

        let mut lines: Vec<&str> = local_copy.split('\n').collect();
        // split always adds an empty element to the end of the vector
        // so we filter it out here.
        if !lines.is_empty() && lines.last().unwrap() == &"" {
            lines.pop();
        }

        let mut new_text = "".to_string();
        for line in lines {
            let new_line = match line {
                "" => format!("{}\n", self.character),
                _ => format!("{} {}\n", self.character, line),
            };

            new_text.push_str(&new_line);
        }

        for _ in 0..self.trailing_lines {
            new_text.push('\n');
        }

        new_text
    }
}
