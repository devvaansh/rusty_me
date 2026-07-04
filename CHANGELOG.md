# Changelog

## [0.2.0] – 2026-07-04

### Added

- `Record::len`, `Record::iter`, `Record::iter_pairs`, `Record::iter_flags` – typed iteration helpers
- `Record::values_for` – collect every value for a repeated key
- `Record::push`, `Record::extend`, `Record::retain`, `Record::remove_key` – mutation API
- `Record::sorted`, `Record::dedup_first`, `Record::dedup_last` – ordering and deduplication
- `Record::well_formed` – filter fields by `key_is_valid`
- `Document::len`, `Document::iter` – typed iteration helpers
- `Document::find_first`, `Document::find_last`, `Document::contains_key` – lookup helpers
- `Document::push`, `Document::extend`, `Document::retain`, `Document::drop_empty_records` – mutation API
- `Document::merge` – collapse records into one with last-write-wins semantics
- `Document::flatten` – flat iterator over all fields
- `Field::with_value`, `Field::without_value` – builder helpers
- `Field::is_well_formed` – delegates to `key_is_valid`
- `From<(K, V)>` and `From<(K,)>` conversions for `Field`
- `FromIterator`, `Extend` for `Record` and `Document`
- `Display`, `FromStr` for `Field`, `Record`, and `Document`
- `parse_pairs`, `parse_flags` – focused parse helpers
- `parse_lines_lossy`, `parse_document_lossy` – recovery parsers that continue past errors
- `escape_value`, `unescape_value` – standalone value encoding helpers
- `encode_sorted` – encodes fields with keys in alphabetical order
- `key_is_valid` – public key validation predicate

## [0.1.0] – 2026-05-24

- Initial logfmt parser with tokenizer and strict/lenient modes
- `Field`, `Record`, `Document` typed wrappers
- `encode_fields`, `encode_lines`, `encode_map` serialization helpers
- `normalize`, `normalize_lines`, `normalize_document` and strict variants
- `cargo-fuzz` target with roundtrip and multiline coverage
