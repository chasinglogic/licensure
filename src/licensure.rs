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
use std::fmt;
use std::fs::File;
use std::io::{self, prelude::*};
use std::sync::LazyLock;

use regex::Regex;

use crate::comments::Comment;
use crate::config::Config;
use crate::template::Template;

static SHEBANG_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^#!.*\n").expect("shebang regex didn't compile!"));

enum Cause {
    IO(io::Error),
}

pub struct Error {
    context: String,
    cause: Cause,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: ", self.context)?;

        match &self.cause {
            Cause::IO(err) => err.fmt(f),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
enum Action {
    NeedsUpdate(String),
    AlreadyLicensed,
    NoConfigMatched,
    NoCommenterMatched,
}

pub struct Licensure {
    config: Config,
    stats: LicenseStats,
    check_mode: bool,
}

impl Licensure {
    pub fn new(config: Config) -> Licensure {
        Licensure {
            config,
            check_mode: false,
            stats: LicenseStats::new(),
        }
    }

    pub fn with_check_mode(mut self, check_mode: bool) -> Licensure {
        self.check_mode = check_mode;
        self
    }

    pub fn license_files(mut self, files: &[String]) -> Result<LicenseStats, Error> {
        self.stats = LicenseStats::new();

        for file in files {
            if self.config.excludes.is_match(file) {
                info!("skipping {} because it is excluded.", file);
                continue;
            }

            debug!("working on file: {}", &file);

            let mut content = String::new();
            {
                let mut f = File::open(file).map_err(|e| Error {
                    context: format!("failed to open file {}", file),
                    cause: Cause::IO(e),
                })?;
                f.read_to_string(&mut content).map_err(|e| Error {
                    context: format!("failed to read file {}", file),
                    cause: Cause::IO(e),
                })?;
            }

            match self.determine_required_action(file, &mut content) {
                Action::NeedsUpdate(update) => self.handle_update(file, &update)?,
                Action::NoConfigMatched => self.stats.files_not_licensed.push(file.clone()),
                Action::NoCommenterMatched => {
                    self.stats.files_not_licensed.push(file.clone());
                    self.stats.files_needing_commenter.push(file.clone())
                }
                Action::AlreadyLicensed => continue,
            }
        }

        Ok(self.stats)
    }

    fn handle_update(&self, file: &String, content: &str) -> Result<(), Error> {
        if self.check_mode {
            return Result::Ok(());
        }

        if self.config.change_in_place {
            let mut f = File::create(file).map_err(|e| Error {
                context: format!("failed to create file {}", file),
                cause: Cause::IO(e),
            })?;
            return f.write_all(content.as_bytes()).map_err(|e| Error {
                context: format!("failed to write to file {}", file),
                cause: Cause::IO(e),
            });
        }

        println!("{}", content);
        Result::Ok(())
    }

    /// Strip the shebang from content and return the stripped string so it can later be added back
    /// to the content.
    fn strip_shebang_if_found(content: &mut String) -> Option<String> {
        // Can't use Option::map because of double borrow of content.
        #[allow(clippy::manual_map)]
        match SHEBANG_REGEX.find(content) {
            // If we idenfied a shebang, strip it from content (we'll add it back at the end)
            Some(shebang_match) => Some(content.drain(..shebang_match.end()).collect()),
            None => None,
        }
    }

    fn get_outdated_replacement(
        &self,
        templ: &Template,
        commenter: &dyn Comment,
        content: &str,
        header: &str,
    ) -> Option<String> {
        let comment_width = commenter.comment_width();
        let normalised = content
            .to_string()
            .clone()
            .lines()
            .map(|line| {
                line.chars()
                    // Strip any comment characters off the front. We don't care what this does to the
                    // rest of the content since we're specifically trying to normalise the license
                    // header itself.
                    .skip(comment_width)
                    .collect::<String>()
                    // Trim so we can remove any erroneous blank lines that would cause double
                    // spaces when we join.
                    // .trim()
                    .to_string()
            })
            // Remove blank lines for more consistent normalisation (otherwise we get double spaces
            // in some spots).
            // .filter(|line| line.len() > 0)
            .collect::<Vec<String>>()
            .join("NEWLINE");

        let rgx = templ.build_year_varying_regex(commenter, false);
        if let Some(m) = rgx.find(&normalised) {
            let start = m.start();
            let end = m.end();
            let found_header = &normalised[start..end];
            let prefix = &normalised[..start];
            let nl = Regex::new("NEWLINE").expect("ahhh");

            let header_size = nl.split(found_header).count() + 1;
            let prefix_size = nl.split(prefix).count() - 1;

            let old_header = content
                .to_string()
                .split_inclusive('\n')
                .skip(prefix_size)
                .take(header_size)
                .collect::<Vec<_>>()
                .join("");

            Some(content.replace(&old_header, header).to_string())
        } else {
            None
        }
    }

    // fn get_outdated_replacement(
    //     &self,
    //     templ: &Template,
    //     commenter: &dyn Comment,
    //     content: &str,
    //     header: &str,
    // ) -> Option<String> {
    //     let outdated_re = templ.outdated_license_pattern(commenter);
    //     trace!("Content: {}", content);
    //     trace!("Outdated Regex: {:?}", outdated_re);
    //     if outdated_re.is_match(content) {
    //         trace!("outdated regex matched.");
    //         return Some(outdated_re.replace(content, header).to_string());
    //     }

    //     // Account for possible whitespace changes
    //     let trimmed_outdated_re = templ.outdated_license_trimmed_pattern(commenter);
    //     trace!("trimmed_outdated_re regex: {:?}", trimmed_outdated_re);
    //     if trimmed_outdated_re.is_match(content) {
    //         Some(trimmed_outdated_re.replace(content, header).to_string())
    //     } else {
    //         None
    //     }
    // }

    fn get_replaces_replacement(
        &self,
        replaces: &Vec<Regex>,
        content: &str,
        header: &str,
    ) -> Option<String> {
        for old in replaces {
            if old.is_match(content) {
                return Some(old.replace(content, header).to_string());
            }
            // TODO: Add a check here with comments stripped from content
        }
        None
    }

    fn add_header(&self, mut header: String, content: &mut String) -> String {
        if let Some(value) = Self::strip_shebang_if_found(content) {
            header.insert_str(0, &value);
        }

        header.push_str(content);
        header
    }

    fn determine_required_action(&mut self, file: &String, content: &mut String) -> Action {
        let templ = match self.config.licenses.get_template(file) {
            Some(t) => t,
            None => {
                info!("skipping {} because no license config matched.", file);
                return Action::NoConfigMatched;
            }
        };

        let commenter = match self.config.comments.get_commenter(file) {
            Some(c) => c,
            None => {
                return Action::NoCommenterMatched;
            }
        };

        let uncommented = templ.render();
        let header = commenter.comment(&uncommented);
        if content.contains(&header) || content.contains(header.trim_end()) {
            info!("{} already licensed", file);
            return Action::AlreadyLicensed;
        }

        if let Some(update) =
            self.get_outdated_replacement(&templ, commenter.as_ref(), content, &header)
        {
            info!("{} licensed, but year is outdated", file);
            self.stats.files_needing_license_update.push(file.clone());
            return Action::NeedsUpdate(update);
        }

        if let Some(replaces) = self.config.licenses.get_replaces(file) {
            if let Some(update) = self.get_replaces_replacement(replaces, content, &header) {
                info!("{} licensed, but license is outdated", file);
                self.stats.files_needing_license_update.push(file.clone());
                return Action::NeedsUpdate(update);
            }
        }

        info!("{} is not licensed", file);
        self.stats.files_needing_license_update.push(file.clone());
        Action::NeedsUpdate(self.add_header(header, content))
    }
}

pub struct LicenseStats {
    pub files_not_licensed: Vec<String>,
    pub files_needing_license_update: Vec<String>,
    pub files_needing_commenter: Vec<String>,
}

impl LicenseStats {
    fn new() -> Self {
        Self {
            files_not_licensed: Vec::new(),
            files_needing_license_update: Vec::new(),
            files_needing_commenter: Vec::new(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::Config;
    use crate::template::test_context_with_range;
    use crate::{
        comments::LineComment,
        template::{test_context, Template},
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn test_detects_outdated_year() {
        let l = Licensure::new(Config::default());
        let templ = Template::new("License [year]\n\ntext", test_context("2024"));
        let commenter = LineComment::new("#", None);
        let header = commenter.comment(&templ.render());
        let content = "# License 2020\n#\n# text";
        let result = l.get_outdated_replacement(&templ, &commenter, content, &header);
        assert!(result.is_some());
    }

    #[test]
    fn test_detects_outdated_year_range() {
        let l = Licensure::new(Config::default());
        let templ = Template::new(
            "License [year]\n\ntext",
            test_context_with_range("2020", "2024"),
        );
        let commenter = LineComment::new("#", None);
        let header = commenter.comment(&templ.render());
        let content = "# License 2020, 2023\n#\n# text";
        let result = l.get_outdated_replacement(&templ, &commenter, content, &header);
        assert!(result.is_some());
    }

    #[test]
    fn test_detects_outdated_year_range_when_previous_header_wasnt_a_range() {
        let l = Licensure::new(Config::default());
        let templ = Template::new(
            "License [year]\n\ntext",
            test_context_with_range("2020", "2024"),
        );
        let commenter = LineComment::new("#", None);
        let header = commenter.comment(&templ.render());
        let content = "# License 2020\n#\n# text";
        let result = l.get_outdated_replacement(&templ, &commenter, content, &header);
        assert!(result.is_some());
    }

    #[test]
    fn test_detects_outdated_year_trailing_whitespace() {
        let l = Licensure::new(Config::default());
        let templ = Template::new("License [year]\n\ntext", test_context("2024"));
        let commenter = LineComment::new("#", None);
        let header = commenter.comment(&templ.render());
        let content = "# License 2020\n#\n# text\n";
        let result = l.get_outdated_replacement(&templ, &commenter, content, &header);
        assert!(result.is_some());
    }

    #[test]
    fn test_detects_outdated_year_when_upstream_template_changes_wrapping() {
        let l = Licensure::new(Config::default());
        let templ = Template::new(
            r#"Copyright (C) <year> <name of author>
This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, version 3.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with this program. If not, see <https://www.gnu.org/licenses/>.

"#,
            test_context("2025"),
        ).set_spdx_template(true);
        let commenter = LineComment::new("//", Some(98)).skip_trailing_lines();
        let rendered = templ.render();
        let header = commenter.comment(&rendered);
        let content = r#"// Copyright (C) 2024 Mathew Robinson <chasinglogic@gmail.com>
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
use std::fmt;
use std::fs::File;
use std::io::{self, prelude::*};
use std::sync::LazyLock;

use regex::Regex;

use crate::comments::Comment;
use crate::config::Config;
use crate::template::Template;"#;
        let result = l.get_outdated_replacement(&templ, &commenter, content, &header);
        if let Some(replacement) = result {
            assert_eq!(
                replacement,
                r#"// Copyright (C) 2025 Mathew Robinson <chasinglogic@gmail.com>
// This program is free software: you can redistribute it and/or modify it under the terms of the
// GNU General Public License as published by the Free Software Foundation, version 3.
//
// This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
// without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See
// the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with this program. If
// not, see <https://www.gnu.org/licenses/>.
//
use std::fmt;
use std::fs::File;
use std::io::{self, prelude::*};
use std::sync::LazyLock;

use regex::Regex;

use crate::comments::Comment;
use crate::config::Config;
use crate::template::Template;"#
                    .to_string()
            )
        } else {
            assert!(
                false,
                "get_oudated_replacement didn't match the old header!"
            )
        }
    }

    #[test]
    fn test_detects_replaces() {
        let l = Licensure::new(Config::default());
        let replaces = vec![
            Regex::new("This first regex is not going to hit").expect("Can compile static regex"),
            Regex::new("(// *)?foo \\(C\\) .* another thing\n?").expect("Can compile static regex"),
        ];
        let templ = Template::new("License [year]\n\ntext", test_context("2024"));
        let commenter = LineComment::new("//", None);
        let header = commenter.comment(&templ.render());
        let content = "BEFORE// foo (C) fill fill fill another thing\nAFTER";
        let result = l.get_replaces_replacement(&replaces, content, &header);
        eprintln!("{:?}", result);
        assert!(result.is_some());
        assert!(result
            .unwrap()
            .eq("BEFORE// License 2024\n//\n// text\nAFTER"));
    }

    #[test]
    fn test_add_header() {
        let l = Licensure::new(Config::default());
        let templ = Template::new("License [year]\n\ntext", test_context("2024"));
        let commenter = LineComment::new("#", None);
        let header = commenter.comment(&templ.render());
        let mut content = r#"
def main():
    print('hello world')

if __name__ == '__main__':
    main()
"#
        .to_string();
        let result = l.add_header(header, &mut content);
        assert_eq!(
            result,
            r#"# License 2024
#
# text

def main():
    print('hello world')

if __name__ == '__main__':
    main()
"#
        )
    }

    #[test]
    fn test_add_header_handles_shebang() {
        let l = Licensure::new(Config::default());
        let templ = Template::new("License [year]\n\ntext", test_context("2024"));
        let commenter = LineComment::new("#", None);
        let header = commenter.comment(&templ.render());
        let mut content = r#"#!/usr/bin/env python3

def main():
    print('hello world')

if __name__ == '__main__':
    main()
"#
        .to_string();
        let expected = r#"#!/usr/bin/env python3
# License 2024
#
# text

def main():
    print('hello world')

if __name__ == '__main__':
    main()
"#;

        let result = l.add_header(header, &mut content);
        println!("result: {}", result);
        println!("----------------------");
        println!("expected: {}", expected);
        assert_eq!(result, expected)
    }

    #[test]
    fn test_add_header_ignores_shebang_in_middle_of_file() {
        let l = Licensure::new(Config::default());
        let templ = Template::new("License [year]\n\ntext", test_context("2024"));
        let commenter = LineComment::new("#", None);
        let header = commenter.comment(&templ.render());
        let mut content = r#"
def main():
    print('hello world')

#!/usr/bin/env python3

if __name__ == '__main__':
    main()
"#
        .to_string();
        let expected = r#"# License 2024
#
# text

def main():
    print('hello world')

#!/usr/bin/env python3

if __name__ == '__main__':
    main()
"#;

        let result = l.add_header(header, &mut content);
        assert_eq!(result, expected)
    }

    static CONFIG_WITH_REPLACES: &str = r##"
excludes: []
licenses:
  - files: any
    ident: TESTING
    authors:
      - name: The Tester
    template: "New Test License [name of author]\nOnly For Testing"
    replaces:
      - "(# *)?Before replacement\n?"
comments:
  - columns: 80
    extensions:
      - py
    commenter:
      type: line
      comment_char: "#""##;

    #[test]
    fn test_add_license_header_with_replaces() {
        let config: Config =
            serde_yaml::from_str(CONFIG_WITH_REPLACES).expect("Static config to be parsable");
        let mut l = Licensure::new(config);
        let mut content = r#"
# Before replacement
def main():
    print('hello world')

if __name__ == '__main__':
    main()
"#
        .to_string();
        let result = l.determine_required_action(&"test_file.py".to_string(), &mut content);
        assert_eq!(
            result,
            Action::NeedsUpdate(
                r#"
# New Test License The Tester
# Only For Testing
def main():
    print('hello world')

if __name__ == '__main__':
    main()
"#
                .to_string()
            )
        )
    }

    static CONFIG_DEFAULT_COMMENTER_FALSE: &str = r##"
excludes: []
licenses:
  - files: any
    ident: TESTING
    authors:
      - name: The Tester
    template: "New Test License [name of author]\nOnly For Testing"
comments: []
"##;

    #[test]
    fn test_add_license_header_default_commenter_false() {
        let config: Config = serde_yaml::from_str(CONFIG_DEFAULT_COMMENTER_FALSE)
            .expect("Static config to be parsable");
        let mut l = Licensure::new(config);
        let mut content = r#"
// Before replacement
# include somefile.h
"#
        .to_string();
        let result = l.determine_required_action(&"test_file.c".to_string(), &mut content);
        assert_eq!(result, Action::NoCommenterMatched);
    }
}
