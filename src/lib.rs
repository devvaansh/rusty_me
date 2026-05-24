//! Parser playground prepared for fuzzing experiments.

pub mod logfmt;

pub use logfmt::{
    Field, LineParseError, ParseError, ParseErrorKind, Record, Token, encode_fields, encode_lines,
    encode_map, normalize, normalize_lines, normalize_lines_strict, normalize_strict, parse,
    parse_fields, parse_lines, parse_lines_strict, parse_record, parse_record_strict, parse_strict,
    parse_to_map, parse_to_map_strict, tokenize,
};
