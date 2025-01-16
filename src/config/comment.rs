use std::path::Path;

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
use serde::Deserialize;

use crate::comments::BlockComment;
use crate::comments::Comment;
use crate::comments::LineComment;

use super::RegexList;

fn def_trailing_lines() -> usize {
    0
}

pub fn get_filetype(filename: &str) -> &str {
    // Get just the filename component of the given filename (which is really a path)
    let path_filename = Path::new(filename)
        .file_name()
        .unwrap_or_default()
        // We should always be able to go to_str here because we created the os_str from a &str
        .to_str()
        .unwrap();

    // If there's no "." in the filename, return no extension
    if !path_filename.contains('.') {
        return "";
    }

    let iter = path_filename.split('.');
    iter.last().unwrap_or_default()
}

#[derive(Clone, Deserialize, Debug)]
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

#[derive(Clone, Deserialize, Debug)]
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

#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    #[serde(alias = "extensions")]
    extension: FileType,
    #[serde(default)]
    files: Option<RegexList>,
    columns: Option<usize>,
    commenter: Commenter,
}

impl Config {
    pub fn matches(&self, file_type: &str, filename: &str) -> bool {
        if self.extension.matches(file_type) {
            if let Some(files) = &self.files {
                files.is_match(filename)
            } else {
                true
            }
        } else {
            false
        }
    }

    pub fn commenter(&self) -> Box<dyn Comment> {
        match &self.commenter {
            Commenter::Line {
                comment_char,
                trailing_lines,
            } => Box::new(
                LineComment::new(comment_char.as_str(), self.get_columns())
                    .set_trailing_lines(*trailing_lines),
            ),
            Commenter::Block {
                start_block_char,
                end_block_char,
                per_line_char,
                trailing_lines,
            } => {
                let mut bc = BlockComment::new(
                    start_block_char.as_str(),
                    end_block_char.as_str(),
                    self.get_columns(),
                )
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
        assert_eq!("py", get_filetype("test.py"));
        assert_eq!("htaccess", get_filetype(".htaccess"));
        assert_eq!("htaccess", get_filetype("/foo/bar/.htaccess"));
        assert_eq!("htaccess", get_filetype("./foo/.bar/.htaccess"));
        assert_eq!("htaccess", get_filetype("./.htaccess"));
        assert_eq!("html", get_filetype("index.html"));
        assert_eq!("html", get_filetype("/foo/bar/index.html"));
        assert_eq!("html", get_filetype("./foo/.bar/index.html"));
        assert_eq!("html", get_filetype("./index.html"));
        assert_eq!("", get_filetype("NONE"));
        assert_eq!("", get_filetype("/foo/bar/NONE"));
        assert_eq!("", get_filetype("./foo/.bar/NONE"));
        assert_eq!("", get_filetype("./NONE"));
    }

    static COMMENT_CONFIG_PY: &str = r##"columns: 80
extensions:
    - py
commenter:
    type: line
    comment_char: "#""##;

    static COMMENT_CONFIG_PY_EXAMPLE: &str = r##"columns: 80
extensions:
    - py
files:
    - example/.*
commenter:
    type: line
    comment_char: "#""##;
    #[test]
    fn test_matches() {
        let config_py: Config =
            serde_yaml::from_str(COMMENT_CONFIG_PY).expect("Parsing static config");
        let config_py_example: Config =
            serde_yaml::from_str(COMMENT_CONFIG_PY_EXAMPLE).expect("Parsing static config");

        let file = "example/foo.py";
        assert!(config_py.matches(get_filetype(file), file));
        assert!(config_py_example.matches(get_filetype(file), file));

        let file = "example/foo.c";
        assert!(!config_py.matches(get_filetype(file), file));
        assert!(!config_py_example.matches(get_filetype(file), file));

        let file = "another_dir/foo.py";
        assert!(config_py.matches(get_filetype(file), file));
        assert!(!config_py_example.matches(get_filetype(file), file));
    }
}
