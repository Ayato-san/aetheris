//! Synthetic protected files, with no client keys or copyrighted game payloads.
use aes::{
    Aes256, Block,
    cipher::{BlockEncrypt, KeyInit},
};
use aetheris::game_data::protection::*;

fn encrypt(bytes: &[u8], key: &[u8; 32]) -> Vec<u8> {
    let cipher = Aes256::new_from_slice(key).unwrap();
    bytes
        .as_chunks::<16>()
        .0
        .iter()
        .flat_map(|b| {
            let mut block = Block::from(*b);
            cipher.encrypt_block(&mut block);
            block.to_vec()
        })
        .collect()
}
fn manifest(seed: u64, key: &[u8; 32]) -> Vec<u8> {
    let mut plaintext = seed.to_le_bytes().to_vec();
    plaintext.extend(key);
    plaintext.extend([0; 8]);
    let mut bytes = 2i32.to_le_bytes().to_vec();
    bytes.extend(1i32.to_le_bytes());
    bytes.extend(48i32.to_le_bytes());
    bytes.extend(encrypt(&plaintext, blake3::hash(&[0; 32]).as_bytes()));
    bytes
}
fn compressed(localization: bool, raw: &[u8]) -> Vec<u8> {
    let packed = lz4_flex::block::compress(raw);
    let mut plaintext = vec![0; 16];
    plaintext.extend(&packed);
    plaintext.resize(plaintext.len().div_ceil(16) * 16, 0);
    let mut bytes = Vec::new();
    if !localization {
        bytes.extend(13i32.to_le_bytes());
    }
    bytes.extend(2i32.to_le_bytes());
    bytes.extend(7u64.to_le_bytes());
    for size in [raw.len(), packed.len(), plaintext.len()] {
        bytes.extend((size as i32).to_le_bytes());
    }
    bytes.extend(encrypt(&plaintext, &[0; 32]));
    bytes
}

#[test]
fn key_manifests_and_both_compressed_families_check_lengths_keys_and_truncation() {
    let source = manifest(7, &[0; 32]);
    let keys = KeyRing::from_manifest(&source, &[0; 32]).unwrap();
    let raw = b"synthetic table content repeated repeated repeated";
    for localization in [false, true] {
        let bytes = compressed(localization, raw);
        let decode = |bytes: &[u8]| {
            if localization {
                keys.localization(bytes)
            } else {
                keys.table(bytes)
            }
        };
        assert_eq!(decode(&bytes).unwrap(), raw);
        for end in 0..bytes.len() {
            assert!(decode(&bytes[..end]).is_err(), "length {end}");
        }
        let mut extra = bytes.clone();
        extra.push(0);
        assert!(decode(&extra).is_err());
        let missing = KeyRing::from_manifest(&manifest(8, &[0; 32]), &[0; 32]).unwrap();
        assert!(
            if localization {
                missing.localization(&bytes)
            } else {
                missing.table(&bytes)
            }
            .is_err()
        );
    }
    for length in 0..source.len() {
        assert!(KeyRing::from_manifest(&source[..length], &[0; 32]).is_err());
    }
}

#[test]
fn counter_stream_uses_the_little_endian_counter_in_the_final_eight_bytes() {
    let keys = KeyRing::from_manifest(&manifest(7, &[0; 32]), &[0; 32]).unwrap();
    // Independent AES-256-ECB vectors generated with OpenSSL from counters
    // [0;16] and [0;8] ++ [1,0,0,0,0,0,0,0], using the all-zero test key.
    let stream =
        hex::decode("dc95c078a2408989ad48a214928420874816efe3deb380566eba0c17bf582090").unwrap();
    let raw = b"a payload exceeding one AES block";
    let mut bytes = 13i32.to_le_bytes().to_vec();
    bytes.extend(3i32.to_le_bytes());
    bytes.extend(7u64.to_le_bytes());
    bytes.extend([0; 8]);
    let raw = &raw[..31];
    bytes.extend((raw.len() as i32).to_le_bytes());
    bytes.extend(raw.iter().zip(&stream).map(|(a, b)| a ^ b));
    assert_eq!(keys.table(&bytes).unwrap(), raw);
    let mut wrong = bytes.clone();
    wrong[4..8].copy_from_slice(&4i32.to_le_bytes());
    assert!(keys.table(&wrong).is_err());
    bytes.push(0);
    assert!(keys.table(&bytes).is_err());
}

