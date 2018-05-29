const SHORT_IDENT: &str =
    "Copyright {year} {author} <{email}>. All rights reserved. Use of this source code is
governed by the {ident} license that can be found in the LICENSE file.

";

const AGPLV3: &str = "Copyright {year} {author} <{email}>. All rights reserved.

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

";

const GPLV3: &str = "Copyright {year} {author} <{email}>. All rights reserved.

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

";

pub fn get_template(ident: &str) -> &str {
    match ident {
        "short-ident" => SHORT_IDENT,
        "GPL-3.0" => GPLV3,
        "AGPL-3.0" => AGPLV3,
        _ => SHORT_IDENT,
    }
}

pub fn render_template(ident: &str, author: &str, email: &str, year: &str) -> String {
    get_template(ident)
        .replace("{year}", year)
        .replace("{author}", author)
        .replace("{email}", email)
        .replace("{ident}", ident)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_returns_short_ident_by_default() {
        assert_eq!(SHORT_IDENT, get_template("NOTALICENSE"));
    }

    #[test]
    fn test_returns_gplv3() {
        assert_eq!(GPLV3, get_template("GPL-3.0"));
    }

    #[test]
    fn test_returns_agplv3() {
        assert_eq!(AGPLV3, get_template("AGPL-3.0"));
    }

    #[test]
    fn test_render_short_ident() {
        assert_eq!("Copyright 2018 Mathew Robinson <chasinglogic@gmail.com>. All rights reserved. Use of this source code is
governed by the MIT license that can be found in the LICENSE file.

", render_template("MIT", "Mathew Robinson", "chasinglogic@gmail.com", "2018"));
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
            render_template(
                "GPL-3.0",
                "Mathew Robinson",
                "chasinglogic@gmail.com",
                "2018"
            )
        );
    }
}
