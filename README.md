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
