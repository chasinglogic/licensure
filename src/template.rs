use std::fmt;

use chrono::prelude::*;
use regex::Regex;
use serde::Deserialize;

use crate::comments::Comment;

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

// this token needs to be exactly 4 chars; this is temporarily used when formatting
// the template into a comment regex with the correct column width that can match
// the [year] against /[\d]{4}/
const INTERMEDIATE_YEAR_TOKEN: &'static str = "@YR@";

// Matches any full 4-digit year
const YEAR_RE: &'static str = "[0-9]{4}";

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

    pub fn outdated_license_pattern(&self, commenter: &Box<dyn Comment>, columns: Option<usize>) -> Regex {
        self.build_year_varying_regex(&mut self.content.clone(), commenter, columns, false)
    }

    pub fn outdated_license_trimmed_pattern(&self, commenter: &Box<dyn Comment>, columns: Option<usize>) -> Regex {
        self.build_year_varying_regex(&mut self.content.clone(), commenter, columns, true)
    }

    pub fn render(&self) -> String {
        let (year_repl, author_repl, ident_repl) = self.replacement_tokens();
        let template = self.unwrap_license_headers(&mut self.content.clone());

        // Perform our substitutions
        template
            .replace(year_repl, &self.context.get_year())
            .replace(author_repl, &self.context.get_authors())
            .replace(ident_repl, &self.context.ident)
    }

    fn build_year_varying_regex(&self, content: &mut str, commenter: &Box<dyn Comment>, columns: Option<usize>, trim_trailing: bool) -> Regex {
        let (year_repl, author_repl, ident_repl) = self.replacement_tokens();
        let mut rendered = commenter.comment(
            &(self.unwrap_license_headers(content)
                .replace(year_repl, INTERMEDIATE_YEAR_TOKEN)
                .replace(author_repl, &self.context.get_authors())
                .replace(ident_repl, &self.context.ident)),
            columns,
        );

        if trim_trailing {
            rendered = rendered.trim_end().to_string();
        }

        let escaped: Vec<_> = rendered
            .split(INTERMEDIATE_YEAR_TOKEN)
            .collect::<Vec<_>>()
            .into_iter()
            .map(|frag| { regex::escape(frag) })
            .collect::<Vec<_>>();

        Regex::new(&(escaped.join(YEAR_RE))).unwrap()
    }

    fn unwrap_license_headers(&self, template: &mut str) -> String {
        // Some license headers come pre-textwrapped. This regex
        // replacement removes their wrapping while preserving
        // intentional line breaks / empty lines.
        let re = Regex::new(r"(?P<char>.)\n").unwrap();
        re.replace_all(&template, "$char ").to_string()
    }

    fn replacement_tokens(&self) -> (&'static str, &'static str, &'static str) {
        if self.spdx_template {
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
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::comments::LineComment;

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
    fn test_outdated_license_matching() {
        let context = Context {
            ident: String::from("test"),
            authors: Authors::from(vec![CopyrightHolder {
                name: "Mathew Robinson".to_string(),
                email: Some("chasinglogic@gmail.com".to_string()),
            }]),
            year: Some(String::from("2022")),
        };
        let template = Template::new("Copyright (C) [year] [name of author] This program is free software.", context);
        let commenter: Box<dyn Comment> = Box::new(LineComment::new("#"));
        let re = template.outdated_license_pattern(&commenter, Option::Some(1000));
        assert_eq!(true, re.is_match("# Copyright (C) 2020 Mathew Robinson <chasinglogic@gmail.com> This program is free software.\n"))
    }

    #[test]
    fn test_outdated_license_trimmed_matching() {
        let context = Context {
            ident: String::from("test"),
            authors: Authors::from(vec![CopyrightHolder {
                name: "Mathew Robinson".to_string(),
                email: Some("chasinglogic@gmail.com".to_string()),
            }]),
            year: Some(String::from("2022")),
        };
        let template = Template::new("Copyright (C) [year] [name of author] This program is free software.", context);
        let commenter: Box<dyn Comment> = Box::new(LineComment::new("#").set_trailing_lines(2));
        let re = template.outdated_license_pattern(&commenter, Option::Some(1000));
        assert_eq!(true, re.is_match("# Copyright (C) 2020 Mathew Robinson <chasinglogic@gmail.com> This program is free software.\n\n\n"));
        assert_eq!(false, re.is_match("# Copyright (C) 2020 Mathew Robinson <chasinglogic@gmail.com> This program is free software."));

        let trimmed = template.outdated_license_trimmed_pattern(&commenter, Option::Some(1000));
        assert_eq!(true, trimmed.is_match("# Copyright (C) 2020 Mathew Robinson <chasinglogic@gmail.com> This program is free software."))
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
