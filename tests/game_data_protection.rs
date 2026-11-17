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
