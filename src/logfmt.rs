#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Atom(String),
    Equal,
}

/// Breaks a logfmt input into a minimal token stream.
pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    for part in input.split_ascii_whitespace() {
        if let Some((key, value)) = part.split_once('=') {
            if !key.is_empty() {
                tokens.push(Token::Atom(key.to_owned()));
            }
            tokens.push(Token::Equal);
            if !value.is_empty() {
                tokens.push(Token::Atom(value.to_owned()));
            }
        } else {
            tokens.push(Token::Atom(part.to_owned()));
        }
    }

    tokens
}

/// Parses a logfmt input string into key-value fields.
///
/// The parser is intentionally small at first so fuzzing can drive its growth.
pub fn parse(_input: &str) -> Vec<(String, String)> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::{parse, tokenize, Token};

    #[test]
    fn empty_input_returns_no_fields() {
        assert!(parse("").is_empty());
    }

    #[test]
    fn tokenize_splits_simple_key_values() {
        let tokens = tokenize("level=info msg=hello");

        assert_eq!(
            tokens,
            vec![
                Token::Atom("level".into()),
                Token::Equal,
                Token::Atom("info".into()),
                Token::Atom("msg".into()),
                Token::Equal,
                Token::Atom("hello".into()),
            ]
        );
    }

    #[test]
    fn tokenize_keeps_bare_atoms() {
        let tokens = tokenize("orphan");

        assert_eq!(tokens, vec![Token::Atom("orphan".into())]);
    }
}
