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

use chrono::prelude::*;
use regex::Regex;
use serde::Deserialize;

use crate::comments::Comment;
use crate::utils::remove_column_wrapping;

#[derive(Clone, Deserialize, Debug)]
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

#[derive(Clone, Deserialize, Debug)]
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
    pub end_year: Option<String>,
    pub start_year: Option<String>,
    pub unwrap_text: bool,
}

impl Context {
    fn get_authors(&self) -> String {
        self.authors.to_string()
    }

    fn get_year(&self) -> String {
        let end_year = match &self.end_year {
            Some(year) => year.clone(),
            None => format!("{}", Local::now().year()),
        };

        match &self.start_year {
            Some(start_year) if *start_year != end_year => format!("{}, {}", start_year, end_year),
            _ => end_year,
        }
    }
}

#[derive(Clone)]
pub struct Template {
    spdx_template: bool,
    content: String,
    context: Context,
}

// this token is temporarily used when formatting the template into a comment
// regex with the correct column width that can match the [year] against /[\d]{4}/
//
// this intermediate token needs to be exactly 4 chars long so we can wrap to the
// correct column width, but also be unique enough so that we can subsequently swap
// it for a regex pattern while not colliding with any text that might already be
// in the license text.
const INTERMEDIATE_YEAR_TOKEN: &str = "@YR@";

