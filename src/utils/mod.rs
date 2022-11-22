pub fn trim_trailing_whitespace(string: &str) -> &str {
    string.trim_end_matches(|c| c == '\n' || c == '\r' || c == ' ')
}
