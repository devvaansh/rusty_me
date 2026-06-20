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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Record {
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Document {
    pub records: Vec<Record>,
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

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn without_value(mut self) -> Self {
        self.value = None;
        self
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

impl Record {
    pub fn new(fields: Vec<Field>) -> Self {
        Self { fields }
    }

    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    pub fn into_fields(self) -> Vec<Field> {
        self.fields
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Field> {
        self.fields.iter()
    }

    pub fn iter_pairs(&self) -> impl Iterator<Item = (&str, &str)> {
        self.fields.iter().filter_map(|field| {
            field
                .value
                .as_deref()
                .map(|value| (field.key.as_str(), value))
        })
    }

    pub fn iter_flags(&self) -> impl Iterator<Item = &str> {
        self.fields
            .iter()
            .filter(|field| field.is_flag())
            .map(|field| field.key.as_str())
    }

    pub fn find(&self, key: &str) -> Option<&Field> {
        self.fields.iter().find(|field| field.key == key)
    }

    pub fn find_last(&self, key: &str) -> Option<&Field> {
        self.fields.iter().rev().find(|field| field.key == key)
    }

    pub fn contains_flag(&self, key: &str) -> bool {
        self.fields
            .iter()
            .any(|field| field.key == key && field.is_flag())
    }

    pub fn get_first_value(&self, key: &str) -> Option<&str> {
        self.find(key).and_then(|field| field.value.as_deref())
    }

    pub fn get_last_value(&self, key: &str) -> Option<&str> {
        self.find_last(key).and_then(|field| field.value.as_deref())
    }

    pub fn values_for(&self, key: &str) -> Vec<&str> {
        self.fields
            .iter()
            .filter(|field| field.key == key)
            .filter_map(|field| field.value.as_deref())
            .collect()
    }

    pub fn retain<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&Field) -> bool,
    {
        self.fields.retain(|field| predicate(field));
    }

    pub fn remove_key(&mut self, key: &str) -> usize {
        let before = self.fields.len();
        self.fields.retain(|field| field.key != key);
        before - self.fields.len()
    }

    pub fn push(&mut self, field: Field) {
        self.fields.push(field);
    }

    pub fn extend<I>(&mut self, fields: I)
    where
        I: IntoIterator<Item = Field>,
    {
        self.fields.extend(fields);
    }

    pub fn to_map(&self) -> std::collections::BTreeMap<String, Option<String>> {
        self.fields
            .iter()
            .fold(std::collections::BTreeMap::new(), |mut map, field| {
                map.insert(field.key.clone(), field.value.clone());
                map
            })
    }

    #[must_use]
    pub fn encode(&self) -> String {
        encode_fields(&self.fields)
    }
}

impl Document {
    pub fn new(records: Vec<Record>) -> Self {
        Self { records }
    }

    pub fn records(&self) -> &[Record] {
        &self.records
    }

    pub fn into_records(self) -> Vec<Record> {
        self.records
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Record> {
        self.records.iter()
    }

    pub fn find_first(&self, key: &str) -> Option<&Field> {
        self.records.iter().find_map(|record| record.find(key))
    }

    pub fn find_last(&self, key: &str) -> Option<&Field> {
        self.records
            .iter()
            .rev()
            .find_map(|record| record.find_last(key))
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.records.iter().any(|record| record.find(key).is_some())
    }

    pub fn push(&mut self, record: Record) {
        self.records.push(record);
    }

    pub fn extend<I>(&mut self, records: I)
    where
        I: IntoIterator<Item = Record>,
    {
        self.records.extend(records);
    }

    pub fn retain<F>(&mut self, predicate: F)
    where
        F: FnMut(&Record) -> bool,
    {
        self.records.retain(predicate);
    }

    pub fn drop_empty_records(&mut self) -> usize {
        let before = self.records.len();
        self.records.retain(|record| !record.is_empty());
        before - self.records.len()
    }

    #[must_use]
    pub fn encode(&self) -> String {
        encode_lines(
            &self
                .records
                .iter()
                .map(|record| record.fields.clone())
                .collect::<Vec<_>>(),
        )
    }
}

impl From<Vec<Field>> for Record {
    fn from(fields: Vec<Field>) -> Self {
        Self::new(fields)
    }
}

impl From<Record> for Vec<Field> {
    fn from(record: Record) -> Self {
        record.fields
    }
}

impl<K, V> From<(K, V)> for Field
where
    K: Into<String>,
    V: Into<String>,
{
    fn from((key, value): (K, V)) -> Self {
        Field::pair(key, value)
    }
}

impl<K> From<(K,)> for Field
where
    K: Into<String>,
{
    fn from((key,): (K,)) -> Self {
        Field::flag(key)
    }
}

impl<'a> IntoIterator for &'a Record {
    type Item = &'a Field;
    type IntoIter = std::slice::Iter<'a, Field>;

    fn into_iter(self) -> Self::IntoIter {
        self.fields.iter()
    }
}

impl IntoIterator for Record {
    type Item = Field;
    type IntoIter = std::vec::IntoIter<Field>;

    fn into_iter(self) -> Self::IntoIter {
        self.fields.into_iter()
    }
}

impl FromIterator<Field> for Record {
    fn from_iter<I: IntoIterator<Item = Field>>(iter: I) -> Self {
        Record {
            fields: iter.into_iter().collect(),
        }
    }
}

impl Extend<Field> for Record {
    fn extend<I: IntoIterator<Item = Field>>(&mut self, iter: I) {
        self.fields.extend(iter);
    }
}

impl FromIterator<Record> for Document {
    fn from_iter<I: IntoIterator<Item = Record>>(iter: I) -> Self {
        Document {
            records: iter.into_iter().collect(),
        }
    }
}

impl Extend<Record> for Document {
    fn extend<I: IntoIterator<Item = Record>>(&mut self, iter: I) {
        self.records.extend(iter);
    }
}

impl From<Vec<Record>> for Document {
    fn from(records: Vec<Record>) -> Self {
        Self::new(records)
    }
}

impl From<Document> for Vec<Record> {
    fn from(document: Document) -> Self {
        document.records
    }
}

impl<'a> IntoIterator for &'a Document {
    type Item = &'a Record;
    type IntoIter = std::slice::Iter<'a, Record>;

    fn into_iter(self) -> Self::IntoIter {
        self.records.iter()
    }
}

impl IntoIterator for Document {
    type Item = Record;
    type IntoIter = std::vec::IntoIter<Record>;

    fn into_iter(self) -> Self::IntoIter {
        self.records.into_iter()
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

impl std::fmt::Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.encode())
    }
}

impl std::fmt::Display for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.encode())
    }
}

