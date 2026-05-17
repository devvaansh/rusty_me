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
pub fn parse(input: &str) -> Vec<(String, String)> {
    let tokens = tokenize(input);
    let mut fields = Vec::new();
    let mut cursor = 0;

    while cursor < tokens.len() {
        match tokens.get(cursor) {
            Some(Token::Atom(key)) => {
                match (tokens.get(cursor + 1), tokens.get(cursor + 2)) {
                    (Some(Token::Equal), Some(Token::Atom(value))) => {
                        fields.push((key.clone(), value.clone()));
                        cursor += 3;
                    }
                    (Some(Token::Equal), _) => {
                        fields.push((key.clone(), String::new()));
                        cursor += 2;
                    }
                    _ => {
                        fields.push((key.clone(), String::new()));
                        cursor += 1;
                    }
                }
            }
            Some(Token::Equal) | None => {
                cursor += 1;
            }
        }
    }

    fields
}

#[cfg(test)]
mod tests {
    use super::{parse, tokenize, Token};

    #[test]
    fn empty_input_returns_no_fields() {
        assert!(parse("").is_empty());
    }

    #[test]
    fn parse_reads_simple_pairs() {
        let fields = parse("level=info msg=hello");

        assert_eq!(
            fields,
            vec![
                ("level".into(), "info".into()),
                ("msg".into(), "hello".into())
            ]
        );
    }

    #[test]
    fn parse_preserves_bare_keys_as_flags() {
        let fields = parse("debug trace=true");

        assert_eq!(
            fields,
            vec![("debug".into(), String::new()), ("trace".into(), "true".into())]
        );
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
