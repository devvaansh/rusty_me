#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(input) = std::str::from_utf8(data) {
        let tokens = rusty_me::tokenize(input);
        let tuples = rusty_me::parse(input);
        let fields = rusty_me::parse_fields(input);
        let pairs = rusty_me::parse_pairs(input);
        let flags = rusty_me::parse_flags(input);
        let record = rusty_me::parse_record(input);
        let records = rusty_me::parse_lines(input);
        let document = rusty_me::parse_document(input);
        let map = rusty_me::parse_to_map(input);
        let (lossy_records, lossy_errors) = rusty_me::parse_lines_lossy(input);
        let (lossy_document, lossy_doc_errors) = rusty_me::parse_document_lossy(input);
        let encoded_fields = rusty_me::encode_fields(&fields);
        let encoded_sorted = rusty_me::encode_sorted(&fields);
        let encoded_lines = rusty_me::encode_lines(&records);
        let encoded_map = rusty_me::encode_map(&map);
        let encoded_record = record.encode();
        let encoded_document = document.encode();
        let normalized = rusty_me::normalize(input);
        let normalized_document = rusty_me::normalize_document(input);
        let normalized_lines = rusty_me::normalize_lines(input);

        assert!(!tokens.is_empty() || input.trim().is_empty());
        assert_eq!(tuples.len(), fields.len());
        assert_eq!(record.fields(), fields.as_slice());
        assert_eq!(document.records().len(), records.len());
        assert_eq!(lossy_document.len(), lossy_records.len());
        assert_eq!(lossy_errors, lossy_doc_errors);

        // encode_sorted must parse back to the same field set (different order)
        let sorted_parsed = rusty_me::parse_fields(&encoded_sorted);
        assert_eq!(sorted_parsed.len(), fields.len());

        // flatten must yield all fields across all records
        let flat_count: usize = document.flatten().count();
        assert_eq!(
            flat_count,
            document.records().iter().map(|r| r.len()).sum::<usize>()
        );

        let expected_pairs: Vec<(String, String)> = fields
            .iter()
            .filter_map(|field| {
                field
                    .value
                    .as_ref()
                    .map(|value| (field.key.clone(), value.clone()))
            })
            .collect();
        assert_eq!(pairs, expected_pairs);

        let expected_flags: Vec<String> = fields
            .iter()
            .filter(|field| field.is_flag())
            .map(|field| field.key.clone())
            .collect();
        assert_eq!(flags, expected_flags);

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
        assert_eq!(rusty_me::parse_record(&encoded_record), record);
        assert_eq!(rusty_me::parse_lines(&encoded_lines), records);
        assert_eq!(rusty_me::parse_document(&encoded_document), document);
        assert_eq!(rusty_me::parse_to_map(&encoded_map), map);
        assert_eq!(rusty_me::parse_fields(&normalized), fields);
        assert_eq!(rusty_me::parse_document(&normalized_document), document);
        assert_eq!(rusty_me::parse_lines(&normalized_lines), records);

        if let Ok(strict_fields) = rusty_me::parse_strict(input) {
            assert_eq!(strict_fields, fields);
            assert_eq!(rusty_me::parse_record_strict(input).unwrap(), record);
            assert_eq!(rusty_me::parse_to_map_strict(input).unwrap(), map);
            assert_eq!(rusty_me::normalize_strict(input).unwrap(), encoded_fields);
        }

        if let Ok(strict_records) = rusty_me::parse_lines_strict(input) {
            assert_eq!(strict_records, records);
            assert_eq!(rusty_me::parse_document_strict(input).unwrap(), document);
            assert_eq!(
                rusty_me::normalize_document_strict(input).unwrap(),
                encoded_document
            );
            assert_eq!(
                rusty_me::normalize_lines_strict(input).unwrap(),
                encoded_lines
            );
            assert!(lossy_errors.is_empty());
            assert_eq!(lossy_records, strict_records);
        }
    }
});
