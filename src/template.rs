use chrono::prelude::*;
use regex::Regex;
use serde::Deserialize;
use std::fmt;

#[derive(Clone, Deserialize)]
struct CopyrightHolder {
    name: String,
    email: Option<String>,
}

impl fmt::Display for CopyrightHolder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut a = self.name.clone();

        if let Some(email) = &self.email {
            a.push_str(&format!(" <{}>", email));
        }

        write!(f, "{}", a)
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

impl fmt::Display for Authors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut a = String::new();

        for author in &self.authors {
            if !a.is_empty() {
                a.push_str(", ");
            }

            a.push_str(&author.to_string());
        }

        write!(f, "{}", a)
    }
}

#[derive(Clone)]
pub struct Context {
    pub ident: String,
    pub authors: Authors,
    pub year: Option<String>,
    pub unwrap_text: bool,
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

        if self.context.unwrap_text {
            // Some license headers come pre-textwrapped. This regex
            // replacement removes their wrapping while preserving
            // intentional line breaks / empty lines.
            let re = Regex::new(r"(?P<char>.)\n").unwrap();
            templ = re.replace_all(&templ, "$char ").to_string();
        }

        // Perform our substitutions
        templ
            .replace(year_repl, &self.context.get_year())
            .replace(author_repl, &self.context.get_authors())
            .replace(ident_repl, &self.context.ident)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitution_at_end_of_line() {
        let context = Context {
            ident: String::from("test"),
            authors: Authors::from(vec![]),
            year: Some(String::from("2020")),
        };
        let template = Template::new("License [year]\ntext", context);
        let expected = String::from("License 2020 text");
        assert_eq!(expected, template.render())
    }

    #[test]
    fn test_substitutions() {
        let context = Context {
            ident: String::from("test"),
            authors: Authors::from(vec![CopyrightHolder {
                name: "Mathew Robinson".to_string(),
                email: Some("chasinglogic@gmail.com".to_string()),
            }]),
            year: Some(String::from("2020")),
        };
        let template = Template::new("Copyright (C) [year] [name of author] This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, version 3. This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details. You should have received a copy of the GNU Affero General Public License along with this program. If not, see <https://www.gnu.org/licenses/>", context);
        let expected = String::from("Copyright (C) 2020 Mathew Robinson <chasinglogic@gmail.com> This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, version 3. This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details. You should have received a copy of the GNU Affero General Public License along with this program. If not, see <https://www.gnu.org/licenses/>");
        assert_eq!(expected, template.render())
    }

    #[test]
    fn test_substitutions_prewrapped() {
        let context = Context {
            ident: String::from("test"),
            authors: Authors::from(vec![CopyrightHolder {
                name: "Mathew Robinson".to_string(),
                email: Some("chasinglogic@gmail.com".to_string()),
            }]),
            year: Some(String::from("2020")),
        };
        let template = Template::new(
            "Copyright (C) [year] [name of author] This
program is free software: you can redistribute it and/or modify it under
the terms of the GNU Affero General Public License as published by the
Free Software Foundation, version 3. This program is distributed in the
hope that it will be useful, but WITHOUT ANY WARRANTY; without even the
implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
See the GNU Affero General Public License for more details. You should
have received a copy of the GNU Affero General Public License along with
this program. If not, see <https://www.gnu.org/licenses/>",
            context,
        );
        let expected = String::from("Copyright (C) 2020 Mathew Robinson <chasinglogic@gmail.com> This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, version 3. This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details. You should have received a copy of the GNU Affero General Public License along with this program. If not, see <https://www.gnu.org/licenses/>");
        assert_eq!(expected, template.render())
    }

    #[test]
    fn test_substitutions_prewrapped_preserves_linebreaks() {
        let context = Context {
            ident: String::from("test"),
            authors: Authors::from(vec![CopyrightHolder {
                name: "Mathew Robinson".to_string(),
                email: Some("chasinglogic@gmail.com".to_string()),
            }]),
            year: Some(String::from("2020")),
        };
        let template = Template::new(
            "Copyright (C) [year] [name of author] This
program is free software: you can redistribute it and/or modify it under
the terms of the GNU Affero General Public License as published by the

Free Software Foundation, version 3. This program is distributed in the
hope that it will be useful, but WITHOUT ANY WARRANTY; without even the
implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
See the GNU Affero General Public License for more details. You should
have received a copy of the GNU Affero General Public License along with
this program. If not, see <https://www.gnu.org/licenses/>",
            context,
        );
        let expected = String::from("Copyright (C) 2020 Mathew Robinson <chasinglogic@gmail.com> This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the 
Free Software Foundation, version 3. This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details. You should have received a copy of the GNU Affero General Public License along with this program. If not, see <https://www.gnu.org/licenses/>");
        assert_eq!(expected, template.render())
    }
}
