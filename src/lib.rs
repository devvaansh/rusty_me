//! Parser playground prepared for fuzzing experiments.

pub mod logfmt;

pub use logfmt::{
    Field, ParseError, ParseErrorKind, Token, parse, parse_fields, parse_strict, parse_to_map,
    parse_to_map_strict, tokenize,
};
