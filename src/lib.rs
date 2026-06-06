//! Parser playground prepared for fuzzing experiments.
//!
//! `rusty_me` implements a small, deliberately hand-rolled `logfmt` parser so
//! we can grow parsing rules alongside fuzzing coverage. The crate exposes
//! three complementary APIs:
//!
//! - a tuple-oriented [`parse`] helper for the loosest possible interpretation,
//! - structured [`Field`] / [`Record`] / [`Document`] types with `find`,
//!   iteration, and mutation helpers, and
//! - strict variants such as [`parse_strict`] that surface [`ParseError`]s at
//!   byte positions for fuzzing regressions.
//!
//! ```
//! use rusty_me::{Field, parse_record_strict};
//!
//! let record = parse_record_strict("debug level=info msg=\"hello world\"").unwrap();
//! assert_eq!(record.len(), 3);
//! assert_eq!(record.get_first_value("level"), Some("info"));
//! assert!(record.contains_flag("debug"));
//! # let _ = Field::flag("debug");
//! ```

pub mod logfmt;

pub use logfmt::{
    Document, Field, LineParseError, ParseError, ParseErrorKind, Record, Token, encode_fields,
    encode_lines, encode_map, normalize, normalize_document, normalize_document_strict,
    normalize_lines, normalize_lines_strict, normalize_strict, parse, parse_document,
    parse_document_strict, parse_fields, parse_lines, parse_lines_strict, parse_record,
    parse_record_strict, parse_strict, parse_to_map, parse_to_map_strict, tokenize,
};
