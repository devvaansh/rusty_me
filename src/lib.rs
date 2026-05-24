//! Parser playground prepared for fuzzing experiments.

pub mod logfmt;

pub use logfmt::{
    Document, Field, LineParseError, ParseError, ParseErrorKind, Record, Token, encode_fields,
    encode_lines, encode_map, normalize, normalize_document, normalize_document_strict,
    normalize_lines, normalize_lines_strict, normalize_strict, parse, parse_document,
    parse_document_strict, parse_fields, parse_lines, parse_lines_strict, parse_record,
    parse_record_strict, parse_strict, parse_to_map, parse_to_map_strict, tokenize,
};
