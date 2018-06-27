use chrono::prelude::*;

mod templates;
use self::templates::*;

pub struct Config {
    ident: String,
    author: String,
    year: String,
    email: Option<String>,
}

impl Config {
    pub fn new(ident: &str, author: &str) -> Config {
        Config {
            ident: ident.to_string(),
            author: author.to_string(),
            email: None,
            year: Local::now().year().to_string(),
        }
    }

    pub fn with_email(mut self, email: String) -> Config {
        self.email = Some(email);
        self
    }

    pub fn with_year(mut self, year: String) -> Config {
        self.year = year;
        self
    }

    pub fn render(&self) -> String {
        let mut rendered = self.template()
            .replace("{year}", &self.year)
            .replace("{author}", &self.author)
            .replace("{ident}", &self.ident);

        if let Some(ref email) = self.email {
            let mut tmpl_email = "<".to_string();
            tmpl_email.push_str(&email);
            tmpl_email.push_str(">");
            rendered = rendered.replace("{email}", &tmpl_email);
        }

        rendered
    }

    pub fn template(&self) -> String {
        let template = match self.ident.as_str() {
            "short-ident" => SHORT_IDENT,
            "GPL-3.0" => GPLV3,
            "AGPL-3.0" => AGPLV3,
            _ => SHORT_IDENT,
        };

        template.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_returns_short_ident_by_default() {
        assert_eq!(
            SHORT_IDENT,
            Config::new("NOTALICENSE", "Mathew Robinson").template()
        );
    }

    #[test]
    fn test_returns_gplv3() {
        assert_eq!(GPLV3, Config::new("GPL-3.0", "Mathew Robinson").template());
    }

    #[test]
    fn test_returns_agplv3() {
        assert_eq!(
            AGPLV3,
            Config::new("AGPL-3.0", "Mathew Robinson").template()
        );
    }

    #[test]
    fn test_render_short_ident() {
        assert_eq!("Copyright 2018 Mathew Robinson <chasinglogic@gmail.com>. All rights reserved. Use of this source code is
governed by the MIT license that can be found in the LICENSE file.

",
            Config::new("MIT", "Mathew Robinson")
                .with_email("chasinglogic@gmail.com")
                .with_year("2018")
                .render()
);
    }

    #[test]
    fn test_render_gplv3() {
        assert_eq!(
            "Copyright 2018 Mathew Robinson <chasinglogic@gmail.com>. All rights reserved.

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.

",
            Config::new("MIT", "Mathew Robinson")
                .with_email("chasinglogic@gmail.com")
                .with_year("2018")
                .render()
        );
    }
}