#[test]
fn map_xor_is_a_distinct_branch_and_bad_lz4_is_rejected() {
    let raw = b"{\"synthetic\":true}";
    let encrypted = map_payload(raw).unwrap();
    assert_eq!(map_payload(&encrypted).unwrap(), raw);
    let keys = KeyRing::from_manifest(&manifest(7, &[0; 32]), &[0; 32]).unwrap();
    assert!(keys.table(&encrypted).is_err());
    let mut invalid = compressed(false, b"a");
    invalid[16..20].copy_from_slice(&100i32.to_le_bytes());
    assert!(keys.table(&invalid).is_err());
}

fn hash64(bytes: &[u8]) -> u64 {
    u64::from_le_bytes(blake3::hash(bytes).as_bytes()[..8].try_into().unwrap())
}
fn current_manifest(name: &str) -> Vec<u8> {
    let mut record = hash64(name.as_bytes()).to_le_bytes().to_vec();
    record.extend([0; 40]);
    let mut bytes = vec![0; 8];
    bytes.extend(encrypt(&record, blake3::hash(&[0; 32]).as_bytes()));
    bytes
}
fn current_header(seed: u64, kind: i32, fields: &[i32]) -> Vec<u8> {
    let root = hash64(&0xCD02190910CE83F6u64.to_le_bytes());
    let mut material = root.to_le_bytes().to_vec();
    material.extend(seed.to_le_bytes());
    material.extend(kind.to_le_bytes());
    material.extend([0; 4]);
    let mask = blake3::hash(&material);
    fields
        .iter()
        .flat_map(|n| n.to_le_bytes())
        .zip(mask.as_bytes())
        .map(|(a, b)| a ^ b)
        .collect()
}
#[test]
fn reference_profile_compressed_tables_and_localisations_validate_names_sizes_and_truncations() {
    let raw = b"synthetic reference-profile plaintext";
    let packed = lz4_flex::block::compress(raw);
    let mut plain = vec![0; 32];
    plain.extend(&packed);
    plain.resize(plain.len().next_multiple_of(16), 0);
    for (name, path, kind, marker) in [
        ("Synthetic", "Data/Table/Synthetic.dat", 1, 13i32),
        ("L10NString_en-US", "L10N/en-US/Text.dat", 3, 2),
    ] {
        let manifest = current_manifest(name);
        let keys = KeyRing::from_current_manifest(&manifest, &[0; 32]).unwrap();
        let mut bytes = marker.to_le_bytes().to_vec();
        bytes.extend(current_header(
            hash64(name.as_bytes()),
            kind,
            &[packed.len() as i32, 2, plain.len() as i32, raw.len() as i32],
        ));
        bytes.extend(encrypt(&plain, &[0; 32]));
        let decode = |b: &[u8]| {
            if kind == 1 {
                keys.current_table(b, path)
            } else {
                keys.current_localization(b, path)
            }
        };
        assert_eq!(decode(&bytes).unwrap(), raw);
        for end in 0..bytes.len() {
            assert!(decode(&bytes[..end]).is_err(), "{name}: {end}");
        }
        bytes.push(0);
        assert!(decode(&bytes).is_err());
        assert!(keys.current_table(&bytes, "Data/Table/Other.dat").is_err());
        for end in 0..manifest.len() {
            assert!(KeyRing::from_current_manifest(&manifest[..end], &[0; 32]).is_err());
        }
        let mut duplicate = manifest.clone();
        duplicate.extend(&manifest[8..]);
        assert!(KeyRing::from_current_manifest(&duplicate, &[0; 32]).is_ok());
        let mut record = hash64(name.as_bytes()).to_le_bytes().to_vec();
        record.extend([1; 32]);
        record.extend([0; 8]);
        duplicate.extend(encrypt(&record, blake3::hash(&[0; 32]).as_bytes()));
        assert!(KeyRing::from_current_manifest(&duplicate, &[0; 32]).is_err());
    }
}
#[test]
fn reference_profile_counter_stream_uses_key_nonce_and_preserves_partial_final_blocks() {
    let keys = KeyRing::from_current_manifest(&current_manifest("Synthetic"), &[0; 32]).unwrap();
    let raw = b"synthetic payload longer than one block";
    let mut bytes = 13i32.to_le_bytes().to_vec();
    bytes.extend(current_header(
        hash64(b"Synthetic"),
        2,
        &[raw.len() as i32, 3],
    ));
    let mut nonce_input = vec![0; 32];
    nonce_input.extend(b"nonc");
    let nonce = hash64(&nonce_input);
    let cipher = Aes256::new_from_slice(&[0; 32]).unwrap();
    for (index, chunk) in raw.chunks(16).enumerate() {
        let mut counter = nonce.to_le_bytes().to_vec();
        counter.extend((index as u64).to_le_bytes());
        let mut block = Block::clone_from_slice(&counter);
        cipher.encrypt_block(&mut block);
        bytes.extend(chunk.iter().zip(block).map(|(a, b)| a ^ b));
    }
    assert_eq!(keys.current_table(&bytes, "Synthetic.dat").unwrap(), raw);
    for end in 0..bytes.len() {
        assert!(keys.current_table(&bytes[..end], "Synthetic.dat").is_err());
    }
    assert!(keys.current_table(&bytes, "synthetic.dat").is_err());
}

