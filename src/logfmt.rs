/// Parses a logfmt input string into key-value fields.
///
/// The parser is intentionally small at first so fuzzing can drive its growth.
pub fn parse(_input: &str) -> Vec<(String, String)> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn empty_input_returns_no_fields() {
        assert!(parse("").is_empty());
    }
}
