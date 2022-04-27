use std::process;

use regex::Regex;
use serde::Deserialize;

use crate::template::{Authors, Context, Template};

#[derive(Deserialize)]
#[serde(from = "String")]
struct FileMatcher {
    any: bool,
    regex: Option<Regex>,
}

impl FileMatcher {
    pub fn is_match(&self, s: &str) -> bool {
        if self.any {
            return true;
        }

        match &self.regex {
            Some(r) => r.is_match(s),
            None => false,
        }
    }
}

impl From<String> for FileMatcher {
    fn from(s: String) -> FileMatcher {
        if s == "any" {
            return FileMatcher {
                any: true,
                regex: None,
            };
        }

        let r = match Regex::new(&s) {
            Ok(r) => r,
            Err(e) => {
                println!("Failed to compile file matcher regex: {}", e);
                process::exit(1);
            }
        };

        FileMatcher {
            any: false,
            regex: Some(r),
        }
    }
}

#[derive(Deserialize)]
struct SPDXLicenseInfo {
    #[serde(alias = "licenseText")]
    license_text: String,
    #[serde(alias = "standardLicenseHeader")]
    license_header: Option<String>,
}

#[derive(Deserialize)]
pub struct Config {
    files: FileMatcher,

    ident: String,
    authors: Authors,
    year: Option<String>,

    template: Option<String>,
    auto_template: Option<bool>,
}

impl Config {
    pub fn file_is_match(&self, s: &str) -> bool {
        self.files.is_match(s)
    }

    async fn fetch_template(&self) -> String {
        let r = match reqwest::get(&format!("https://spdx.org/licenses/{}.json", &self.ident)).await {
            Ok(r) => r,
            Err(e) => {
                println!("Failed to fetch license template from SPDX: {}", e);
                process::exit(1);
            }
        };

        match r.status() {
            reqwest::StatusCode::NOT_FOUND => {
                println!(
                    "{} does not appear to be a valid SPDX identifier, go to https://spdx.org/licenses/ to view a list of valid identifiers",
                    &self.ident
                );
                process::exit(1)
            }
            reqwest::StatusCode::OK => (),
            _ => {
                println!(
                    "Failed to fetch license template from SPDX for {}: {:?}",
                    &self.ident,
                    r.status()
                );
                process::exit(1);
            }
        }

        let json: SPDXLicenseInfo = match r.json().await {
            Ok(j) => j,
            Err(e) => {
                println!("Failed to deserialize SPDX JSON: {}", e);
                process::exit(1);
            }
        };

        match json.license_header {
            Some(header) => header,
            None => json.license_text,
        }
    }

    pub async fn get_template(&self) -> Template {
        let auto_templ;
        let t = match &self.template {
            Some(ref t) => t,
            None => {
                if self.auto_template.unwrap_or(false) {
                    auto_templ = self.fetch_template().await;
                    &auto_templ
                } else {
                    println!("auto_template not enabled and no template provided, please add a template option to the license definition for {}. Exitting", self.ident);
                    process::exit(1);
                }
            }
        };

        let t = Template::new(
            t,
            Context {
                ident: self.ident.clone(),
                year: self.year.clone(),
                authors: self.authors.clone(),
            },
        );

        if self.auto_template.unwrap_or(false) {
            return t.set_spdx_template(true);
        }

        t
    }
}
