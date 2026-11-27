use aetheris::game_data::usmap::Mapping;
use serde_json::json;

const NAMES: &[&str] = &["Base", "Row", "ID", "Name", "Active", "EKind", "Value"];

fn mapping(version: u8, names: &[&str], structs: &[Vec<u8>]) -> Vec<u8> {
    let mut payload = (names.len() as u32).to_le_bytes().to_vec();
    for name in names {
        if version >= 2 {
            payload.extend((name.len() as u16).to_le_bytes());
        } else {
            payload.push(name.len() as u8);
        }
        payload.extend(name.as_bytes());
    }
    payload.extend(1u32.to_le_bytes()); // one enum
    payload.extend(5u32.to_le_bytes());
    if version >= 3 {
        payload.extend(1u16.to_le_bytes());
    } else {
        payload.push(1);
    }
    if version >= 4 {
        payload.extend(u64::MAX.to_le_bytes());
    }
    payload.extend(6u32.to_le_bytes());
    payload.extend((structs.len() as u32).to_le_bytes());
    for structure in structs {
        payload.extend(structure);
    }
    let mut bytes = 0x30c4u16.to_le_bytes().to_vec();
    bytes.push(version);
    if version >= 1 {
        bytes.extend(0u32.to_le_bytes());
    }
    bytes.push(0); // uncompressed
    bytes.extend((payload.len() as u32).to_le_bytes());
    bytes.extend((payload.len() as u32).to_le_bytes());
    bytes.extend(payload);
    bytes
}

fn structure(
    name: u32,
    parent: u32,
    slots: u16,
    properties: &[(u16, u8, u32, Vec<u8>)],
) -> Vec<u8> {
    let mut bytes = name.to_le_bytes().to_vec();
    bytes.extend(parent.to_le_bytes());
    bytes.extend(slots.to_le_bytes());
    bytes.extend((properties.len() as u16).to_le_bytes());
    for (index, dimension, name, kind) in properties {
        bytes.extend(index.to_le_bytes());
        bytes.push(*dimension);
        bytes.extend(name.to_le_bytes());
        bytes.extend(kind);
    }
    bytes
}

fn fixture(version: u8) -> Vec<u8> {
    mapping(
        version,
        NAMES,
        &[
            structure(0, u32::MAX, 1, &[(0, 1, 2, vec![2])]),
            // Reverse input order to verify serialization follows indices.
            structure(1, 0, 2, &[(1, 1, 4, vec![1]), (0, 1, 3, vec![5])]),
        ],
    )
}

fn string(bytes: &mut Vec<u8>, value: &str) {
    bytes.extend((value.len() as i32 + 1).to_le_bytes());
    bytes.extend(value.as_bytes());
    bytes.push(0);
}

fn table(columns: usize, rows: usize, data: &[u8]) -> Vec<u8> {
    let mut bytes = 13i32.to_le_bytes().to_vec();
    string(
        &mut bytes,
        &(0..columns)
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(","),
    );
    bytes.extend((rows as i32).to_le_bytes());
    bytes.extend(data);
    bytes
}

#[test]
fn imports_all_supported_versions_and_reads_inherited_fields_without_claiming_compatibility() {
    let mut row = 42i32.to_le_bytes().to_vec();
    string(&mut row, "sample");
    row.push(1);
    for version in 0..=4 {
        let bytes = fixture(version);
        let mapping = Mapping::parse(&bytes).unwrap();
        let report = mapping.probe_table("rOw", &table(3, 1, &row)).unwrap();
        assert_eq!(
            report.layout.rows,
            vec![json!({"ID":42,"Name":"sample","Active":true})]
        );
        assert!(!report.layout.compatibility_verified);
        assert_eq!(report.mapping_version, version);
        assert_eq!(report.mapping_sha256.len(), 64);
        for end in 0..bytes.len() {
            assert!(
                Mapping::parse(&bytes[..end]).is_err(),
                "v{version}, byte {end}"
            );
        }
        let mut extra = bytes.clone();
        extra.push(0);
        assert!(Mapping::parse(&extra).is_err());
        for end in 0..row.len() {
            assert!(
                mapping
                    .probe_table("Row", &table(3, 1, &row[..end]))
                    .is_err()
            );
        }
        assert!(mapping.probe_table("Row", &table(2, 1, &row)).is_err());
        assert!(mapping.probe_table("Missing", &table(3, 1, &row)).is_err());
    }
}