#[test]
fn versioned_manifest_and_shuffled_l10n_decode_with_synthetic_material() {
    let version = "0.0.7";
    let context = blake3::hash(format!("AION2.TableKey.AppVersion{version}").as_bytes());
    let mut material = vec![0; 32];
    material.extend(context.as_bytes());
    let name = "L10NString_en-US";
    let seed = hash64(name.as_bytes());
    let mut records = seed.to_le_bytes().to_vec();
    records.extend([0; 40]);
    let mut manifest = vec![0; 8];
    manifest.extend(encrypt(&records, blake3::hash(&material).as_bytes()));
    assert!(KeyRing::from_current_manifest(&manifest, &[0; 32]).is_err());
    let keys = KeyRing::from_versioned_manifest(&manifest, &[0; 32]).unwrap();
    assert_eq!(keys.app_version(), Some(version));
    let mut root_input = 0xCD02190910CE83F6u64.to_le_bytes().to_vec();
    root_input.extend(context.as_bytes());
    let root = hash64(&root_input);
    let header_root = hash64(&root.to_le_bytes());
    let mut shuffle_input = root.to_le_bytes().to_vec();
    shuffle_input.extend(0x51i32.to_le_bytes());
    let shuffle = blake3::hash(&shuffle_input);
    let mut permutation = [0usize, 1, 2, 3];
    for i in (1..4).rev() {
        permutation.swap(i, shuffle.as_bytes()[3 - i] as usize % (i + 1));
    }
    // The payload must survive both decryption and the independent FString parser.
    let raw = hex::decode(
        "0100000005000000746573740001000000040000006b6579000600000076616c75650000000000",
    )
    .unwrap();
    let packed = lz4_flex::block::compress(&raw);
    let mut plain = vec![0; 32];
    plain.extend(&packed);
    plain.resize(plain.len().next_multiple_of(16), 0);
    let fields = [2, raw.len() as i32, packed.len() as i32, plain.len() as i32];
    let mut header_material = header_root.to_le_bytes().to_vec();
    header_material.extend(seed.to_le_bytes());
    header_material.extend(3i32.to_le_bytes());
    header_material.extend([0; 4]);
    let mask = blake3::hash(&header_material);
    let header: Vec<_> = permutation
        .into_iter()
        .flat_map(|i| fields[i].to_le_bytes())
        .zip(mask.as_bytes())
        .map(|(a, b)| a ^ b)
        .collect();
    let mut bytes = 2i32.to_le_bytes().to_vec();
    bytes.extend(&header);
    bytes.extend(encrypt(&plain, &[0; 32]));
    let path = "L10N/en-US/L10NString.dat";
    assert_eq!(keys.current_localization(&bytes, path).unwrap(), raw);
    assert_eq!(
        aetheris::game_data::assets::localization(&raw)
            .unwrap()
            .entries["key"],
        "value"
    );
    assert!(
        keys.current_localization(&bytes, "L10N/ja-JP/L10NString.dat")
            .is_err()
    );
    for end in 0..bytes.len() {
        assert!(keys.current_localization(&bytes[..end], path).is_err());
    }
    bytes.push(0);
    assert!(keys.current_localization(&bytes, path).is_err());
    for end in 0..manifest.len() {
        assert!(KeyRing::from_versioned_manifest(&manifest[..end], &[0; 32]).is_err());
    }
    // A correct first record cannot hide a corrupted reserved tail later on.
    let legacy = current_manifest(name);
    let mut invalid = legacy.clone();
    let mut wrong = records.clone();
    wrong[47] = 1;
    invalid.extend(encrypt(&wrong, blake3::hash(&[0; 32]).as_bytes()));
    assert!(KeyRing::from_current_manifest(&invalid, &[0; 32]).is_err());
    assert!(
        KeyRing::from_versioned_manifest(&legacy, &[0; 32])
            .unwrap()
            .app_version()
            .is_none()
    );
}

