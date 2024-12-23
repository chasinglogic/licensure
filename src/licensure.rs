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
use std::fs::File;
use std::io::{self, prelude::*};

use regex::Regex;

use crate::comments::Comment;
use crate::config::Config;
use crate::template::Template;

pub struct Licensure {
    config: Config,
    stats: LicenseStats,
    check_mode: bool,
}

enum LicenseStatus {
    NeedsUpdate(String),
    AlreadyLicensed,
    NoConfigMatched,
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

    pub fn license_files(mut self, files: &[String]) -> Result<LicenseStats, io::Error> {
        self.stats = LicenseStats::new();

        for file in files {
            if self.config.excludes.is_match(file) {
                info!("skipping {} because it is excluded.", file);
                continue;
            }

            let mut content = String::new();
            {
                let mut f = File::open(file)?;
                f.read_to_string(&mut content)?;
            }

            match self.add_license_header(file, &mut content) {
                LicenseStatus::NeedsUpdate(update) => self.handle_update(file, &update)?,
                LicenseStatus::NoConfigMatched => self.stats.files_not_licensed.push(file.clone()),
                LicenseStatus::AlreadyLicensed => continue,
            }
        }

        Ok(self.stats)
    }

    fn handle_update(&self, file: &String, content: &str) -> Result<(), io::Error> {
        if self.check_mode {
            return Result::Ok(());
        }

        if self.config.change_in_place {
            let mut f = File::create(file)?;
            return f.write_all(content.as_bytes());
        }

        println!("{}", content);
        Result::Ok(())
    }

    fn strip_shebang_if_found(content: &mut String) -> Option<String> {
        // Can't use Option::map because of double borrow.
        #[allow(clippy::manual_map)]
        match Regex::new(r"^#!.*\n")
            .expect("shebang regex didn't compile!")
            .find(content)
        {
            // If we idenfied a shebang, strip it from content (we'll add it back at the end)
            Some(shebang_match) => Some(content.drain(..shebang_match.end()).collect()),
            None => None,
        }
    }

    fn check_if_outdated(
        &self,
        templ: &Template,
        commenter: &dyn Comment,
        content: &str,
        header: &str,
    ) -> Option<String> {
        let outdated_re = templ.outdated_license_pattern(commenter);
        println!("{}", content);
        println!("{:?}", outdated_re);
        if outdated_re.is_match(content) {
            return Some(outdated_re.replace(content, header).to_string());
        }

        // Account for possible whitespace changes
        let trimmed_outdated_re = templ.outdated_license_trimmed_pattern(commenter);
        if trimmed_outdated_re.is_match(content) {
            Some(trimmed_outdated_re.replace(content, header).to_string())
        } else {
            None
        }
    }

    fn check_if_needs_replacement(
        &self,
        replaces: &Vec<Regex>,
        _commenter: &dyn Comment,
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
            println!("Shebang: {}", value);
            header.insert_str(0, &value);
        }

        header.push_str(content);
        header
    }

    fn add_license_header(&mut self, file: &String, content: &mut String) -> LicenseStatus {
        let templ = match self.config.licenses.get_template(file) {
            Some(t) => t,
            None => {
                info!("skipping {} because no license config matched.", file);
                return LicenseStatus::NoConfigMatched;
            }
        };

        let commenter = self.config.comments.get_commenter(file);

        let uncommented = templ.render();
        let header = commenter.comment(&uncommented);
        if content.contains(&header) || content.contains(header.trim_end()) {
            info!("{} already licensed", file);
            return LicenseStatus::AlreadyLicensed;
        }

        if let Some(update) = self.check_if_outdated(&templ, commenter.as_ref(), content, &header) {
            info!("{} licensed, but year is outdated", file);
            self.stats.files_needing_license_update.push(file.clone());
            return LicenseStatus::NeedsUpdate(update);
        }

        if let Some(replaces) = self.config.licenses.get_replaces(file) {
            if let Some(update) =
                self.check_if_needs_replacement(replaces, commenter.as_ref(), content, &header)
            {
                info!("{} licensed, but license is outdated", file);
                self.stats.files_needing_license_update.push(file.clone());
                return LicenseStatus::NeedsUpdate(update);
            }
        }

        self.stats.files_needing_license_update.push(file.clone());
        LicenseStatus::NeedsUpdate(self.add_header(header, content))
    }
}

pub struct LicenseStats {
    pub files_not_licensed: Vec<String>,
    pub files_needing_license_update: Vec<String>,
}

impl LicenseStats {
    fn new() -> Self {
        Self {
            files_not_licensed: Vec::new(),
            files_needing_license_update: Vec::new(),
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

    #[test]
    fn test_detects_outdated_year() {
        let l = Licensure::new(Config::default());
        let templ = Template::new("License [year]\n\ntext", test_context("2024"));
        let commenter = LineComment::new("#", None);
        let header = commenter.comment(&templ.render());
        let content = "# License 2020\n#\n# text";
        let result = l.check_if_outdated(&templ, &commenter, content, &header);
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
        let result = l.check_if_outdated(&templ, &commenter, content, &header);
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
        let result = l.check_if_outdated(&templ, &commenter, content, &header);
        assert!(result.is_some());
    }

    #[test]
    fn test_detects_outdated_year_trailing_whitespace() {
        let l = Licensure::new(Config::default());
        let templ = Template::new("License [year]\n\ntext", test_context("2024"));
        let commenter = LineComment::new("#", None);
        let header = commenter.comment(&templ.render());
        let content = "# License 2020\n#\n# text\n";
        let result = l.check_if_outdated(&templ, &commenter, content, &header);
        assert!(result.is_some());
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
}
