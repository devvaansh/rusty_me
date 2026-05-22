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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
    MissingKey,
    UnexpectedQuote,
    UnterminatedQuote,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub position: usize,
    pub kind: ParseErrorKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineParseError {
    pub line: usize,
    pub error: ParseError,
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

    #[must_use]
    pub fn encode(&self) -> String {
        let mut out = String::new();
        push_encoded_field(&mut out, self);
        out
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ParseErrorKind::MissingKey => {
                write!(f, "missing key before '=' at byte {}", self.position)
            }
            ParseErrorKind::UnexpectedQuote => {
                write!(f, "unexpected quoted token at byte {}", self.position)
            }
            ParseErrorKind::UnterminatedQuote => {
                write!(f, "unterminated quoted value at byte {}", self.position)
            }
        }
    }
}

impl std::error::Error for ParseError {}

impl std::fmt::Display for LineParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.error)
    }
}

impl std::error::Error for LineParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
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

fn read_until_delimiter_with_spans<I>(cursor: &mut std::iter::Peekable<I>) -> String
where
    I: Iterator<Item = (usize, char)>,
{
    let mut atom = String::new();

    while let Some((_, ch)) = cursor.peek() {
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

fn read_until_whitespace_with_spans<I>(cursor: &mut std::iter::Peekable<I>) -> String
where
    I: Iterator<Item = (usize, char)>,
{
    let mut value = String::new();

    while let Some((_, ch)) = cursor.peek() {
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

fn read_quoted_value_strict<I>(
    cursor: &mut std::iter::Peekable<I>,
    quote_position: usize,
) -> Result<String, ParseError>
where
    I: Iterator<Item = (usize, char)>,
{
    let mut value = String::new();

    while let Some((_, ch)) = cursor.next() {
        match ch {
            '"' => return Ok(value),
            '\\' => {
                if let Some((_, escaped)) = cursor.next() {
                    value.push(match escaped {
                        '"' => '"',
                        '\\' => '\\',
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        other => other,
                    });
                } else {
                    return Err(ParseError {
                        position: quote_position,
                        kind: ParseErrorKind::UnterminatedQuote,
                    });
                }
            }
            other => value.push(other),
        }
    }

    Err(ParseError {
        position: quote_position,
        kind: ParseErrorKind::UnterminatedQuote,
    })
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

/// Parses a logfmt input string and returns a structured error for malformed input.
pub fn parse_strict(input: &str) -> Result<Vec<Field>, ParseError> {
    let mut fields = Vec::new();
    let mut cursor = input.char_indices().peekable();

    while let Some((position, ch)) = cursor.peek().copied() {
        if ch.is_ascii_whitespace() {
            cursor.next();
            continue;
        }

        if ch == '=' {
            return Err(ParseError {
                position,
                kind: ParseErrorKind::MissingKey,
            });
        }

        if ch == '"' {
            return Err(ParseError {
                position,
                kind: ParseErrorKind::UnexpectedQuote,
            });
        }

        let key = read_until_delimiter_with_spans(&mut cursor);

        if matches!(cursor.peek(), Some((_, '='))) {
            let equal_position = cursor.next().map(|(pos, _)| pos).unwrap_or(position);

            if key.is_empty() {
                return Err(ParseError {
                    position: equal_position,
                    kind: ParseErrorKind::MissingKey,
                });
            }

            let value = match cursor.peek().copied() {
                Some((quote_position, '"')) => {
                    cursor.next();
                    read_quoted_value_strict(&mut cursor, quote_position)?
                }
                Some((_, next)) if !next.is_ascii_whitespace() => {
                    read_until_whitespace_with_spans(&mut cursor)
                }
                _ => String::new(),
            };

            fields.push(Field::pair(key, value));
        } else {
            fields.push(Field::flag(key));
        }
    }

    Ok(fields)
}

/// Parses newline-delimited logfmt records into structured fields.
pub fn parse_lines(input: &str) -> Vec<Vec<Field>> {
    input
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(parse_fields)
        .collect()
}

/// Parses newline-delimited logfmt records and reports the failing line on malformed input.
pub fn parse_lines_strict(input: &str) -> Result<Vec<Vec<Field>>, LineParseError> {
    input
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| {
            parse_strict(line).map_err(|error| LineParseError {
                line: index + 1,
                error,
            })
        })
        .collect()
}

/// Parses a logfmt input string into a last-write-wins map.
pub fn parse_to_map(input: &str) -> std::collections::BTreeMap<String, Option<String>> {
    fields_to_map(parse_fields(input))
}

/// Parses a logfmt input string into a last-write-wins map and surfaces malformed input.
pub fn parse_to_map_strict(
    input: &str,
) -> Result<std::collections::BTreeMap<String, Option<String>>, ParseError> {
    parse_strict(input).map(fields_to_map)
}

/// Encodes a sequence of structured fields into normalized logfmt text.
#[must_use]
pub fn encode_fields(fields: &[Field]) -> String {
    let mut out = String::new();

    for (index, field) in fields.iter().enumerate() {
        if index > 0 {
            out.push(' ');
        }

        push_encoded_field(&mut out, field);
    }

    out
}

/// Encodes a last-write-wins map into normalized logfmt text.
#[must_use]
pub fn encode_map(map: &std::collections::BTreeMap<String, Option<String>>) -> String {
    let mut out = String::new();

    for (index, (key, value)) in map.iter().enumerate() {
        if index > 0 {
            out.push(' ');
        }

        push_encoded_key(&mut out, key);

        if let Some(value) = value {
            out.push('=');
            push_encoded_value(&mut out, value);
        }
    }

    out
}

/// Parses and re-encodes logfmt input into a normalized representation.
#[must_use]
pub fn normalize(input: &str) -> String {
    encode_fields(&parse_fields(input))
}

/// Parses and re-encodes logfmt input, returning malformed input as an error.
pub fn normalize_strict(input: &str) -> Result<String, ParseError> {
    parse_strict(input).map(|fields| encode_fields(&fields))
}

fn fields_to_map<I>(fields: I) -> std::collections::BTreeMap<String, Option<String>>
where
    I: IntoIterator<Item = Field>,
{
    let mut map = std::collections::BTreeMap::new();

    for field in fields {
        map.insert(field.key, field.value);
    }

    map
}

fn push_encoded_field(out: &mut String, field: &Field) {
    push_encoded_key(out, &field.key);

    if let Some(value) = &field.value {
        out.push('=');
        push_encoded_value(out, value);
    }
}

fn push_encoded_key(out: &mut String, key: &str) {
    out.push_str(key);
}

fn push_encoded_value(out: &mut String, value: &str) {
    if value.is_empty() {
        out.push_str("\"\"");
        return;
    }

    if value_needs_quotes(value) {
        out.push('"');

        for ch in value.chars() {
            match ch {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                other => out.push(other),
            }
        }

        out.push('"');
    } else {
        out.push_str(value);
    }
}

fn value_needs_quotes(value: &str) -> bool {
    value
        .chars()
        .any(|ch| ch.is_ascii_whitespace() || matches!(ch, '"' | '\\' | '\n' | '\r' | '\t'))
}

#[cfg(test)]
mod tests {
    use super::{
        Field, LineParseError, ParseError, ParseErrorKind, Token, encode_fields, encode_map,
        normalize, normalize_strict, parse, parse_fields, parse_lines, parse_lines_strict,
        parse_strict, parse_to_map, parse_to_map_strict, tokenize,
    };

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
    fn parse_strict_accepts_valid_input() {
        let fields = parse_strict("debug level=info msg=\"hello world\"").unwrap();

        assert_eq!(
            fields,
            vec![
                Field::flag("debug"),
                Field::pair("level", "info"),
                Field::pair("msg", "hello world")
            ]
        );
    }

    #[test]
    fn parse_strict_rejects_missing_keys() {
        let error = parse_strict("=broken").unwrap_err();

        assert_eq!(
            error,
            ParseError {
                position: 0,
                kind: ParseErrorKind::MissingKey
            }
        );
    }

    #[test]
    fn parse_strict_rejects_unexpected_quoted_tokens() {
        let error = parse_strict("\"orphan\"").unwrap_err();

        assert_eq!(
            error,
            ParseError {
                position: 0,
                kind: ParseErrorKind::UnexpectedQuote
            }
        );
    }

    #[test]
    fn parse_strict_rejects_unterminated_quotes() {
        let error = parse_strict("msg=\"unterminated").unwrap_err();

        assert_eq!(
            error,
            ParseError {
                position: 4,
                kind: ParseErrorKind::UnterminatedQuote
            }
        );
    }

    #[test]
    fn parse_to_map_preserves_flags_and_uses_last_value() {
        let fields = parse_to_map("debug level=info level=warn");

        assert_eq!(fields.get("debug"), Some(&None));
        assert_eq!(fields.get("level"), Some(&Some("warn".into())));
    }

    #[test]
    fn parse_to_map_strict_propagates_parse_errors() {
        let error = parse_to_map_strict("=broken").unwrap_err();

        assert_eq!(
            error,
            ParseError {
                position: 0,
                kind: ParseErrorKind::MissingKey
            }
        );
    }

    #[test]
    fn parse_lines_splits_multiline_input() {
        let records = parse_lines("level=info\nmsg=hello world\ntrace=true");

        assert_eq!(records.len(), 3);
        assert_eq!(records[0], vec![Field::pair("level", "info")]);
        assert_eq!(
            records[1],
            vec![Field::pair("msg", "hello"), Field::flag("world")]
        );
        assert_eq!(records[2], vec![Field::pair("trace", "true")]);
    }

    #[test]
    fn parse_lines_skips_blank_records() {
        let records = parse_lines("level=info\n\n  \nmsg=hello");

        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec![Field::pair("level", "info")]);
        assert_eq!(records[1], vec![Field::pair("msg", "hello")]);
    }

    #[test]
    fn parse_lines_strict_reports_the_failing_line() {
        let error = parse_lines_strict("level=info\n=broken\nmsg=hello").unwrap_err();

        assert_eq!(
            error,
            LineParseError {
                line: 2,
                error: ParseError {
                    position: 0,
                    kind: ParseErrorKind::MissingKey,
                },
            }
        );
    }

    #[test]
    fn field_encode_emits_flags_and_pairs() {
        assert_eq!(Field::flag("debug").encode(), "debug");
        assert_eq!(Field::pair("level", "info").encode(), "level=info");
    }

    #[test]
    fn encode_fields_quotes_and_escapes_special_values() {
        let encoded = encode_fields(&[
            Field::flag("debug"),
            Field::pair("msg", "hello world"),
            Field::pair("quote", "say \"hi\""),
            Field::pair("empty", ""),
        ]);

        assert_eq!(
            encoded,
            "debug msg=\"hello world\" quote=\"say \\\"hi\\\"\" empty=\"\""
        );
    }

    #[test]
    fn encode_fields_roundtrips_through_parse_strict() {
        let fields = vec![
            Field::flag("debug"),
            Field::pair("level", "info"),
            Field::pair("msg", "hello world"),
            Field::pair("note", "line 1\nline 2"),
        ];
        let encoded = encode_fields(&fields);

        assert_eq!(parse_strict(&encoded).unwrap(), fields);
    }

    #[test]
    fn encode_map_uses_sorted_keys_and_quoted_values() {
        let map = std::collections::BTreeMap::from([
            (String::from("debug"), None),
            (String::from("level"), Some(String::from("warn"))),
            (String::from("msg"), Some(String::from("hello world"))),
        ]);

        assert_eq!(encode_map(&map), "debug level=warn msg=\"hello world\"");
    }

    #[test]
    fn normalize_reencodes_lenient_input() {
        let normalized = normalize("msg=\"hello world\" debug empty=");

        assert_eq!(normalized, "msg=\"hello world\" debug empty=\"\"");
    }

    #[test]
    fn normalize_strict_rejects_malformed_input() {
        let error = normalize_strict("=broken").unwrap_err();

        assert_eq!(
            error,
            ParseError {
                position: 0,
                kind: ParseErrorKind::MissingKey
            }
        );
    }

    #[test]
    fn normalize_strict_roundtrips_valid_input() {
        let normalized = normalize_strict("debug level=info msg=\"hello world\"").unwrap();

        assert_eq!(normalized, "debug level=info msg=\"hello world\"");
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