#[test]
fn refuses_bad_ranges_encodings_and_recursive_or_unsupported_layouts() {
    for properties in [
        vec![(0, 0, 2, vec![2])],                     // empty fixed array
        vec![(1, 1, 2, vec![2])],                     // outside property count
        vec![(0, 1, 2, vec![2]), (0, 1, 3, vec![5])], // overlap
        vec![(0, 1, 9999, vec![2])],                  // invalid name
        vec![(0, 1, 2, vec![99])],                    // unknown encoding
    ] {
        let bytes = mapping(4, NAMES, &[structure(1, u32::MAX, 1, &properties)]);
        assert!(Mapping::parse(&bytes).is_err());
    }
    for kind in [vec![4], vec![28, 2]] {
        let bytes = mapping(4, NAMES, &[structure(1, u32::MAX, 1, &[(0, 1, 2, kind)])]);
        assert!(
            Mapping::parse(&bytes)
                .unwrap()
                .probe_table("Row", &table(1, 0, &[]))
                .is_err()
        );
    }
    let recursive = mapping(4, NAMES, &[structure(1, 1, 0, &[])]);
    assert!(
        Mapping::parse(&recursive)
            .unwrap()
            .probe_table("Row", &table(0, 0, &[]))
            .is_err()
    );
    let mut bad = fixture(4);
    bad[7] = 1; // compressed
    assert!(Mapping::parse(&bad).is_err());
    let mut bad = fixture(4);
    bad[2] = 5;
    assert!(Mapping::parse(&bad).is_err());
    let mut bad = fixture(4);
    bad[8..12].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(Mapping::parse(&bad).is_err());
}

#[test]
fn ambiguous_unrelated_structures_are_reported_and_never_selected() {
    let base = structure(0, u32::MAX, 1, &[(0, 1, 2, vec![2])]);
    let bytes = mapping(
        4,
        NAMES,
        &[
            base.clone(),
            base, // identical repetitions are safe
            structure(0, u32::MAX, 1, &[(0, 1, 2, vec![19])]),
            structure(1, u32::MAX, 1, &[(0, 1, 2, vec![2])]),
        ],
    );
    let mapping = Mapping::parse(&bytes).unwrap();
    let report = mapping
        .probe_table("Row", &table(1, 1, &42i32.to_le_bytes()))
        .unwrap();
    assert!(report.mapping_ambiguous_structures.contains("base"));
    assert!(mapping.probe_table("Base", &table(1, 0, &[])).is_err());
}

#[test]
fn fixed_arrays_retain_each_slot() {
    let bytes = mapping(
        4,
        NAMES,
        &[structure(1, u32::MAX, 2, &[(0, 2, 2, vec![0])])],
    );
    let report = Mapping::parse(&bytes)
        .unwrap()
        .probe_table("Row", &table(2, 1, &[7, 9]))
        .unwrap();
    assert_eq!(report.layout.rows, vec![json!({"ID[0]":7,"ID[1]":9})]);
}

#[test]
fn nested_structures_enum_arrays_and_maps_keep_source_values() {
    let mut structure_kind = vec![9];
    structure_kind.extend(0u32.to_le_bytes());
    let mut array_kind = vec![8, 26, 0];
    array_kind.extend(5u32.to_le_bytes());
    let bytes = mapping(
        4,
        NAMES,
        &[
            structure(0, u32::MAX, 1, &[(0, 1, 2, vec![19])]),
            structure(
                1,
                u32::MAX,
                3,
                &[
                    (0, 1, 2, structure_kind),
                    (1, 1, 6, array_kind),
                    (2, 1, 3, vec![24, 5, 1]),
                ],
            ),
        ],
    );
    let mut row = 42u32.to_le_bytes().to_vec();
    row.extend(2i32.to_le_bytes());
    string(&mut row, "EKind::Value");
    string(&mut row, "None");
    row.extend(0i32.to_le_bytes());
    row.extend(1i32.to_le_bytes());
    string(&mut row, "flag");
    row.push(1);
    let report = Mapping::parse(&bytes)
        .unwrap()
        .probe_table("Row", &table(4, 1, &row))
        .unwrap();
    assert_eq!(
        report.layout.rows,
        vec![json!({
            "ID":{"ID":42}, "Value":["EKind::Value","None"],
            "Name":{"Removed":[],"Entries":[{"Key":"flag","Value":true}]}
        })]
    );
}

#[test]
fn package_metadata_is_bounded_and_does_not_shift_the_payload() {
    let mut bytes = fixture(4);
    bytes[3..7].copy_from_slice(&1u32.to_le_bytes());
    let mut metadata = 100i32.to_le_bytes().to_vec();
    metadata.extend(200i32.to_le_bytes());
    metadata.extend(1u32.to_le_bytes());
    metadata.extend([0; 16]);
    metadata.extend(3i32.to_le_bytes());
    metadata.extend(42u32.to_le_bytes());
    bytes.splice(7..7, metadata);
    assert!(Mapping::parse(&bytes).is_ok());
    bytes[15..19].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(Mapping::parse(&bytes).is_err());
}

