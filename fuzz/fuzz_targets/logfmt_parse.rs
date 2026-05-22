#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(input) = std::str::from_utf8(data) {
        let tokens = rusty_me::tokenize(input);
        let tuples = rusty_me::parse(input);
        let fields = rusty_me::parse_fields(input);
        let records = rusty_me::parse_lines(input);
        let map = rusty_me::parse_to_map(input);
        let encoded_fields = rusty_me::encode_fields(&fields);
        let encoded_lines = rusty_me::encode_lines(&records);
        let encoded_map = rusty_me::encode_map(&map);
        let normalized = rusty_me::normalize(input);
        let normalized_lines = rusty_me::normalize_lines(input);

        assert!(!tokens.is_empty() || input.trim().is_empty());
        assert_eq!(tuples.len(), fields.len());

        let tuples_from_fields: Vec<_> = fields
            .iter()
            .map(|field| (field.key.clone(), field.value.clone().unwrap_or_default()))
            .collect();
        assert_eq!(tuples, tuples_from_fields);

        let expected_map = fields.iter().fold(
            std::collections::BTreeMap::<String, Option<String>>::new(),
            |mut acc, field| {
                acc.insert(field.key.clone(), field.value.clone());
                acc
            },
        );
        assert_eq!(map, expected_map);

        assert_eq!(rusty_me::parse_fields(&encoded_fields), fields);
        assert_eq!(rusty_me::parse_lines(&encoded_lines), records);
        assert_eq!(rusty_me::parse_to_map(&encoded_map), map);
        assert_eq!(rusty_me::parse_fields(&normalized), fields);
        assert_eq!(rusty_me::parse_lines(&normalized_lines), records);

        if let Ok(strict_fields) = rusty_me::parse_strict(input) {
            assert_eq!(strict_fields, fields);
            assert_eq!(rusty_me::parse_to_map_strict(input).unwrap(), map);
            assert_eq!(rusty_me::normalize_strict(input).unwrap(), encoded_fields);
        }

        if let Ok(strict_records) = rusty_me::parse_lines_strict(input) {
            assert_eq!(strict_records, records);
            assert_eq!(rusty_me::normalize_lines_strict(input).unwrap(), encoded_lines);
        }
    }
});