// Matches any full 4-digit year
const YEAR_RE: &str = "[0-9]{4}(, [0-9]{4})?";

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

    pub fn outdated_license_pattern(&self, commenter: &dyn Comment) -> Regex {
        self.build_year_varying_regex(commenter, false)
    }

    pub fn outdated_license_trimmed_pattern(&self, commenter: &dyn Comment) -> Regex {
        self.build_year_varying_regex(commenter, true)
    }

    pub fn render(&self) -> String {
        self.interpolate(&self.context)
    }

    fn interpolate(&self, context: &Context) -> String {
        let (year_repl, author_repl, ident_repl) = self.replacement_tokens();
        let templ = if self.context.unwrap_text {
            // Some license headers come pre-textwrapped. This regex
            // replacement removes their wrapping while preserving
            // intentional line breaks / empty lines.
            remove_column_wrapping(&self.content.clone())
        } else {
            self.content.clone()
        };

        // Perform our substitutions
        templ
            .replace(year_repl, &context.get_year())
            .replace(author_repl, &context.get_authors())
            .replace(ident_repl, &context.ident)
    }

    fn build_year_varying_regex(&self, commenter: &dyn Comment, trim_trailing: bool) -> Regex {
        let mut context = self.context.clone();

        // interpolate the header with the intermediate year token
        context.end_year = Some(INTERMEDIATE_YEAR_TOKEN.to_string());
        // The year regex accounts for ranges so we don't need to worry about start_year here.
        context.start_year = None;

        let interpolated_header = self.interpolate(&context);
        let mut rendered = commenter.comment(&interpolated_header);

        if trim_trailing {
            rendered = rendered.trim_end().to_string();
        }

        // let's now replace the intermediate year token with a proper
        // regex for a 4-digit year (see const `YEAR_RE`)
        let escaped = rendered
            // split removes all instances of the token, yielding all text fragments
            // around the locations where tokens were excised
            .split(INTERMEDIATE_YEAR_TOKEN)
            // convert to iterable for functional-style chaining
            .collect::<Vec<_>>()
            .into_iter()
            // regex-escape each text fragment so we can match the literal
            // text via regex
            .map(regex::escape)
            // yields a list containing all of the text fragments we want
            // to match as literals via regex
            .collect::<Vec<_>>()
            // joining the fragments with the year-matching regex pattern
            // effectively inserts itself into all the locations where the
            // intermediate token existed. We now have a regex that matches
            // the exact license header text, but with any 4-digit year.
            //
            // And we only care about 4-digit years in our lifetime ;).
            .join(YEAR_RE);

        Regex::new(&escaped).unwrap()
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
pub fn test_context(year: &str) -> Context {
    Context {
        ident: String::from("test"),
        authors: Authors::from(vec![]),
        end_year: Some(String::from(year)),
        start_year: None,
        unwrap_text: true,
    }
}

#[cfg(test)]
pub fn test_context_with_range(start_year: &str, end_year: &str) -> Context {
    Context {
        ident: String::from("test"),
        authors: Authors::from(vec![]),
        end_year: Some(String::from(end_year)),
        start_year: Some(String::from(start_year)),
        unwrap_text: true,
    }
}

#[cfg(test)]
mod tests {
    use crate::comments::LineComment;

    use super::*;

    #[test]
    fn test_year_varying_regex_untrimmed() {
        let context = test_context("2020");
        let template = Template::new("License [year]\n\ntext", context);
        let commenter = LineComment::new("#", None);
        let rgx = template.outdated_license_pattern(&commenter);
        let expected = Regex::new("\\# License [0-9]{4}(, [0-9]{4})?\n\\#\n\\# text\n")
            .expect("This should have compiled?");

        assert_eq!(rgx.to_string(), expected.to_string());
        assert!(rgx.is_match(
            r#"# License 2020
#
# text
"#
        ));
    }

    #[test]
    fn test_year_varying_regex_trimmed() {
        let context = test_context("2020");
        let template = Template::new("License [year]\n\ntext", context);
        let commenter = LineComment::new("#", None);
        let rgx = template.outdated_license_trimmed_pattern(&commenter);
        let expected = Regex::new("\\# License [0-9]{4}(, [0-9]{4})?\n\\#\n\\# text")
            .expect("This should have compiled?");

        assert_eq!(rgx.to_string(), expected.to_string());
        assert!(rgx.is_match(
            r#"# License 2020
#
# text"#
        ));
    }

    #[test]
    fn test_substitution_at_end_of_line() {
        let context = test_context("2020");
        let template = Template::new("License [year]\ntext", context);
        // Newline is removed because of column unwrapping.
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
            end_year: Some(String::from("2020")),
            start_year: None,
            unwrap_text: true,
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
            end_year: Some(String::from("2022")),
            start_year: None,
            unwrap_text: true,
        };
        let template = Template::new(
            "Copyright (C) [year] [name of author] This program is free software.",
            context,
        );
        let commenter: Box<dyn Comment> = Box::new(LineComment::new("#", Option::Some(1000)));
        let re = template.outdated_license_pattern(commenter.as_ref());
        assert!(re.is_match("# Copyright (C) 2020 Mathew Robinson <chasinglogic@gmail.com> This program is free software.\n"))
    }

    #[test]
    fn test_outdated_license_trimmed_matching() {
        let context = Context {
            ident: String::from("test"),
            authors: Authors::from(vec![CopyrightHolder {
                name: "Mathew Robinson".to_string(),
                email: Some("chasinglogic@gmail.com".to_string()),
            }]),
            end_year: Some(String::from("2022")),
            start_year: None,
            unwrap_text: true,
        };
        let template = Template::new(
            "Copyright (C) [year] [name of author] This program is free software.",
            context,
        );
        let commenter: Box<dyn Comment> =
            Box::new(LineComment::new("#", Option::Some(1000)).set_trailing_lines(2));
        let re = template.outdated_license_pattern(commenter.as_ref());
        assert!(re.is_match("# Copyright (C) 2020 Mathew Robinson <chasinglogic@gmail.com> This program is free software.\n\n\n"));
        assert!(!re.is_match("# Copyright (C) 2020 Mathew Robinson <chasinglogic@gmail.com> This program is free software."));

        let trimmed = template.outdated_license_trimmed_pattern(commenter.as_ref());
        assert!(trimmed.is_match("# Copyright (C) 2020 Mathew Robinson <chasinglogic@gmail.com> This program is free software."))
    }

    #[test]
    fn test_substitutions_prewrapped() {
        let context = Context {
            ident: String::from("test"),
            authors: Authors::from(vec![CopyrightHolder {
                name: "Mathew Robinson".to_string(),
                email: Some("chasinglogic@gmail.com".to_string()),
            }]),
            end_year: Some(String::from("2020")),
            start_year: None,
            unwrap_text: true,
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

    // https://github.com/chasinglogic/licensure/issues/25
    #[test]
    fn test_substitutions_unwrap_text_false() {
        let context = Context {
            ident: String::from("test"),
            authors: Authors::from(vec![CopyrightHolder {
                name: "Mathew Robinson".to_string(),
                email: Some("chasinglogic@gmail.com".to_string()),
            }]),
            end_year: Some(String::from("2020")),
            start_year: None,
            unwrap_text: false,
        };
        let template = Template::new(
            "Copyright (c) [name of author]
 SPDX-License-Identifier: [ident]",
            context,
        );
        let expected = String::from(
            "Copyright (c) Mathew Robinson <chasinglogic@gmail.com>
 SPDX-License-Identifier: test",
        );
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
            end_year: Some(String::from("2020")),
            start_year: None,
            unwrap_text: true,
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

    #[test]
    fn test_substitutions_year_ranges() {
        let context = Context {
            ident: String::from("test"),
            authors: Authors::from(vec![CopyrightHolder {
                name: "Mathew Robinson".to_string(),
                email: Some("chasinglogic@gmail.com".to_string()),
            }]),
            end_year: Some(String::from("2024")),
            start_year: Some(String::from("2020")),
            unwrap_text: true,
        };
        let template = Template::new("Copyright (C) [year] [name of author] This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, version 3. This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details. You should have received a copy of the GNU Affero General Public License along with this program. If not, see <https://www.gnu.org/licenses/>", context);
        let expected = String::from("Copyright (C) 2020, 2024 Mathew Robinson <chasinglogic@gmail.com> This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, version 3. This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details. You should have received a copy of the GNU Affero General Public License along with this program. If not, see <https://www.gnu.org/licenses/>");
        assert_eq!(expected, template.render())
    }
}
