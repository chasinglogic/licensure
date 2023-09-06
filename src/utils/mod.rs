use regex::Regex;

pub fn remove_column_wrapping(string: &str) -> String {
    // Some license headers come pre-wrapped to a column width.
    // This regex replacement undoes the column-width wrapping
    // while preserving intentional line breaks / empty lines.
    let re = Regex::new(r"(?P<char>.)\n").unwrap();
    re.replace_all(string, "$char ").replace(" \n", "\n\n")
}

#[cfg(test)]
mod tests {
    use crate::utils::remove_column_wrapping;

    #[test]
    fn test_remove_column_wrapping() {
        let content = "\
some wrapped
text to unwrap.

The line above
is an intentional
line break.

So is this.";

        let expected = "some wrapped text to unwrap.\n\nThe line above \
        is an intentional line break.\n\nSo is this.";
        assert_eq!(expected, remove_column_wrapping(&content))
    }
}
