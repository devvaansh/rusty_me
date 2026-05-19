#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(input) = std::str::from_utf8(data) {
        let tokens = rusty_me::tokenize(input);
        let tuples = rusty_me::parse(input);
        let fields = rusty_me::parse_fields(input);
        let map = rusty_me::parse_to_map(input);

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

        if let Ok(strict_fields) = rusty_me::parse_strict(input) {
            assert_eq!(strict_fields, fields);
            assert_eq!(rusty_me::parse_to_map_strict(input).unwrap(), map);
        }
    }
});
