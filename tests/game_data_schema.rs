use aetheris::game_data::{ClientIdentity, schema::*};
use serde_json::json;

fn identity() -> ClientIdentity {
    ClientIdentity {
        version: "synthetic-1".into(),
        region: "test".into(),
    }
}

fn schema() -> Schema {
    Schema {
        id: "synthetic-v1".into(),
        compatible_clients: vec![identity()],
        table_version: 0,
        column_hashes: vec!["hash1".into(), "hash2".into()],
        fields: vec![
            Field {
                name: "ID".into(),
                kind: Kind::U32,
            },
            Field {
                name: "Unknown1".into(),
                kind: Kind::Bool,
            },
        ],
    }
}
fn payload() -> Vec<u8> {
    let mut bytes = 0i32.to_le_bytes().to_vec();
    bytes.extend(12i32.to_le_bytes());
    bytes.extend(b"hash1,hash2\0");
    // Correct length includes the terminator (11 bytes + zero = 12).
    bytes.extend(1i32.to_le_bytes());
    bytes.extend(42u32.to_le_bytes());
    bytes.push(1);
    bytes
}

#[test]
fn strict_decoder_rejects_incompatibility_truncation_extra_data_and_wrong_types() {
    let schema = schema();
    let bytes = payload();
    assert_eq!(
        schema.decode(&identity(), &bytes).unwrap(),
        vec![json!({"ID":42,"Unknown1":true})]
    );
    for length in 0..bytes.len() {
        assert!(
            schema.decode(&identity(), &bytes[..length]).is_err(),
            "length {length}"
        );
    }
    let mut extra = bytes.clone();
    extra.push(0);
    assert!(schema.decode(&identity(), &extra).is_err());
    let mut bad_bool = bytes.clone();
    *bad_bool.last_mut().unwrap() = 2;
    assert!(schema.decode(&identity(), &bad_bool).is_err());
    let mut client = identity();
    client.version = "synthetic-2".into();
    assert!(schema.decode(&client, &bytes).is_err());
    let mut bad_schema = schema.clone();
    bad_schema.column_hashes.reverse();
    assert!(bad_schema.decode(&identity(), &bytes).is_err());
}

fn string(bytes: &mut Vec<u8>, text: &str) {
    bytes.extend(((text.len() + 1) as i32).to_le_bytes());
    bytes.extend(text.as_bytes());
    bytes.push(0);
}
fn row(schema: &Schema, data: &[u8]) -> Vec<u8> {
    let mut bytes = 0i32.to_le_bytes().to_vec();
    string(&mut bytes, &schema.column_hashes.join(","));
    bytes.extend(1i32.to_le_bytes());
    bytes.extend(data);
    bytes
}
fn single(kind: Kind, columns: usize) -> Schema {
    Schema {
        id: "synthetic".into(),
        compatible_clients: vec![identity()],
        table_version: 0,
        column_hashes: (0..columns).map(|i| format!("h{i}")).collect(),
        fields: vec![Field {
            name: "Data".into(),
            kind,
        }],
    }
}

#[test]
fn unicode_name_xor_keeps_the_terminator_raw_and_rejects_embedded_nuls() {
    let schema = single(Kind::String { xor_utf16: true }, 1);
    let words: Vec<_> = "Épée 🗡".encode_utf16().collect();
    let mut data = (-(words.len() as i32 + 1)).to_le_bytes().to_vec();
    let mut encoded: Vec<_> = words.into_iter().flat_map(u16::to_le_bytes).collect();
    for (i, byte) in encoded.iter_mut().enumerate() {
        *byte ^= [0x25, 0, 0xa8, 0, 0x7e, 0, 0x91, 0][i % 8];
    }
    data.extend(encoded);
    data.extend([0, 0]);
    assert_eq!(
        schema.decode(&identity(), &row(&schema, &data)).unwrap()[0]["Data"],
        "Épée 🗡"
    );
    *data.last_mut().unwrap() = 1;
    assert!(schema.decode(&identity(), &row(&schema, &data)).is_err());
    let plain = single(Kind::String { xor_utf16: false }, 1);
    let mut data = Vec::new();
    string(&mut data, "nul\0value");
    assert!(plain.decode(&identity(), &row(&plain, &data)).is_err());
}

#[test]
fn maps_sets_and_text_keep_deleted_values_namespaces_and_unknown_fields() {
    let schema = single(
        Kind::Struct {
            fields: vec![
                Field {
                    name: "UnknownMap".into(),
                    kind: Kind::Map {
                        key: Box::new(Kind::U32),
                        value: Box::new(Kind::Bool),
                    },
                },
                Field {
                    name: "UnknownSet".into(),
                    kind: Kind::Set {
                        element: Box::new(Kind::I32),
                    },
                },
                Field {
                    name: "Label".into(),
                    kind: Kind::Text,
                },
            ],
        },
        4,
    );
    let mut data = Vec::new();
    for word in [1u32, 7, 1, 9] {
        data.extend(word.to_le_bytes());
    }
    data.push(1);
    for word in [1i32, -4, 2, 5, 6] {
        data.extend(word.to_le_bytes());
    }
    data.extend(0u32.to_le_bytes());
    data.push(0);
    for text in ["test_namespace", "key", "source"] {
        string(&mut data, text);
    }
    let decoded = schema.decode(&identity(), &row(&schema, &data)).unwrap();
    assert_eq!(
        decoded[0]["Data"],
        json!({"UnknownMap":{"Removed":[7],"Entries":[{"Key":9,"Value":true}]},"UnknownSet":{"Removed":[-4],"Values":[5,6]},"Label":{"Flags":0,"Namespace":"test_namespace","Key":"key","SourceString":"source"}})
    );
    let bytes = row(&schema, &data);
    for size in 0..bytes.len() {
        assert!(schema.decode(&identity(), &bytes[..size]).is_err());
    }
}

#[test]
fn tagged_struct_arrays_check_names_sizes_and_property_guids() {
    let schema = single(
        Kind::TaggedArray {
            field: "Data".into(),
            struct_name: "SyntheticRow".into(),
            fields: vec![Field {
                name: "ID".into(),
                kind: Kind::U32,
            }],
        },
        1,
    );
    let mut data = 1i32.to_le_bytes().to_vec();
    string(&mut data, "Data");
    string(&mut data, "StructProperty");
    let size_offset = data.len();
    data.extend(4i64.to_le_bytes());
    string(&mut data, "SyntheticRow");
    data.extend([0; 16]);
    data.push(1);
    data.extend([0; 16]);
    data.extend(7u32.to_le_bytes());
    assert_eq!(
        schema.decode(&identity(), &row(&schema, &data)).unwrap()[0]["Data"],
        json!([{"ID":7}])
    );
    data[size_offset..size_offset + 8].copy_from_slice(&3i64.to_le_bytes());
    assert!(schema.decode(&identity(), &row(&schema, &data)).is_err());
}

#[test]
fn excessive_counts_unknown_histories_and_leaf_count_mismatches_are_rejected() {
    let array = single(
        Kind::Array {
            element: Box::new(Kind::U32),
        },
        1,
    );
    assert!(
        array
            .decode(&identity(), &row(&array, &i32::MAX.to_le_bytes()))
            .is_err()
    );
    let text = single(Kind::Text, 1);
    let mut data = 0u32.to_le_bytes().to_vec();
    data.push(2);
    assert!(text.decode(&identity(), &row(&text, &data)).is_err());
    let mut schema = schema();
    schema.fields.pop();
    assert!(schema.decode(&identity(), &payload()).is_err());
}