// Synthetic versioned containers built independently of the reader. The wire
// fields follow the logical type/raw/packed/aligned order from the reference.
fn versioned_table_fixture(
    version: &str,
    content_key: [u8; 32],
    raw: &[u8],
    stream: bool,
) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let seed = hash64(b"Synthetic");
    let context = blake3::hash(format!("AION2.TableKey.AppVersion{version}").as_bytes());
    let mut material = vec![0; 32];
    material.extend(context.as_bytes());
    let mut records = seed.to_le_bytes().to_vec();
    records.extend(content_key);
    records.extend([0; 8]);
    let mut manifest = vec![0; 8];
    manifest.extend(encrypt(&records, blake3::hash(&material).as_bytes()));
    let mut root_input = 0xCD02190910CE83F6u64.to_le_bytes().to_vec();
    root_input.extend(context.as_bytes());
    let root = hash64(&root_input);
    let mut mask_input = hash64(&root.to_le_bytes()).to_le_bytes().to_vec();
    mask_input.extend(seed.to_le_bytes());
    mask_input.extend(if stream { 2i32 } else { 1i32 }.to_le_bytes());
    mask_input.extend([0; 4]);
    let mask = blake3::hash(&mask_input);
    let (fields, payload, wrong_fields) = if stream {
        let mut nonce = content_key.to_vec();
        nonce.extend(b"nonc");
        let nonce = hash64(&nonce);
        let cipher = Aes256::new_from_slice(&content_key).unwrap();
        let mut payload = Vec::new();
        for (index, chunk) in raw.chunks(16).enumerate() {
            let mut counter = nonce.to_le_bytes().to_vec();
            counter.extend((index as u64).to_le_bytes());
            let mut block = Block::clone_from_slice(&counter);
            cipher.encrypt_block(&mut block);
            payload.extend(chunk.iter().zip(block).map(|(a, b)| a ^ b));
        }
        (
            vec![raw.len() as i32, 3],
            payload,
            vec![3, raw.len() as i32],
        )
    } else {
        let packed = lz4_flex::block::compress(raw);
        let mut plain = vec![0; 32];
        plain.extend(&packed);
        plain.resize(plain.len().next_multiple_of(16), 0);
        let logical = [2, raw.len() as i32, packed.len() as i32, plain.len() as i32];
        let mut shuffle_input = root.to_le_bytes().to_vec();
        shuffle_input.extend(0x51i32.to_le_bytes());
        let shuffle = blake3::hash(&shuffle_input);
        let mut order = [0, 1, 2, 3];
        for i in (1..4).rev() {
            order.swap(i, shuffle.as_bytes()[3 - i] as usize % (i + 1));
        }
        let fields: Vec<_> = order.iter().map(|&i| logical[i]).collect();
        let mut wrong = fields.clone();
        // Rotate the wire fields without changing the mask or payload.
        wrong.rotate_left(1);
        (fields, encrypt(&plain, &content_key), wrong)
    };
    let encode = |fields: Vec<i32>| {
        let mut bytes = 13i32.to_le_bytes().to_vec();
        bytes.extend(
            fields
                .into_iter()
                .flat_map(i32::to_le_bytes)
                .zip(mask.as_bytes())
                .map(|(a, b)| a ^ b),
        );
        bytes.extend(&payload);
        bytes
    };
    (manifest, encode(fields), encode(wrong_fields))
}

#[test]
fn versioned_tables_decode_shuffled_compression_and_short_and_long_streams() {
    for stream in [false, true] {
        for raw in [
            b"x".as_slice(),
            b"independent synthetic table spanning several AES blocks",
        ] {
            let (manifest, bytes, wrong) = versioned_table_fixture("0.0.7", [1; 32], raw, stream);
            let keys = KeyRing::from_versioned_manifest(&manifest, &[0; 32]).unwrap();
            assert_eq!(
                keys.current_table(&bytes, "Data/Table/Synthetic.dat")
                    .unwrap(),
                raw
            );
            assert!(keys.current_table(&wrong, "Synthetic.dat").is_err());
            assert!(keys.current_table(&bytes, "Renamed.dat").is_err());
            for end in 0..bytes.len() {
                assert!(
                    keys.current_table(&bytes[..end], "Synthetic.dat").is_err(),
                    "{stream}: {end}"
                );
            }
            let mut extra = bytes.clone();
            extra.push(0);
            assert!(keys.current_table(&extra, "Synthetic.dat").is_err());
            // The unversioned profile cannot silently interpret this variant.
            let legacy =
                KeyRing::from_current_manifest(&current_manifest("Synthetic"), &[0; 32]).unwrap();
            assert!(legacy.current_table(&bytes, "Synthetic.dat").is_err());
        }
    }
}

