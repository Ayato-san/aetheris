//! Original synthetic containers: no game assets or production keys.
use aes::{
    Aes256, Block,
    cipher::{BlockEncrypt, KeyInit},
};
use aetheris::game_data::{
    archives::{Pak, extract_data, iostore::IoStore},
    assets,
};
use sha1::{Digest, Sha1};
use std::{fs, path::Path};

struct Fixture(std::path::PathBuf);
impl Fixture {
    fn new() -> Self {
        let mut random = [0; 16];
        getrandom::fill(&mut random).unwrap();
        let path = std::env::temp_dir().join(format!("aetheris-archives-{}", hex::encode(random)));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn u32le(out: &mut Vec<u8>, n: u32) {
    out.extend(n.to_le_bytes());
}
fn u64le(out: &mut Vec<u8>, n: u64) {
    out.extend(n.to_le_bytes());
}
fn string(out: &mut Vec<u8>, value: &str) {
    u32le(out, value.len() as u32 + 1);
    out.extend(value.as_bytes());
    out.push(0);
}
fn encrypt(bytes: &mut Vec<u8>) {
    bytes.resize(bytes.len().next_multiple_of(16), 0);
    let cipher = Aes256::new_from_slice(&[0; 32]).unwrap();
    for chunk in bytes.as_chunks_mut::<16>().0 {
        let mut block = Block::clone_from_slice(chunk);
        cipher.encrypt_block(&mut block);
        chunk.copy_from_slice(&block);
    }
}
fn pak(path: &Path, payload: &[u8], encrypted: bool, oodle: bool, name: &str, deleted: bool) {
    let mut packed = if oodle { vec![0xcc, 6] } else { vec![] };
    packed.extend(payload);
    let packed_len = packed.len();
    if encrypted {
        encrypt(&mut packed);
    }
    let mut bytes = vec![0; if oodle { 73 } else { 53 }];
    bytes.extend(packed);
    let mut directory = Vec::new();
    u32le(&mut directory, 1);
    string(&mut directory, "Data/");
    u32le(&mut directory, 1);
    string(&mut directory, name);
    u32le(&mut directory, if deleted { i32::MIN as u32 } else { 0 });
    if encrypted {
        directory.resize(directory.len().next_multiple_of(16), 0);
    }
    let dir_hash = Sha1::digest(&directory);
    let dir_offset = bytes.len();
    if encrypted {
        encrypt(&mut directory);
    }
    bytes.extend(&directory);
    let mut encoded = Vec::new();
    let bits = (1 << 31)
        | (1 << 30)
        | if encrypted { 1 << 22 } else { 0 }
        | if oodle {
            (1 << 29) | (1 << 23) | (1 << 6)
        } else {
            0
        };
    u32le(&mut encoded, bits);
    u32le(&mut encoded, 0);
    u32le(&mut encoded, payload.len() as u32);
    if oodle {
        u32le(&mut encoded, packed_len as u32);
        if encrypted {
            u32le(&mut encoded, packed_len as u32);
        }
    }
    let mut index = Vec::new();
    string(&mut index, "../../../Aion2/Content/");
    u32le(&mut index, 1);
    u64le(&mut index, 0);
    u32le(&mut index, 0);
    u32le(&mut index, 1);
    u64le(&mut index, dir_offset as u64);
    u64le(&mut index, directory.len() as u64);
    index.extend(dir_hash);
    u32le(&mut index, encoded.len() as u32);
    index.extend(encoded);
    u32le(&mut index, 0);
    if encrypted {
        index.resize(index.len().next_multiple_of(16), 0);
    }
    let hash = Sha1::digest(&index);
    let offset = bytes.len();
    if encrypted {
        encrypt(&mut index);
    }
    bytes.extend(&index);
    bytes.extend([0; 16]);
    bytes.push(encrypted as u8);
    u32le(&mut bytes, 0x5a6f12e1);
    u32le(&mut bytes, 11);
    u64le(&mut bytes, offset as u64);
    u64le(&mut bytes, index.len() as u64);
    bytes.extend(hash);
    let mut codecs = [0; 160];
    codecs[..5].copy_from_slice(b"Oodle");
    bytes.extend(codecs);
    fs::write(path, bytes).unwrap();
}
#[test]
fn pak_reads_encrypted_and_oodle_payloads_and_rejects_wrong_keys_and_corruption() {
    let dir = Fixture::new();
    let path = dir.path().join("test.pak");
    for encrypted in [false, true] {
        for oodle in [false, true] {
            pak(
                &path,
                b"synthetic payload",
                encrypted,
                oodle,
                "Skill.dat",
                false,
            );
            let archive = Pak::open(&path, Some([0; 32])).unwrap();
            let entry = archive.entries.get("aion2/content/data/skill.dat").unwrap();
            assert_eq!(archive.read(entry).unwrap(), b"synthetic payload");
            if encrypted {
                assert!(Pak::open(&path, None).is_err());
                assert!(Pak::open(&path, Some([1; 32])).is_err());
            }
            let mut bytes = fs::read(&path).unwrap();
            let trailer = bytes.len() - 221;
            bytes[trailer + 41] ^= 1;
            fs::write(&path, &bytes).unwrap();
            assert!(Pak::open(&path, Some([0; 32])).is_err());
        }
    }
    pak(&path, b"a", false, false, "../bad.dat", false);
    assert!(Pak::open(&path, None).is_err());
    let bytes = fs::read(&path).unwrap();
    for end in [0, 16, 220, bytes.len() - 1] {
        fs::write(&path, &bytes[..end]).unwrap();
        assert!(Pak::open(&path, None).is_err());
    }
}
#[test]
fn extraction_applies_patch_replacement_and_tombstones_and_invalidates_stale_reports() {
    let dir = Fixture::new();
    let root = dir.path().join("paks");
    let out = dir.path().join("out");
    fs::create_dir(&root).unwrap();
    pak(
        &root.join("base.pak"),
        b"base",
        false,
        false,
        "Skill.dat",
        false,
    );
    pak(
        &root.join("patch_1_P.pak"),
        b"patch",
        false,
        false,
        "Skill.dat",
        false,
    );
    let report = extract_data(&root, &out, [0; 32]).unwrap();
    assert!(report.errors.is_empty());
    assert_eq!(report.files.len(), 1);
    assert_eq!(fs::read(out.join(&report.files[0].path)).unwrap(), b"patch");
    assert_eq!(report.files[0].container, "patch_1_P.pak");
    pak(
        &root.join("patch_2_P.pak"),
        b"",
        false,
        false,
        "Skill.dat",
        true,
    );
    assert!(extract_data(&root, &out, [0; 32]).unwrap().files.is_empty());
    fs::write(root.join("broken.pak"), b"broken").unwrap();
    assert!(extract_data(&root, &out, [0; 32]).is_err());
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(out.join("extraction.json")).unwrap()).unwrap();
    assert!(!report["errors"].as_array().unwrap().is_empty());
    assert!(extract_data(&root, &root.join("output"), [0; 32]).is_err());
}
fn toc(path: &Path, encrypted: bool) {
    let mut bytes = b"-==--==--==--==-".to_vec();
    bytes.extend([5, 0, 0, 0]);
    for n in [144, 1, 1, 12, 0, 0, 16, 0, 1] {
        u32le(&mut bytes, n);
    }
    bytes.extend([0; 24]);
    u32le(&mut bytes, if encrypted { 2 } else { 0 });
    u32le(&mut bytes, 0);
    u64le(&mut bytes, u64::MAX);
    u32le(&mut bytes, 0);
    bytes.extend([0; 44]);
    assert_eq!(bytes.len(), 144);
    bytes.extend([0; 12]);
    bytes.extend([0, 0, 0, 0, 2, 0, 0, 0, 0, 5]);
    u64le(&mut bytes, 10u64 << 40);
    u32le(&mut bytes, 10);
    fs::write(path, bytes).unwrap();
    let mut data = b"0123456789".to_vec();
    if encrypted {
        encrypt(&mut data);
    }
    fs::write(path.with_extension("ucas"), data).unwrap();
}
#[test]
fn iostore_reads_partial_chunks_and_checks_partitions_truncation_and_versions() {
    let dir = Fixture::new();
    let path = dir.path().join("test.utoc");
    for encrypted in [false, true] {
        toc(&path, encrypted);
        let store = IoStore::open(&path, Some([0; 32])).unwrap();
        assert_eq!(store.read_chunk(0).unwrap(), b"23456");
        assert!(store.read_chunk(1).is_err());
        if encrypted {
            assert!(IoStore::open(&path, None).unwrap().read_chunk(0).is_err());
        }
    }
    let bytes = fs::read(&path).unwrap();
    for end in 0..bytes.len() {
        fs::write(&path, &bytes[..end]).unwrap();
        assert!(IoStore::open(&path, None).is_err(), "{end}");
    }
    let mut wrong = bytes.clone();
    wrong[16] = 6;
    fs::write(&path, wrong).unwrap();
    assert!(IoStore::open(&path, None).is_err());
    fs::write(&path, bytes).unwrap();
    fs::remove_file(path.with_extension("ucas")).unwrap();
    assert!(IoStore::open(&path, None).is_err());
}
#[test]
fn localisation_rejects_duplicates_and_all_truncations_and_decryption_invalidates_stale_report() {
    let mut bytes = 1u32.to_le_bytes().to_vec();
    string(&mut bytes, "test");
    u32le(&mut bytes, 1);
    string(&mut bytes, "key");
    string(&mut bytes, "texte");
    assert_eq!(
        assets::localization(&bytes).unwrap().entries["key"],
        "texte"
    );
    for end in 0..bytes.len() {
        assert!(assets::localization(&bytes[..end]).is_err());
    }
    let mut terminated = bytes.clone();
    terminated.extend([0; 4]);
    assert!(assets::localization(&terminated).is_ok());
    terminated.push(0);
    assert!(assets::localization(&terminated).is_err());
    terminated.pop();
    *terminated.last_mut().unwrap() = 1;
    assert!(assets::localization(&terminated).is_err());
    bytes[13..17].copy_from_slice(&2u32.to_le_bytes());
    string(&mut bytes, "key");
    string(&mut bytes, "autre");
    assert!(assets::localization(&bytes).is_err());
    let dir = Fixture::new();
    let raw = dir.path().join("raw");
    let out = dir.path().join("out");
    fs::create_dir(&raw).unwrap();
    fs::create_dir(&out).unwrap();
    fs::write(out.join("decryption.json"), b"{}").unwrap();
    fs::write(raw.join("extraction.json"), br#"{"files":[],"errors":[]}"#).unwrap();
    assert!(assets::decrypt_extraction(&raw, &[0; 32], &out).is_err());
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(out.join("decryption.json")).unwrap()).unwrap();
    assert!(!report["errors"].as_array().unwrap().is_empty());
    assert!(assets::decrypt_extraction(&raw, &[0; 32], &raw).is_err());
}

#[cfg(unix)]
#[test]
fn extraction_refuses_output_symlinks_that_escape_the_destination() {
    use std::os::unix::fs::symlink;
    let dir = Fixture::new();
    let root = dir.path().join("paks");
    let out = dir.path().join("out");
    let other = dir.path().join("other");
    fs::create_dir(&root).unwrap();
    fs::create_dir(&out).unwrap();
    fs::create_dir(&other).unwrap();
    pak(
        &root.join("base.pak"),
        b"payload",
        false,
        false,
        "Skill.dat",
        false,
    );
    symlink(&other, out.join("Aion2")).unwrap();
    let report = extract_data(&root, &out, [0; 32]).unwrap();
    assert!(report.files.is_empty());
    assert_eq!(report.errors.len(), 1);
    assert_eq!(fs::read_dir(&other).unwrap().count(), 0);
}

fn partitioned_toc(path: &Path, encrypted: bool) {
    let mut bytes = b"-==--==--==--==-".to_vec();
    bytes.extend([5, 0, 0, 0]);
    for n in [144, 1, 2, 12, 1, 32, 16, 0, 2] {
        u32le(&mut bytes, n);
    }
    bytes.extend([0; 24]);
    u32le(&mut bytes, if encrypted { 2 } else { 0 });
    u32le(&mut bytes, 0);
    u64le(&mut bytes, 64);
    u32le(&mut bytes, 0);
    bytes.extend([0; 44]);
    bytes.extend([0; 12]);
    bytes.extend([0, 0, 0, 0, 7, 0, 0, 0, 0, 17]);
    // Two uncompressed Oodle records, in distinct physical partitions.
    u64le(&mut bytes, 18u64 << 40);
    u32le(&mut bytes, (1 << 24) | 16);
    u64le(&mut bytes, (12u64 << 40) | 64);
    u32le(&mut bytes, (1 << 24) | 10);
    let mut name = [0; 32];
    name[..5].copy_from_slice(b"Oodle");
    bytes.extend(name);
    fs::write(path, bytes).unwrap();
    for (partition, payload) in [b"0123456789abcdef".as_slice(), b"ghijklmnop"]
        .into_iter()
        .enumerate()
    {
        let mut data = vec![0xcc, 6];
        data.extend(payload);
        if encrypted {
            encrypt(&mut data);
        }
        let file = if partition == 0 {
            path.with_extension("ucas")
        } else {
            path.with_file_name("test_s1.ucas")
        };
        fs::write(file, data).unwrap();
    }
}

#[test]
fn iostore_checks_toc_baseline_and_reads_across_encrypted_oodle_partitions() {
    let dir = Fixture::new();
    let path = dir.path().join("test.utoc");
    for encrypted in [false, true] {
        partitioned_toc(&path, encrypted);
        let digest = hex::encode(<sha2::Sha256 as Digest>::digest(fs::read(&path).unwrap()));
        let store = IoStore::open_with_hash(&path, Some([0; 32]), Some(&digest)).unwrap();
        assert_eq!(store.summary().toc_sha256, digest);
        assert!(!store.summary().signatures_verified);
        assert!(!store.summary().package_dependencies_verified);
        assert_eq!(store.read_chunk(0).unwrap(), b"789abcdefghijklmn");
        assert!(IoStore::open_with_hash(&path, Some([0; 32]), Some(&"00".repeat(32))).is_err());
        if encrypted {
            assert!(IoStore::open(&path, None).unwrap().read_chunk(0).is_err());
        }
        // A known layout with an unavailable codec must give an explicit error.
        let original = fs::read(&path).unwrap();
        let mut wrong = original.clone();
        let codec = wrong.len() - 32;
        wrong[codec..codec + 5].copy_from_slice(b"Other");
        fs::write(&path, &wrong).unwrap();
        let error = IoStore::open(&path, Some([0; 32]))
            .unwrap()
            .read_chunk(0)
            .unwrap_err();
        assert!(error.to_string().contains("Other"));
        // Stored AES padding also has to fit inside the declared partition.
        let mut wrong = original.clone();
        wrong[88..96].copy_from_slice(&16u64.to_le_bytes());
        fs::write(&path, &wrong).unwrap();
        assert!(IoStore::open(&path, Some([0; 32])).is_err());
        // A logical slice may not reach the short final block's padding.
        let mut wrong = original.clone();
        wrong[165] = 20;
        fs::write(&path, &wrong).unwrap();
        assert!(IoStore::open(&path, Some([0; 32])).is_err());
        fs::write(&path, &original).unwrap();
        fs::write(path.with_file_name("test_s1.ucas"), [0]).unwrap();
        assert!(IoStore::open(&path, Some([0; 32])).is_err());
    }
}

#[test]
fn iostore_rejects_a_chunk_above_64_mib_before_decompression() {
    let dir = Fixture::new();
    let path = dir.path().join("large.utoc");
    let mut bytes = b"-==--==--==--==-".to_vec();
    bytes.extend([5, 0, 0, 0]);
    for n in [144, 1, 65, 12, 1, 32, 1024 * 1024, 0, 1] {
        u32le(&mut bytes, n);
    }
    bytes.extend([0; 24]);
    u32le(&mut bytes, 0);
    u32le(&mut bytes, 0);
    u64le(&mut bytes, u64::MAX);
    u32le(&mut bytes, 0);
    bytes.extend([0; 44]);
    bytes.extend([0; 12]);
    bytes.extend([0, 0, 0, 0, 0, 0, 4, 0, 0, 1]);
    for offset in 0..65 {
        u64le(&mut bytes, (1u64 << 40) | offset);
        u32le(&mut bytes, (1 << 24) | (1024 * 1024));
    }
    let mut codec = [0; 32];
    codec[..5].copy_from_slice(b"Oodle");
    bytes.extend(codec);
    fs::write(&path, bytes).unwrap();
    fs::write(path.with_extension("ucas"), [0; 65]).unwrap();
    let store = IoStore::open(&path, None).unwrap();
    assert!(
        store
            .read_chunk(0)
            .unwrap_err()
            .to_string()
            .contains("allocation limit")
    );
}
