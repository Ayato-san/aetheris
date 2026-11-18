use std::{fs, path::PathBuf};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let mut random = [0u8; 16];
        getrandom::fill(&mut random).unwrap();
        let path = std::env::temp_dir().join(format!("aetheris-inventory-{}", hex::encode(random)));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn inventories_content_reproducibly_across_client_paths() {
    let a = Fixture::new();
    let b = Fixture::new();
    for fixture in [&a, &b] {
        fs::create_dir(fixture.0.join("Paks")).unwrap();
        fs::write(fixture.0.join("Paks/z.UCAS"), b"abc").unwrap();
        fs::write(fixture.0.join("Paks/a.pak"), b"").unwrap();
        fs::write(fixture.0.join("ignored.txt"), b"ignored").unwrap();
    }
    let inventory = aetheris::game_data::inspect(&a.0).unwrap();
    assert_eq!(inventory, aetheris::game_data::inspect(&b.0).unwrap());
    assert_eq!(inventory.len(), 2);
    assert_eq!(inventory[1].path, ["Paks", "z.UCAS"]);
    assert_eq!(
        inventory[1].sha256,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    fs::write(b.0.join("Paks/z.UCAS"), b"changed").unwrap();
    assert_ne!(inventory, aetheris::game_data::inspect(&b.0).unwrap());
}

#[test]
fn missing_archives_are_reported() {
    let fixture = Fixture::new();
    assert_eq!(
        aetheris::game_data::inspect(&fixture.0).unwrap_err().kind(),
        std::io::ErrorKind::NotFound
    );
}

#[test]
fn discovers_all_steam_libraries_and_keeps_verified_metadata_separate_from_declarations() {
    use aetheris::game_data::{discovery, inventory};
    let fixture = Fixture::new();
    let steam = fixture.0.join("Steam");
    let library = fixture.0.join("library");
    fs::create_dir_all(steam.join("steamapps")).unwrap();
    let client = library.join("steamapps/common/AION2");
    fs::create_dir_all(client.join("Aion2/Content/Paks")).unwrap();
    fs::write(client.join("Aion2/Content/Paks/data.pak"), b"test").unwrap();
    let library_path = library.to_str().unwrap().replace('\\', "\\\\");
    fs::write(
        steam.join("steamapps/libraryfolders.vdf"),
        format!("\"libraryfolders\" {{ \"0\" {{ \"path\" \"{library_path}\" }} }}"),
    )
    .unwrap();
    let manifest = library.join("steamapps/appmanifest_123.acf");
    fs::write(
        &manifest,
        "\"AppState\" { \"installdir\" \"AION2\" \"buildid\" \"25800038\" }",
    )
    .unwrap();
    let discovery = discovery::installations(&[steam]);
    assert!(discovery.problems.is_empty(), "{:?}", discovery.problems);
    assert_eq!(
        discovery.installations,
        vec![client.canonicalize().unwrap()]
    );
    let snapshot = inventory(&client).unwrap();
    assert!(snapshot.client.is_none());
    assert_eq!(snapshot.metadata[0].version.as_deref(), Some("25800038"));
    assert_eq!(snapshot.metadata[0].region, None);
    assert_eq!(snapshot.metadata[0].kind, "steam_appmanifest");
    fs::write(
        &manifest,
        "\"AppState\" { \"installdir\" \"AION2\" \"buildid\" \"25800039\" }",
    )
    .unwrap();
    assert!(snapshot.verify_unchanged(&client).is_err());
}

#[test]
fn partitions_share_the_base_iostore_toc_and_missing_companions_are_reported() {
    use aetheris::game_data::inventory;
    let fixture = Fixture::new();
    for name in ["data.UTOC", "data.UCAS", "data_s1.ucas", "data_s2.ucas"] {
        fs::write(fixture.0.join(name), b"test").unwrap();
    }
    assert!(inventory(&fixture.0).unwrap().problems.is_empty());
    fs::remove_file(fixture.0.join("data.UTOC")).unwrap();
    assert_eq!(inventory(&fixture.0).unwrap().problems.len(), 3);
}

#[test]
fn copied_steam_manifest_preserves_installed_and_target_builds_without_private_fields() {
    use aetheris::game_data::discovery::steam_manifest;
    use sha2::{Digest, Sha256};
    let fixture = Fixture::new();
    let path = fixture.0.join("appmanifest_123.acf");
    let bytes = br#""AppState" {
        "appid" "123" "installdir" "AION2" "buildid" "100"
        "TargetBuildID" "101" "StateFlags" "6"
        "LastOwner" "private-account" "LauncherPath" "private-path"
        "InstalledDepots" {
            "124" { "manifest" "18446744073709551615" "size" "90000000000" }
        }
        "UserConfig" { "language" "french" }
        "MountedConfig" { "language" "english" }
    }"#;
    fs::write(&path, bytes).unwrap();
    let manifest = steam_manifest(&path).unwrap();
    assert_eq!(manifest.app_id, "123");
    assert_eq!(manifest.install_directory, "AION2");
    assert_eq!(manifest.build_id, "100");
    assert_eq!(manifest.target_build_id.as_deref(), Some("101"));
    assert_eq!(manifest.state_flags, 6);
    assert_eq!(manifest.language.as_deref(), Some("english"));
    assert_eq!(
        manifest.installed_depots["124"].manifest_id,
        "18446744073709551615"
    );
    assert_eq!(manifest.installed_depots["124"].size, 90_000_000_000);
    assert_eq!(manifest.sha256, hex::encode(Sha256::digest(bytes)));
    assert_eq!(manifest, steam_manifest(&path).unwrap());
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_aetheris-game-data"))
        .arg("inspect-steam-manifest")
        .arg(&path)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&output.stdout).unwrap(),
        serde_json::to_value(&manifest).unwrap()
    );
    let json = String::from_utf8(output.stdout).unwrap();
    assert!(!json.contains("private-"));
    assert!(!json.contains("LastOwner"));
    assert!(!json.contains("region"));
    assert!(!json.contains("app_version"));
    fs::write(
        &path,
        std::str::from_utf8(bytes)
            .unwrap()
            .replace("private-account", "changed-account"),
    )
    .unwrap();
    assert_ne!(manifest.sha256, steam_manifest(&path).unwrap().sha256);
}