#[test]
fn manifest_contexts_and_content_keys_stay_isolated_across_sessions_and_threads() {
    let raw = b"synthetic payload with independently selected content keys";
    let (manifest_a, bytes_a, _) = versioned_table_fixture("0.0.7", [1; 32], raw, false);
    let (manifest_b, bytes_b, _) = versioned_table_fixture("0.0.8", [2; 32], raw, false);
    let a = KeyRing::from_versioned_manifest(&manifest_a, &[0; 32]).unwrap();
    let b = KeyRing::from_versioned_manifest(&manifest_b, &[0; 32]).unwrap();
    let check = |keys: &KeyRing, bytes: &[u8], other: &[u8]| {
        for _ in 0..8 {
            assert_eq!(keys.current_table(bytes, "Synthetic.dat").unwrap(), raw);
            assert!(keys.current_table(other, "Synthetic.dat").is_err());
        }
    };
    check(&a, &bytes_a, &bytes_b);
    check(&b, &bytes_b, &bytes_a);
    check(&a, &bytes_a, &bytes_b);
    let a_again = KeyRing::from_versioned_manifest(&manifest_a, &[0; 32]).unwrap();
    std::thread::scope(|scope| {
        scope.spawn(|| check(&a, &bytes_a, &bytes_b));
        scope.spawn(|| check(&b, &bytes_b, &bytes_a));
        scope.spawn(|| check(&a_again, &bytes_a, &bytes_b));
    });
}

#[test]
fn versioned_manifest_reports_missing_context_and_rejects_a_corrupt_later_record() {
    let (manifest, _, _) = versioned_table_fixture("0.0.7", [1; 32], b"synthetic", false);
    let error = KeyRing::from_versioned_manifest(&manifest, &[9; 32])
        .err()
        .unwrap();
    assert!(error.to_string().contains("manifest key not found"));
    // A valid first entry cannot conceal an invalid reserved tail in another.
    let context = blake3::hash(b"AION2.TableKey.AppVersion0.0.7");
    let mut material = vec![0; 32];
    material.extend(context.as_bytes());
    let mut record = hash64(b"Other").to_le_bytes().to_vec();
    record.extend([2; 32]);
    record.extend([0; 8]);
    record[47] = 1;
    let mut corrupt = manifest;
    corrupt.extend(encrypt(&record, blake3::hash(&material).as_bytes()));
    assert!(KeyRing::from_versioned_manifest(&corrupt, &[0; 32]).is_err());
}

#[test]
fn numeric_context_search_is_cancelled_between_candidates_and_stays_session_local() {
    use aetheris::game_data::reader::{Control, Phase};
    use std::sync::atomic::{AtomicBool, Ordering};
    let (manifest, _, _) = versioned_table_fixture("9.9.999", [2; 32], b"test", false);
    let cancel = AtomicBool::new(false);
    let mut last = 0;
    let mut progress = |p: aetheris::game_data::reader::Progress| {
        if p.phase == Phase::KeySearch {
            last = p.completed;
            if last >= 1000 {
                cancel.store(true, Ordering::Release);
            }
        }
    };
    let error = KeyRing::from_versioned_manifest_with_control(
        &manifest,
        &[0; 32],
        &mut Control::new(&cancel, &mut progress),
    )
    .err()
    .unwrap();
    assert_eq!(error.kind(), std::io::ErrorKind::Interrupted);
    assert_eq!(last, 1000);
    let (manifest, table, _) =
        versioned_table_fixture("0.0.7", [3; 32], b"unrelated session", false);
    let keys = KeyRing::from_versioned_manifest(&manifest, &[0; 32]).unwrap();
    assert_eq!(
        keys.current_table(&table, "Synthetic.dat").unwrap(),
        b"unrelated session"
    );
}
