// Copyright (C) 2025 Mathew Robinson <chasinglogic@gmail.com>
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free Software
// Foundation, version 3.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
// FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for
// more details.
//
// You should have received a copy of the GNU General Public License along with
// this program. If not, see <https://www.gnu.org/licenses/>.
//
use std::process::{self, Command};

use chrono::Local;
use regex::Regex;
use serde::{Deserialize, Deserializer};

use super::RegexList;
use crate::template::{Authors, Context, Template};

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum FileMatcher {
    #[serde(deserialize_with = "deserialize_file_matcher_any")]
    Any,
    #[serde(with = "serde_regex")]
    Single(Regex),
    RegexList(RegexList),
}

impl FileMatcher {
    pub fn is_match(&self, s: &str) -> bool {
        match self {
            FileMatcher::Any => true,
            FileMatcher::Single(r) => r.is_match(s),
            FileMatcher::RegexList(ref regex_list) => regex_list.is_match(s),
        }
    }
}

/// Tries to deserialize an "any" string and errors on any other input
fn deserialize_file_matcher_any<'de, D>(deserializer: D) -> Result<(), D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;
    if buf == "any" {
        Ok(())
    } else {
        Err(serde::de::Error::custom("Not an 'any'"))
    }
}

#[derive(Deserialize)]
struct SPDXLicenseInfo {
    #[serde(alias = "licenseText")]
    license_text: String,
    #[serde(alias = "standardLicenseHeader")]
    license_header: Option<String>,
}

fn fetch_template(ident: &str) -> String {
    let url = format!("https://spdx.org/licenses/{}.json", ident);
    let response = match ureq::get(&url).call() {
        Ok(r) => r,
        Err(e) => {
            println!("Failed to fetch license template from SPDX: {}", e);
            process::exit(1);
        }
    };

    match response.status() {
        404 => {
            println!(
                    "{} does not appear to be a valid SPDX identifier, go to https://spdx.org/licenses/ to view a list of valid identifiers",
                    ident
                );
            process::exit(1)
        }
        200 => (),
        _ => {
            println!(
                "Failed to fetch license template from SPDX for {}: {:?}",
                ident,
                response.status()
            );
            process::exit(1);
        }
    }

    let license_info: SPDXLicenseInfo = match response.into_json() {
        Ok(json) => json,
        Err(err) => {
            println!("Failed to deserialize SPDX JSON: {}", err);
            process::exit(1);
        }
    };

    match license_info.license_header {
        Some(header) => header,
        None => license_info.license_text,
    }
}

fn default_unwrap_text() -> bool {
    true
}

fn default_dynamic_year_ranges() -> bool {
    false
}

#[derive(Deserialize, Debug)]
pub struct Config {
    files: FileMatcher,

    ident: String,
    authors: Authors,
    #[serde(alias = "year")]
    end_year: Option<String>,
    start_year: Option<String>,
    #[serde(default = "default_dynamic_year_ranges")]
    use_dynamic_year_ranges: bool,

    template: Option<String>,
    auto_template: Option<bool>,

    #[serde(with = "serde_regex", default)]
    replaces: Option<Vec<Regex>>,

    #[serde(default = "default_unwrap_text")]
    unwrap_text: bool,
}

impl Config {
    pub fn file_is_match(&self, s: &str) -> bool {
        self.files.is_match(s)
    }

    pub fn get_template(&mut self, filename: &str) -> Template {
        let auto_templ;
        let t = match &self.template {
            Some(ref t) => t,
            None => {
                if self.auto_template.unwrap_or(false) {
                    auto_templ = fetch_template(&self.ident);
                    self.template = Some(auto_templ.clone());
                    &auto_templ
                } else {
                    println!("auto_template not enabled and no template provided, please add a template option to the license definition for {}. Exitting", self.ident);
                    process::exit(1);
                }
            }
        };

        let (end_year, start_year) = if self.use_dynamic_year_ranges {
            let now_date = Local::now().format("%a %b %d %T %Y %z").to_string();
            let dates = get_git_dates_for_file(filename);
            let (last_updated_date, created_date) = match &dates[..] {
                [first_date, .., last_date] => (first_date, last_date),
                [first_date] => (first_date, first_date),
                _ => {
                    debug!("Did not get any dates from git for file: {}", filename);
                    (&now_date, &now_date)
                }
            };

            // Git formats the dates such that we get "Wed May 29 04:54:58 2024 +0100" we only care
            // about the 4th "field" which is the year.
            let created_year = created_date
                .split(' ')
                .nth(4)
                .expect("Unable to parse created year!");
            let last_updated_year = last_updated_date
                .split(' ')
                .nth(4)
                .expect("Unable to parse last updated year!");

            (
                Some(last_updated_year.to_string()),
                Some(created_year.to_string()),
            )
        } else {
            (self.end_year.clone(), self.start_year.clone())
        };

        let t = Template::new(
            t,
            Context {
                end_year,
                start_year,
                ident: self.ident.clone(),
                authors: self.authors.clone(),
                unwrap_text: self.unwrap_text,
            },
        );

        if self.auto_template.unwrap_or(false) {
            return t.set_spdx_template(true);
        }

        t
    }

    pub fn get_replaces(&self) -> &Option<Vec<Regex>> {
        &self.replaces
    }
}

fn get_git_dates_for_file(filename: &str) -> Vec<String> {
    match Command::new("git")
        .arg("log")
        .arg("--follow")
        .arg("--format=%ad")
        .args(["--date", "default"])
        .arg(filename)
        .output()
    {
        Ok(proc) => String::from_utf8(proc.stdout)
            .expect("git log output was not UTF-8!")
            .split('\n')
            .map(str::to_string)
            .filter(|s| !s.is_empty())
            .collect(),
        Err(e) => {
            println!("Failed to run git log to get file dates. Make sure you're in a git repo.");
            println!("{}", e);
            process::exit(1)
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    static ANY_TEST: &str = r#"
files: any
ident: foo
authors:
    - name: Author1
      email: a@example.com
template: "some license"
"#;
    #[test]
    fn test_deserialize_file_matcher_any() {
        let test: Config =
            serde_yaml::from_str(ANY_TEST).expect("To be able to parse static config");
        if let FileMatcher::Any = test.files {
            // This is good...
        } else {
            panic!(
                "Expected to have any type for files but had: {:?}",
                test.files
            );
        };
    }

    static REGEX_TEST: &str = r#"
files: .*foo
ident: foo
authors:
    - name: Author1
      email: a@example.com
template: "some license"
"#;
    #[test]
    fn test_deserialize_file_matcher_regex() {
        let test: Config =
            serde_yaml::from_str(REGEX_TEST).expect("To be able to parse static config");
        if let FileMatcher::Single(r) = test.files {
            assert_eq!(".*foo", r.as_str());
            return;
        }
        panic!(
            "Expected to have a single regex for files but had: {:?}",
            test.files
        );
    }
    static REGEX_LIST_TEST: &str = r#"
files: 
    - a.*
    - b.*
ident: foo
authors:
    - name: Author1
      email: a@example.com
template: "some license"
"#;
    #[test]
    fn test_deserialize_file_matcher_regex_list() {
        let test: Config =
            serde_yaml::from_str(REGEX_LIST_TEST).expect("To be able to parse static config");
        if let FileMatcher::RegexList(r) = test.files {
            assert_eq!(&["a.*", "b.*"], r.regex.patterns());
            return;
        }
        panic!(
            "Expected to have a RegexList for files but had: {:?}",
            test.files
        );
    }
}