#[test]
fn steam_manifest_does_not_substitute_download_target_or_requested_language() {
    use aetheris::game_data::discovery::steam_manifest;
    let fixture = Fixture::new();
    let path = fixture.0.join("copy.acf");
    fs::write(
        &path,
        br#""AppState" {
        "appid" "123" "installdir" "AION2" "buildid" "0" "StateFlags" "0"
        "UserConfig" { "language" "french" }
    }"#,
    )
    .unwrap();
    let manifest = steam_manifest(&path).unwrap();
    assert_eq!(manifest.build_id, "0");
    assert!(manifest.target_build_id.is_none());
    assert!(manifest.language.is_none());
    assert!(manifest.installed_depots.is_empty());
}

#[test]
fn malformed_or_mismatched_copied_steam_manifests_are_rejected() {
    use aetheris::game_data::discovery::steam_manifest;
    let fixture = Fixture::new();
    let path = fixture.0.join("appmanifest_123.acf");
    let valid = r#""AppState" {
        "appid" "123" "installdir" "AION2" "buildid" "100" "StateFlags" "4"
        "TargetBuildID" "101"
        "InstalledDepots" { "124" { "manifest" "999" "size" "42" } }
    }"#;
    for (from, to) in [
        (r#""appid" "123""#, r#""appid" "456""#),
        (r#""appid" "123""#, r#""appid" "bad""#),
        (r#""buildid" "100""#, r#""buildid" """#),
        (r#""buildid" "100""#, r#""buildid" "-1""#),
        (r#""buildid" "100""#, r#""buildid" {}"#),
        (
            r#""StateFlags" "4""#,
            r#""StateFlags" "18446744073709551616""#,
        ),
        (r#""TargetBuildID" "101""#, r#""TargetBuildID" {}"#),
        (
            r#""InstalledDepots" { "124" { "manifest" "999" "size" "42" } }"#,
            r#""InstalledDepots" "bad""#,
        ),
        (r#""124" {"#, r#""bad" {"#),
        (r#""manifest" "999""#, r#""manifest" "bad""#),
        (r#""size" "42""#, r#""size" "18446744073709551616""#),
        (r#""appid" "123""#, r#""appid" "123" "AppID" "123""#),
    ] {
        fs::write(&path, valid.replace(from, to)).unwrap();
        assert!(
            steam_manifest(&path).is_err(),
            "accepted replacement {from} -> {to}"
        );
    }
    for directory in [
        "",
        ".",
        "..",
        "../AION2",
        "..\\AION2",
        "C:\\AION2",
        "/AION2",
    ] {
        fs::write(&path, valid.replace("AION2", directory)).unwrap();
        assert!(steam_manifest(&path).is_err(), "accepted {directory}");
    }
    fs::write(&path, vec![b' '; 4 * 1024 * 1024 + 1]).unwrap();
    assert!(steam_manifest(&path).is_err());
}

#[test]
fn malformed_steam_metadata_is_reported_without_hiding_other_installations() {
    use aetheris::game_data::discovery;
    let fixture = Fixture::new();
    fs::create_dir_all(fixture.0.join("steamapps")).unwrap();
    fs::create_dir_all(fixture.0.join("Aion2/Content/Paks")).unwrap();
    fs::write(
        fixture.0.join("steamapps/libraryfolders.vdf"),
        b"\"libraryfolders\" { \"0\" { \"path\" \"unterminated",
    )
    .unwrap();
    let result = discovery::installations(std::slice::from_ref(&fixture.0));
    assert_eq!(result.installations.len(), 1);
    assert_eq!(result.problems.len(), 1);
}

#[test]
fn missing_source_and_changed_identity_are_not_accepted() {
    use aetheris::game_data::inventory;
    let fixture = Fixture::new();
    assert!(inventory(&fixture.0.join("missing")).is_err());
    fs::write(fixture.0.join("a.pak"), b"source").unwrap();
    fs::write(
        fixture.0.join("aetheris-client.json"),
        br#"{"version":"test-1","region":"test"}"#,
    )
    .unwrap();
    let snapshot = inventory(&fixture.0).unwrap();
    assert_eq!(snapshot.metadata[0].kind, "operator_declaration");
    fs::write(
        fixture.0.join("aetheris-client.json"),
        br#"{"version":"test-2","region":"test"}"#,
    )
    .unwrap();
    assert!(snapshot.verify_unchanged(&fixture.0).is_err());
    fs::remove_file(fixture.0.join("a.pak")).unwrap();
    assert!(snapshot.verify_unchanged(&fixture.0).is_err());
}

#[cfg(unix)]
#[test]
fn symbolic_archive_links_do_not_produce_an_incomplete_inventory() {
    let fixture = Fixture::new();
    fs::write(fixture.0.join("a.pak"), b"source").unwrap();
    std::os::unix::fs::symlink(fixture.0.join("a.pak"), fixture.0.join("link.pak")).unwrap();
    assert!(aetheris::game_data::inspect(&fixture.0).is_err());
}
