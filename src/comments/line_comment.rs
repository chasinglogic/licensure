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
use super::Comment;

pub struct LineComment {
    character: String,
    trailing_lines: usize,
    cols: Option<usize>,
}

impl LineComment {
    pub fn new(character: &str, cols: Option<usize>) -> LineComment {
        LineComment {
            character: String::from(character),
            trailing_lines: 0,
            cols,
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
    fn comment(&self, text: &str) -> String {
        let local_copy = match self.cols {
            Some(cols) => {
                // Subtract the comment width to account for the comment
                // character and space we will add later.
                textwrap::fill(
                    text,
                    if cols > self.comment_width() {
                        cols - self.comment_width()
                    } else {
                        cols
                    },
                )
            }
            None => text.to_string(),
        };

        let lines = local_copy.lines();
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

    fn comment_width(&self) -> usize {
        self.character.len() + 1
    }
}
