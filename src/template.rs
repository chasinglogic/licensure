use chrono::prelude::*;
use regex::Regex;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
struct CopyrightHolder {
    name: String,
    email: Option<String>,
}

impl CopyrightHolder {
    fn to_string(&self) -> String {
        let mut a = self.name.clone();

        if let Some(email) = &self.email {
            a.push_str(&format!(" <{}>", email));
        }

        a
    }
}

#[derive(Clone, Deserialize)]
#[serde(from = "Vec<CopyrightHolder>")]
pub struct Authors {
    authors: Vec<CopyrightHolder>,
}

impl From<Vec<CopyrightHolder>> for Authors {
    fn from(authors: Vec<CopyrightHolder>) -> Authors {
        Authors { authors }
    }
}

impl Authors {
    fn to_string(&self) -> String {
        let mut a = String::new();

        for author in &self.authors {
            if a != "" {
                a.push_str(", ");
            }

            a.push_str(&author.to_string());
        }

        a
    }
}

#[derive(Clone)]
pub struct Context {
    pub ident: String,
    pub authors: Authors,
    pub year: Option<String>,
}

impl Context {
    fn get_authors(&self) -> String {
        self.authors.to_string()
    }

    fn get_year(&self) -> String {
        match &self.year {
            Some(year) => year.clone(),
            None => format!("{}", Local::now().year()),
        }
    }
}

#[derive(Clone)]
pub struct Template {
    spdx_template: bool,
    content: String,
    context: Context,
}

impl Template {
    pub fn new(template: &str, context: Context) -> Template {
        Template {
            spdx_template: false,
            content: template.to_string(),
            context,
        }
    }

    pub fn set_spdx_template(mut self, yes_or_no: bool) -> Template {
        self.spdx_template = yes_or_no;
        self
    }

    pub fn render(self) -> String {
        let (year_repl, author_repl, ident_repl) = if self.spdx_template {
            // Check if it's the Apache license which has a super
            // special format.
            if self.content.contains("[name of copyright owner]") {
                ("[yyyy]", "[name of copyright owner]", "[ident]")
            } else {
                (
                    "<year>",
                    if self.content.contains("<copyright holders>") {
                        "<copyright holders>"
                    } else if self.content.contains("<owner>") {
                        "<owner>"
                    } else {
                        "<name of author>"
                    },
                    "<ident>",
                )
            }
        } else {
            ("[year]", "[name of author]", "[ident]")
        };

        let mut templ = self.content.clone();

        // Some license headers come pre-textwrapped. This regex
        // replacement removes their wrapping while preserving
        // intentional newlines.
        let re = Regex::new("[A-z0-9]\n").unwrap();
        templ = re.replace_all(&templ, " ").to_string();

        // Perform our substitutions
        templ
            .replace(year_repl, &self.context.get_year())
            .replace(author_repl, &self.context.get_authors())
            .replace(ident_repl, &self.context.ident)
    }
}