#[test]
fn probe_command_is_reproducible_and_cannot_overwrite_its_sources() {
    use std::{fs, path::PathBuf, process::Command};
    struct Temp(PathBuf);
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let mut random = [0; 16];
    getrandom::fill(&mut random).unwrap();
    let temp = Temp(std::env::temp_dir().join(format!("aetheris-usmap-{}", hex::encode(random))));
    fs::create_dir(&temp.0).unwrap();
    let mapping_path = temp.0.join("mapping.usmap");
    let payload_path = temp.0.join("table.bin");
    let output_path = temp.0.join("probe.json");
    let mapping_bytes = fixture(4);
    let payload_bytes = table(3, 0, &[]);
    fs::write(&mapping_path, &mapping_bytes).unwrap();
    fs::write(&payload_path, &payload_bytes).unwrap();
    let run = |output: &std::path::Path| {
        Command::new(env!("CARGO_BIN_EXE_aetheris-game-data"))
            .arg("probe-table")
            .arg(&mapping_path)
            .arg("Row")
            .arg(&payload_path)
            .arg(output)
            .output()
            .unwrap()
    };
    let first = run(&output_path);
    assert!(first.status.success());
    let bytes = fs::read(&output_path).unwrap();
    assert!(run(&output_path).status.success());
    assert_eq!(bytes, fs::read(&output_path).unwrap());
    for input in [&mapping_path, &payload_path] {
        assert!(!run(input).status.success());
    }
    assert_eq!(mapping_bytes, fs::read(mapping_path).unwrap());
    assert_eq!(payload_bytes, fs::read(payload_path).unwrap());
    let summary: serde_json::Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(summary["compatibility_verified"], false);
}

#[test]
fn item_rows_select_only_their_variant_and_count_all_header_columns() {
    let names = [
        "ItemTableItem",
        "ItemType",
        "ID",
        "Unused",
        "Unused2",
        "EItemType",
        "Value",
        "ItemTableCurrencyInfo",
        "ItemTableEquipmentInfo",
        "ItemTableUsableInfo",
        "ItemTableMiscInfo",
    ];
    let mut enum_kind = vec![26, 0];
    enum_kind.extend(5u32.to_le_bytes());
    let mut structs = vec![structure(0, u32::MAX, 1, &[(0, 1, 1, enum_kind)])];
    for index in 7..=10 {
        structs.push(structure(index, u32::MAX, 1, &[(0, 1, 2, vec![19])]));
    }
    let mapping = Mapping::parse(&mapping(4, &names, &structs)).unwrap();
    let mut data = Vec::new();
    for (index, case) in ["Currency", "Equip", "Usable", "Misc"].iter().enumerate() {
        string(&mut data, &format!("EItemType::{case}"));
        data.extend((index as u32).to_le_bytes());
    }
    let payload = table(5, 4, &data);
    let report = mapping.probe_table("ItemTableItem", &payload).unwrap();
    for (index, row) in report.layout.rows.iter().enumerate() {
        assert_eq!(row["$variant"]["ID"], index);
    }
    let mut unknown = Vec::new();
    string(&mut unknown, "EItemType::Future");
    assert!(
        mapping
            .probe_table("ItemTableItem", &table(5, 1, &unknown))
            .is_err()
    );
    let mut old = payload.clone();
    old[..4].copy_from_slice(&12i32.to_le_bytes());
    assert!(mapping.probe_table("ItemTableItem", &old).is_err());
    for end in 0..payload.len() {
        assert!(
            mapping
                .probe_table("ItemTableItem", &payload[..end])
                .is_err()
        );
    }
}

#[test]
fn map_structs_have_their_own_header_and_do_not_accept_table_or_encrypted_input() {
    let names = [
        "Base", "MapData", "MapId", "Name", "Active", "EKind", "Value",
    ];
    let bytes = mapping(
        4,
        &names,
        &[structure(1, u32::MAX, 1, &[(0, 1, 2, vec![2])])],
    );
    let m = Mapping::parse(&bytes).unwrap();
    let payload = [
        1u32.to_le_bytes().as_slice(),
        b"9",
        20i32.to_le_bytes().as_slice(),
    ]
    .concat();
    let probe = m.probe_map("MapData", &payload).unwrap();
    assert_eq!(probe.layout.rows, vec![json!({"MapId":20})]);
    assert!(!probe.layout.compatibility_verified);
    assert!(m.probe_table("MapData", &payload).is_err());
    assert!(m.probe_map("Other", &payload).is_err());
    assert!(
        m.probe_map(
            "MapData",
            &aetheris::game_data::protection::map_payload(&payload).unwrap()
        )
        .is_err()
    );
}