impl std::fmt::Display for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.encode())
    }
}

impl std::str::FromStr for Record {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        parse_record_strict(input)
    }
}

impl std::str::FromStr for Document {
    type Err = LineParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        parse_document_strict(input)
    }
}

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

/// Parses a logfmt input string into a typed record.
pub fn parse_record(input: &str) -> Record {
    Record::from(parse_fields(input))
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

/// Parses a logfmt input string into a typed record and returns a structured error for malformed input.
pub fn parse_record_strict(input: &str) -> Result<Record, ParseError> {
    parse_strict(input).map(Record::from)
}

/// Parses newline-delimited logfmt records into structured fields.
pub fn parse_lines(input: &str) -> Vec<Vec<Field>> {
    input
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(parse_fields)
        .collect()
}

/// Parses newline-delimited logfmt records into a typed document.
pub fn parse_document(input: &str) -> Document {
    Document::from(
        parse_lines(input)
            .into_iter()
            .map(Record::from)
            .collect::<Vec<_>>(),
    )
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

/// Parses newline-delimited logfmt records and separates well-formed lines from errors.
///
/// Unlike [`parse_lines_strict`], this variant continues past malformed lines and
/// returns any errors alongside successfully parsed records so callers can decide
/// how to react.
pub fn parse_lines_lossy(input: &str) -> (Vec<Vec<Field>>, Vec<LineParseError>) {
    let mut records = Vec::new();
    let mut errors = Vec::new();

    for (index, line) in input.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        match parse_strict(line) {
            Ok(fields) => records.push(fields),
            Err(error) => errors.push(LineParseError {
                line: index + 1,
                error,
            }),
        }
    }

    (records, errors)
}

/// Parses newline-delimited logfmt records into a typed document and reports the failing line on malformed input.
pub fn parse_document_strict(input: &str) -> Result<Document, LineParseError> {
    parse_lines_strict(input)
        .map(|records| Document::from(records.into_iter().map(Record::from).collect::<Vec<_>>()))
}

/// Parses a logfmt input string into a last-write-wins map.
pub fn parse_to_map(input: &str) -> std::collections::BTreeMap<String, Option<String>> {
    fields_to_map(parse_fields(input))
}

/// Extracts only the key/value pairs from a logfmt input, discarding bare flags.
#[must_use]
pub fn parse_pairs(input: &str) -> Vec<(String, String)> {
    parse_fields(input)
        .into_iter()
        .filter_map(|field| field.value.map(|value| (field.key, value)))
        .collect()
}

/// Extracts only the bare flags from a logfmt input, discarding key/value pairs.
#[must_use]
pub fn parse_flags(input: &str) -> Vec<String> {
    parse_fields(input)
        .into_iter()
        .filter(|field| field.is_flag())
        .map(|field| field.key)
        .collect()
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

/// Encodes multiple structured records into newline-delimited logfmt text.
#[must_use]
pub fn encode_lines(records: &[Vec<Field>]) -> String {
    let mut out = String::new();

    for (index, record) in records.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }

        out.push_str(&encode_fields(record));
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

/// Parses and re-encodes newline-delimited logfmt input into normalized records.
#[must_use]
pub fn normalize_lines(input: &str) -> String {
    encode_lines(&parse_lines(input))
}

/// Parses and re-encodes newline-delimited logfmt input, returning the failing line as an error.
pub fn normalize_lines_strict(input: &str) -> Result<String, LineParseError> {
    parse_lines_strict(input).map(|records| encode_lines(&records))
}

/// Parses and re-encodes newline-delimited logfmt input into a typed document.
#[must_use]
pub fn normalize_document(input: &str) -> String {
    parse_document(input).encode()
}

/// Parses and re-encodes newline-delimited logfmt input from a typed document, returning the failing line as an error.
pub fn normalize_document_strict(input: &str) -> Result<String, LineParseError> {
    parse_document_strict(input).map(|document| document.encode())
}

/// Escapes a value string using logfmt quoting rules and returns the encoded form.
///
/// Values that already survive unquoted are returned unchanged; values with
/// whitespace, quotes, or control characters get wrapped in double quotes.
#[must_use]
pub fn escape_value(value: &str) -> String {
    let mut out = String::new();
    push_encoded_value(&mut out, value);
    out
}

/// Reverses [`escape_value`] by consuming a possibly-quoted logfmt value.
///
/// Unquoted values are returned as-is. Quoted values have their surrounding
/// quotes removed and their escape sequences interpreted. Unterminated quotes
/// surface as [`ParseErrorKind::UnterminatedQuote`].
pub fn unescape_value(input: &str) -> Result<String, ParseError> {
    if !input.starts_with('"') {
        return Ok(input.to_string());
    }

    let mut cursor = input.char_indices().peekable();
    let (quote_position, _) = cursor.next().expect("leading quote consumed");
    let value = read_quoted_value_strict(&mut cursor, quote_position)?;

    if cursor.next().is_some() {
        return Err(ParseError {
            position: quote_position,
            kind: ParseErrorKind::UnexpectedQuote,
        });
    }

    Ok(value)
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
        Document, Field, LineParseError, ParseError, ParseErrorKind, Record, Token, encode_fields,
        encode_lines, encode_map, escape_value, normalize, normalize_document,
        normalize_document_strict, normalize_lines, normalize_lines_strict, normalize_strict,
        parse, parse_document, parse_document_strict, parse_fields, parse_flags, parse_lines,
        parse_lines_lossy, parse_lines_strict, parse_pairs, parse_record, parse_record_strict,
        parse_strict, parse_to_map, parse_to_map_strict, tokenize, unescape_value,
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
    fn parse_record_wraps_fields() {
        let record = parse_record("debug level=info empty=");

        assert_eq!(
            record,
            Record::new(vec![
                Field::flag("debug"),
                Field::pair("level", "info"),
                Field::pair("empty", "")
            ])
        );
    }

    #[test]
    fn parse_record_strict_preserves_strict_errors() {
        let error = parse_record_strict("=broken").unwrap_err();

        assert_eq!(
            error,
            ParseError {
                position: 0,
                kind: ParseErrorKind::MissingKey,
            }
        );
    }

    #[test]
    fn field_knows_when_it_is_a_flag() {
        assert!(Field::flag("debug").is_flag());
        assert!(!Field::pair("level", "info").is_flag());
    }

    #[test]
    fn field_builders_toggle_value_state() {
        let flag = Field::flag("debug");
        let promoted = flag.clone().with_value("on");
        assert_eq!(promoted, Field::pair("debug", "on"));

        let demoted = promoted.without_value();
        assert_eq!(demoted, Field::flag("debug"));
    }

    #[test]
    fn field_from_tuples_produces_pairs_and_flags() {
        let pair: Field = ("level", "info").into();
        assert_eq!(pair, Field::pair("level", "info"));

        let flag: Field = ("debug",).into();
        assert_eq!(flag, Field::flag("debug"));
    }

    #[test]
    fn record_and_document_collect_from_iterators() {
        let record: Record = vec![Field::flag("debug"), Field::pair("level", "info")]
            .into_iter()
            .collect();
        assert_eq!(record.len(), 2);
        assert!(record.contains_flag("debug"));

        let document: Document = vec![record.clone(), Record::default()]
            .into_iter()
            .collect();
        assert_eq!(document.len(), 2);
        assert_eq!(document.iter().next(), Some(&record));
    }

    #[test]
    fn display_impls_match_encode_output() {
        let field = Field::pair("msg", "hello world");
        assert_eq!(format!("{field}"), field.encode());

        let record = Record::new(vec![Field::flag("debug"), Field::pair("level", "info")]);
        assert_eq!(format!("{record}"), record.encode());

        let document = Document::new(vec![
            record.clone(),
            Record::new(vec![Field::pair("msg", "hi")]),
        ]);
        assert_eq!(format!("{document}"), document.encode());
    }

    #[test]
    fn from_str_roundtrips_display_output() {
        use std::str::FromStr;

        let record = Record::new(vec![Field::flag("debug"), Field::pair("msg", "hi world")]);
        assert_eq!(Record::from_str(&record.to_string()).unwrap(), record);

        let document = Document::new(vec![
            record.clone(),
            Record::new(vec![Field::pair("level", "info")]),
        ]);
        assert_eq!(Document::from_str(&document.to_string()).unwrap(), document);

        assert!(Record::from_str("=broken").is_err());
        assert!(Document::from_str("level=info\n=broken").is_err());
    }

    #[test]
    fn escape_value_quotes_special_characters_and_leaves_plain_text_untouched() {
        assert_eq!(escape_value("info"), "info");
        assert_eq!(escape_value(""), "\"\"");
        assert_eq!(escape_value("hello world"), "\"hello world\"");
        assert_eq!(escape_value("say \"hi\""), "\"say \\\"hi\\\"\"");
        assert_eq!(escape_value("a\tb"), "\"a\\tb\"");
    }

    #[test]
    fn unescape_value_roundtrips_escape_value() {
        for raw in ["info", "", "hello world", "say \"hi\"", "line 1\nline 2"] {
            assert_eq!(unescape_value(&escape_value(raw)).unwrap(), raw);
        }
    }

    #[test]
    fn unescape_value_rejects_trailing_bytes_after_quoted_value() {
        let error = unescape_value("\"quoted\"extra").unwrap_err();

        assert_eq!(
            error,
            ParseError {
                position: 0,
                kind: ParseErrorKind::UnexpectedQuote,
            }
        );
    }

    #[test]
    fn unescape_value_flags_unterminated_quotes() {
        let error = unescape_value("\"broken").unwrap_err();

        assert_eq!(
            error,
            ParseError {
                position: 0,
                kind: ParseErrorKind::UnterminatedQuote,
            }
        );
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
    fn parse_pairs_returns_only_key_value_pairs() {
        let pairs = parse_pairs("debug level=info msg=\"hello world\" trace");

        assert_eq!(
            pairs,
            vec![
                ("level".to_string(), "info".to_string()),
                ("msg".to_string(), "hello world".to_string()),
            ]
        );
    }

    #[test]
    fn parse_flags_returns_only_bare_flags_in_order() {
        let flags = parse_flags("debug level=info trace msg=hi verbose");

        assert_eq!(flags, vec!["debug", "trace", "verbose"]);
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
    fn parse_lines_lossy_keeps_good_records_and_lists_errors() {
        let (records, errors) = parse_lines_lossy("level=info\n=broken\nmsg=hello\n\"orphan\"");

        assert_eq!(records.len(), 2);
        assert_eq!(records[0], vec![Field::pair("level", "info")]);
        assert_eq!(records[1], vec![Field::pair("msg", "hello")]);

        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].line, 2);
        assert_eq!(errors[0].error.kind, ParseErrorKind::MissingKey);
        assert_eq!(errors[1].line, 4);
        assert_eq!(errors[1].error.kind, ParseErrorKind::UnexpectedQuote);
    }

    #[test]
    fn parse_document_wraps_line_records() {
        let document = parse_document("level=info\n\ndebug msg=hello");

        assert_eq!(
            document,
            Document::new(vec![
                Record::new(vec![Field::pair("level", "info")]),
                Record::new(vec![Field::flag("debug"), Field::pair("msg", "hello")]),
            ])
        );
    }

    #[test]
    fn parse_document_strict_reports_line_errors() {
        let error = parse_document_strict("level=info\n=broken").unwrap_err();

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
    fn record_helpers_expose_lookup_and_map_views() {
        let record = Record::new(vec![
            Field::flag("debug"),
            Field::pair("level", "info"),
            Field::pair("level", "warn"),
        ]);

        assert!(!record.is_empty());
        assert!(record.contains_flag("debug"));
        assert_eq!(record.get_first_value("level"), Some("info"));
        assert_eq!(record.get_last_value("level"), Some("warn"));
        assert_eq!(record.find("debug"), Some(&Field::flag("debug")));
        assert_eq!(
            record.to_map().get("level"),
            Some(&Some(String::from("warn")))
        );
    }

    #[test]
    fn record_values_for_returns_every_matching_value_in_order() {
        let record = Record::new(vec![
            Field::pair("level", "info"),
            Field::flag("level"),
            Field::pair("level", "warn"),
            Field::pair("msg", "hello"),
        ]);

        assert_eq!(record.values_for("level"), vec!["info", "warn"]);
        assert_eq!(record.values_for("msg"), vec!["hello"]);
        assert!(record.values_for("missing").is_empty());
    }

    #[test]
    fn record_iter_pairs_and_iter_flags_split_by_shape() {
        let record = Record::new(vec![
            Field::flag("debug"),
            Field::pair("level", "info"),
            Field::flag("trace"),
            Field::pair("msg", "hello"),
        ]);

        let pairs: Vec<(&str, &str)> = record.iter_pairs().collect();
        assert_eq!(pairs, vec![("level", "info"), ("msg", "hello")]);

        let flags: Vec<&str> = record.iter_flags().collect();
        assert_eq!(flags, vec!["debug", "trace"]);
    }

    #[test]
    fn record_retain_drops_fields_that_fail_the_predicate() {
        let mut record = Record::new(vec![
            Field::flag("debug"),
            Field::pair("level", "info"),
            Field::pair("msg", "hello"),
        ]);

        record.retain(|field| field.value.is_some());

        assert_eq!(
            record,
            Record::new(vec![
                Field::pair("level", "info"),
                Field::pair("msg", "hello"),
            ])
        );
    }

    #[test]
    fn record_remove_key_returns_number_of_removed_fields() {
        let mut record = Record::new(vec![
            Field::pair("level", "info"),
            Field::pair("level", "warn"),
            Field::pair("msg", "hello"),
        ]);

        assert_eq!(record.remove_key("level"), 2);
        assert_eq!(record.remove_key("missing"), 0);
        assert_eq!(record, Record::new(vec![Field::pair("msg", "hello")]));
    }

    #[test]
    fn record_push_and_extend_append_new_fields() {
        let mut record = Record::default();

        record.push(Field::flag("debug"));
        record.extend(vec![Field::pair("level", "info"), Field::pair("msg", "hi")]);

        assert_eq!(
            record,
            Record::new(vec![
                Field::flag("debug"),
                Field::pair("level", "info"),
                Field::pair("msg", "hi"),
            ])
        );
    }

    #[test]
    fn record_iteration_reports_length_and_visits_each_field() {
        let record = Record::new(vec![
            Field::flag("debug"),
            Field::pair("level", "info"),
            Field::pair("msg", "hello"),
        ]);

        assert_eq!(record.len(), 3);
        assert_eq!(record.iter().count(), 3);

        let by_ref: Vec<&Field> = (&record).into_iter().collect();
        assert_eq!(by_ref.len(), 3);
        assert_eq!(by_ref[0], &Field::flag("debug"));

        let owned: Vec<Field> = record.into_iter().collect();
        assert_eq!(owned[2], Field::pair("msg", "hello"));
    }

    #[test]
    fn document_iteration_reports_length_and_visits_each_record() {
        let document = Document::new(vec![
            Record::new(vec![Field::pair("level", "info")]),
            Record::new(vec![Field::flag("debug"), Field::pair("msg", "hello")]),
        ]);

        assert_eq!(document.len(), 2);
        assert_eq!(document.iter().count(), 2);

        let by_ref: Vec<&Record> = (&document).into_iter().collect();
        assert_eq!(by_ref[0].len(), 1);
        assert_eq!(by_ref[1].len(), 2);

        let owned: Vec<Record> = document.into_iter().collect();
        assert_eq!(
            owned[1].find("msg").unwrap().value.as_deref(),
            Some("hello")
        );
    }

    #[test]
    fn document_find_helpers_walk_across_records() {
        let document = Document::new(vec![
            Record::new(vec![Field::pair("level", "info")]),
            Record::new(vec![Field::pair("level", "warn"), Field::flag("debug")]),
        ]);

        assert_eq!(
            document.find_first("level"),
            Some(&Field::pair("level", "info"))
        );
        assert_eq!(
            document.find_last("level"),
            Some(&Field::pair("level", "warn"))
        );
        assert!(document.contains_key("debug"));
        assert!(!document.contains_key("missing"));
    }

    #[test]
    fn document_mutation_helpers_grow_shrink_and_filter() {
        let mut document = Document::default();

        document.push(Record::new(vec![Field::pair("level", "info")]));
        document.extend(vec![
            Record::default(),
            Record::new(vec![Field::flag("debug")]),
        ]);

        assert_eq!(document.len(), 3);
        assert_eq!(document.drop_empty_records(), 1);
        assert_eq!(document.len(), 2);

        document.retain(|record| record.contains_flag("debug"));
        assert_eq!(document.len(), 1);
        assert!(document.iter().next().unwrap().contains_flag("debug"));
    }

    #[test]
    fn record_encode_roundtrips_through_strict_parser() {
        let record = Record::new(vec![
            Field::flag("debug"),
            Field::pair("msg", "hello world"),
            Field::pair("empty", ""),
        ]);

        assert_eq!(parse_record_strict(&record.encode()).unwrap(), record);
    }

    #[test]
    fn document_encode_roundtrips_through_strict_parser() {
        let document = Document::new(vec![
            Record::new(vec![Field::pair("level", "info")]),
            Record::new(vec![
                Field::flag("debug"),
                Field::pair("msg", "hello world"),
            ]),
        ]);

        assert_eq!(parse_document_strict(&document.encode()).unwrap(), document);
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
    fn encode_lines_joins_records_with_newlines() {
        let records = vec![
            vec![Field::pair("level", "info")],
            vec![Field::flag("debug"), Field::pair("msg", "hello world")],
        ];

        assert_eq!(
            encode_lines(&records),
            "level=info\ndebug msg=\"hello world\""
        );
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
    fn normalize_lines_reencodes_multiline_input() {
        let normalized = normalize_lines("level=info\n\ndebug empty=\nmsg=\"hello world\"");

        assert_eq!(
            normalized,
            "level=info\ndebug empty=\"\"\nmsg=\"hello world\""
        );
    }

    #[test]
    fn normalize_lines_strict_reports_line_errors() {
        let error = normalize_lines_strict("level=info\n=broken").unwrap_err();

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
    fn normalize_document_matches_line_normalization() {
        let input = "level=info\n\ndebug empty=\nmsg=\"hello world\"";

        assert_eq!(normalize_document(input), normalize_lines(input));
        assert_eq!(
            normalize_document_strict(input).unwrap(),
            normalize_lines_strict(input).unwrap()
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
