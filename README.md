# rusty_me

`rusty_me` starts as a small parser-oriented Rust library that is friendly to fuzzing.

## Why this target?

For initial fuzzing work, a `logfmt` parser is a good moderate target:

- it accepts untrusted text input
- it has escaping and quoting edge cases
- it is small enough to iterate on quickly
- it can later grow into structured parsing and normalization rules

## Initial roadmap

1. bootstrap the crate
2. add a minimal tokenizer
3. expose a parser entry point
4. wire `cargo-fuzz` scaffolding

## Fuzzing

The repository now includes a starter `cargo-fuzz` target for `logfmt::parse`.

Typical flow:

1. install `cargo-fuzz`
2. run `cargo fuzz run logfmt_parse`
3. grow the seed corpus under `fuzz/corpus/logfmt_parse`

## Examples

Two runnable examples are included:

```
# normalize stdin, printing warnings for malformed lines to stderr
cargo run --example normalize_logfmt < input.log

# merge all lines into a single sorted record
cargo run --example merge_logfmt < input.log
```

## Current parser coverage

The in-tree parser now handles a small but useful subset of `logfmt`:

- bare flags like `debug`
- simple pairs like `level=info`
- quoted values like `msg="hello world"`
- common escapes inside quoted values such as `\n`, `\t`, `\\`, and `\"`

## Library entry points

The crate root now re-exports the most useful parsing helpers:

- `tokenize` for low-level token streams
- `parse` for a tuple-oriented compatibility API
- `parse_fields` for structured fields with flag awareness
- `parse_pairs` and `parse_flags` when only one shape is needed
- `parse_record` and `parse_record_strict` for typed single-record APIs
- `parse_strict` when malformed input should return a typed error
- `parse_document` and `parse_document_strict` for typed multi-record APIs
- `parse_lines` and `parse_lines_strict` for newline-delimited records
- `parse_lines_lossy` and `parse_document_lossy` for partial recovery with per-line errors
- `parse_to_map` and `parse_to_map_strict` for last-write-wins map output
- `escape_value` / `unescape_value` for single-value encoding
- `encode_fields`, `encode_lines`, `encode_map`, `normalize`, `normalize_lines`, and strict variants for serialization

Typed wrappers are also available:

- `Field` supports builder-style `with_value` / `without_value`, tuple `From` conversions, and `Display`
- `Record` offers key lookup helpers like `find`, `values_for`, `get_first_value`, and `to_map`; iteration via `iter`, `iter_pairs`, and `iter_flags`; and mutation via `push`, `extend`, `retain`, and `remove_key`
- `Document` groups newline-delimited `Record` values, supports iteration and mutation, and can encode back to text
- `Record` and `Document` implement `Display`, `FromStr`, `FromIterator`, and `Extend`

Example:

```rust
use rusty_me::{Field, parse_record_strict, parse_to_map};

let record = parse_record_strict("debug level=info msg=\"hello world\"")?;
assert_eq!(record.fields()[0], Field::flag("debug"));
assert_eq!(record.get_first_value("level"), Some("info"));

let map = parse_to_map("level=info level=warn");
assert_eq!(map.get("level"), Some(&Some(String::from("warn"))));
# Ok::<(), rusty_me::ParseError>(())
```

Normalization example:

```rust
use rusty_me::normalize_document_strict;

assert_eq!(
    normalize_document_strict("empty=\ndebug msg=\"hello world\"").unwrap(),
    "empty=\"\"\ndebug msg=\"hello world\""
);
```
