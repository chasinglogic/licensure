use serde::Deserialize;

use crate::comments::BlockComment;
use crate::comments::Comment;
use crate::comments::LineComment;

fn def_trailing_lines() -> usize {
    0
}

pub fn get_filetype(filename: &str) -> &str {
    let iter = filename.split('.');
    match iter.last() {
        Some(s) => s,
        None => "",
    }
}

#[derive(Clone, Deserialize)]
#[serde(tag = "type")]
pub enum Commenter {
    #[serde(alias = "block")]
    Block {
        start_block_char: String,
        end_block_char: String,
        per_line_char: Option<String>,
        #[serde(default = "def_trailing_lines")]
        trailing_lines: usize,
    },
    #[serde(alias = "line")]
    Line {
        comment_char: String,
        #[serde(default = "def_trailing_lines")]
        trailing_lines: usize,
    },
}

#[derive(Clone, Deserialize)]
#[serde(untagged)]
enum FileType {
    Single(String),
    List(Vec<String>),
}

impl FileType {
    fn matches(&self, ft: &str) -> bool {
        match self {
            FileType::Single(ext) => ext == "any" || ext == ft,
            FileType::List(ref extensions) => extensions.iter().any(|ext| ext == ft),
        }
    }
}

#[derive(Clone, Deserialize)]
pub struct Config {
    #[serde(alias = "extensions")]
    extension: FileType,
    columns: Option<usize>,
    commenter: Commenter,
}

impl Config {
    pub fn default() -> Config {
        Config {
            extension: FileType::Single("any".to_string()),
            columns: None,
            commenter: Commenter::Line {
                comment_char: "#".to_string(),
                trailing_lines: 0,
            },
        }
    }

    pub fn matches(&self, file_type: &str) -> bool {
        self.extension.matches(file_type)
    }

    pub fn commenter(&self) -> Box<dyn Comment> {
        match &self.commenter {
            Commenter::Line {
                comment_char,
                trailing_lines,
            } => Box::new(
                LineComment::new(comment_char.as_str()).set_trailing_lines(*trailing_lines),
            ),
            Commenter::Block {
                start_block_char,
                end_block_char,
                per_line_char,
                trailing_lines,
            } => {
                let mut bc = BlockComment::new(start_block_char.as_str(), end_block_char.as_str())
                    .set_trailing_lines(*trailing_lines);

                if let Some(ch) = per_line_char {
                    bc = bc.with_per_line(ch.as_str());
                }

                Box::new(bc)
            }
        }
    }

    pub fn get_columns(&self) -> Option<usize> {
        self.columns
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_get_filetype() {
        assert_eq!("py", get_filetype("test.py"))
    }

}
