#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Atom(String),
    Equal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub key: String,
    pub value: Option<String>,
}

impl Field {
    pub fn flag(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: None,
        }
    }

    pub fn pair(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: Some(value.into()),
        }
    }

    pub fn is_flag(&self) -> bool {
        self.value.is_none()
    }
}

/// Breaks a logfmt input into a minimal token stream.
pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    let mut cursor = input.chars().peekable();

    while let Some(ch) = cursor.peek() {
        if ch.is_ascii_whitespace() {
            cursor.next();
            continue;
        }

        if *ch == '"' {
            cursor.next();
            tokens.push(Token::Atom(read_quoted_value(&mut cursor)));
            continue;
        }

        let atom = read_until_delimiter(&mut cursor);

        if matches!(cursor.peek(), Some('=')) {
            if !atom.is_empty() {
                tokens.push(Token::Atom(atom));
            }

            cursor.next();
            tokens.push(Token::Equal);

            match cursor.peek() {
                Some('"') => {
                    cursor.next();
                    tokens.push(Token::Atom(read_quoted_value(&mut cursor)));
                }
                Some(next) if !next.is_ascii_whitespace() => {
                    let value = read_until_whitespace(&mut cursor);
                    tokens.push(Token::Atom(value));
                }
                _ => {}
            }
        } else if !atom.is_empty() {
            tokens.push(Token::Atom(atom));
        }
    }

    tokens
}

fn read_until_delimiter<I>(cursor: &mut std::iter::Peekable<I>) -> String
where
    I: Iterator<Item = char>,
{
    let mut atom = String::new();

    while let Some(ch) = cursor.peek() {
        if ch.is_ascii_whitespace() || *ch == '=' {
            break;
        }

        atom.push(*ch);
        cursor.next();
    }

    atom
}

fn read_until_whitespace<I>(cursor: &mut std::iter::Peekable<I>) -> String
where
    I: Iterator<Item = char>,
{
    let mut value = String::new();

    while let Some(ch) = cursor.peek() {
        if ch.is_ascii_whitespace() {
            break;
        }

        value.push(*ch);
        cursor.next();
    }

    value
}

fn read_quoted_value<I>(cursor: &mut std::iter::Peekable<I>) -> String
where
    I: Iterator<Item = char>,
{
    let mut value = String::new();

    while let Some(ch) = cursor.next() {
        match ch {
            '"' => break,
            '\\' => {
                if let Some(escaped) = cursor.next() {
                    value.push(match escaped {
                        '"' => '"',
                        '\\' => '\\',
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        other => other,
                    });
                } else {
                    value.push('\\');
                }
            }
            other => value.push(other),
        }
    }

    value
}

/// Parses a logfmt input string into key-value fields.
///
/// The parser is intentionally small at first so fuzzing can drive its growth.
pub fn parse(input: &str) -> Vec<(String, String)> {
    parse_fields(input)
        .into_iter()
        .map(|field| (field.key, field.value.unwrap_or_default()))
        .collect()
}

/// Parses a logfmt input string into structured fields.
pub fn parse_fields(input: &str) -> Vec<Field> {
    let tokens = tokenize(input);
    let mut fields = Vec::new();
    let mut cursor = 0;

    while cursor < tokens.len() {
        match tokens.get(cursor) {
            Some(Token::Atom(key)) => match (tokens.get(cursor + 1), tokens.get(cursor + 2)) {
                (Some(Token::Equal), Some(Token::Atom(value))) => {
                    fields.push(Field::pair(key.clone(), value.clone()));
                    cursor += 3;
                }
                (Some(Token::Equal), _) => {
                    fields.push(Field {
                        key: key.clone(),
                        value: Some(String::new()),
                    });
                    cursor += 2;
                }
                _ => {
                    fields.push(Field::flag(key.clone()));
                    cursor += 1;
                }
            },
            Some(Token::Equal) | None => {
                cursor += 1;
            }
        }
    }

    fields
}

#[cfg(test)]
mod tests {
    use super::{Field, Token, parse, parse_fields, tokenize};

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
            vec![
                ("debug".into(), String::new()),
                ("trace".into(), "true".into())
            ]
        );
    }

    #[test]
    fn parse_supports_quoted_values_with_spaces() {
        let fields = parse("level=info msg=\"hello world\"");

        assert_eq!(
            fields,
            vec![
                ("level".into(), "info".into()),
                ("msg".into(), "hello world".into())
            ]
        );
    }

    #[test]
    fn parse_unescapes_common_sequences_in_quoted_values() {
        let fields = parse("msg=\"line 1\\nline 2\" quote=\"say \\\"hi\\\"\"");

        assert_eq!(
            fields,
            vec![
                ("msg".into(), "line 1\nline 2".into()),
                ("quote".into(), "say \"hi\"".into())
            ]
        );
    }

    #[test]
    fn parse_tolerates_unterminated_quoted_values() {
        let fields = parse("msg=\"hello world");

        assert_eq!(fields, vec![("msg".into(), "hello world".into())]);
    }

    #[test]
    fn parse_fields_preserves_flags_and_values() {
        let fields = parse_fields("debug level=info empty=");

        assert_eq!(
            fields,
            vec![
                Field::flag("debug"),
                Field::pair("level", "info"),
                Field::pair("empty", "")
            ]
        );
    }

    #[test]
    fn field_knows_when_it_is_a_flag() {
        assert!(Field::flag("debug").is_flag());
        assert!(!Field::pair("level", "info").is_flag());
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

    #[test]
    fn tokenize_reads_quoted_values_as_single_atoms() {
        let tokens = tokenize("msg=\"hello world\" path=bare");

        assert_eq!(
            tokens,
            vec![
                Token::Atom("msg".into()),
                Token::Equal,
                Token::Atom("hello world".into()),
                Token::Atom("path".into()),
                Token::Equal,
                Token::Atom("bare".into()),
            ]
        );
    }

    #[test]
    fn tokenize_keeps_empty_quoted_values() {
        let tokens = tokenize("msg=\"\"");

        assert_eq!(
            tokens,
            vec![
                Token::Atom("msg".into()),
                Token::Equal,
                Token::Atom(String::new()),
            ]
        );
    }
}
