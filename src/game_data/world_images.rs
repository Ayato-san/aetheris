//! Import explicit Unreal level/sector/texture evidence from the native extractor
//! or the offline reference converter. No filename association establishes a verified coordinate transform.
use super::*;
use serde_json::json;

pub(super) const SOURCE: &str = "WorldMapImages";
const LIMIT: usize = 8192;

fn array<'a>(value: &'a Value, key: &str, limit: usize) -> Result<&'a [Value]> {
    value[key]
        .as_array()
        .filter(|a| a.len() <= limit)
        .map(Vec::as_slice)
        .ok_or_else(|| invalid(format!("invalid map image {key}")).into())
}
fn text<'a>(row: &'a Value, key: &str) -> Result<&'a str> {
    row[key]
        .as_str()
        .filter(|s| !s.is_empty() && s.len() <= 2048)
        .ok_or_else(|| invalid(format!("invalid map image {key}")).into())
}
fn sha(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn safe_file(root: &Path, relative: &str) -> Result<PathBuf> {
    if relative.contains('\\') || relative.contains(':') {
        return Err(invalid("invalid map image path").into());
    }
    let mut path = root.to_owned();
    for part in relative.split('/') {
        if matches!(part, "" | "." | "..") {
            return Err(invalid("invalid map image path component").into());
        }
        path.push(part);
        if fs::symlink_metadata(&path)?.file_type().is_symlink() {
            return Err(invalid("symbolic map image paths are not supported").into());
        }
    }
    Ok(path)
}

/// Validate the manifest and all image inputs before starting publication.
pub(super) fn read_manifest(
    root: Option<&Path>,
    mapping: &str,
    inputs: &mut BTreeMap<PathBuf, String>,
) -> Result<Option<TableProbe>> {
    let Some(root) = root else { return Ok(None) };
    let path = root.join("world-maps.json");
    if !path.exists() {
        return Ok(None);
    }
    let path = safe_file(root, "world-maps.json")?;
    let bytes = read_bounded(&path, 32 * 1024 * 1024)?;
    let manifest: Value = serde_json::from_slice(&bytes)?;
    if manifest["format"] != 1 || manifest["mapping_sha256"].as_str() != Some(mapping) {
        return Err(invalid("incompatible world map image manifest").into());
    }
    let levels = array(&manifest, "levels", 4096)?;
    let images = array(&manifest, "images", LIMIT)?;
    let errors = array(&manifest, "errors", 4096)?;
    if manifest["packages"].as_u64() != Some((levels.len() + errors.len()) as u64) {
        return Err(invalid("incomplete map level inventory").into());
    }
    let mut known = BTreeSet::new();
    let mut rows = vec![
        json!({"kind":"manifest","format":1,"mapping_sha256":mapping,"packages":manifest["packages"],"texture_packages":manifest["texture_packages"]}),
    ];
    let mut total = 0usize;
    for row in images {
        let reference = text(row, "reference")?;
        let suffix = reference
            .strip_prefix("WorldMaps/Textures/")
            .filter(|s| sha(s))
            .ok_or_else(|| invalid("invalid exact map texture reference"))?;
        let object = text(row, "object_path")?;
        if !object.starts_with("/Game/")
            || hash(object.as_bytes()) != suffix
            || !known.insert(reference)
        {
            return Err(invalid("invalid or duplicate map texture identity").into());
        }
        let file = text(row, "file")?;
        if file != format!("{reference}.png") {
            return Err(invalid("map texture file differs from its reference").into());
        }
        let path = safe_file(root, file)?;
        let data = read_bounded(&path, 16 * 1024 * 1024)?;
        total = total
            .checked_add(data.len())
            .ok_or_else(|| invalid("map image size overflow"))?;
        if total > 1024 * 1024 * 1024 {
            return Err(invalid("map images exceed 1 GiB").into());
        }
        if data.len() < 24
            || !data.starts_with(b"\x89PNG\r\n\x1a\n")
            || data.get(12..16) != Some(b"IHDR")
        {
            return Err(invalid("map texture is not PNG").into());
        }
        for (key, offset) in [("width", 16), ("height", 20)] {
            let actual = u32::from_be_bytes(data[offset..offset + 4].try_into()?);
            if actual == 0 || actual > 4096 || row[key].as_u64() != Some(u64::from(actual)) {
                return Err(invalid("map PNG dimensions mismatch").into());
            }
        }
        let digest = hash(&data);
        if row["sha256"].as_str() != Some(&digest) {
            return Err(invalid("map PNG hash mismatch").into());
        }
        text(row, "pixel_format")?;
        inputs.insert(path, digest);
        let mut row = row.clone();
        row["kind"] = json!("texture");
        rows.push(row);
    }
    let mut paths = BTreeSet::new();
    let mut actors_count = 0;
    for level in levels {
        let path = text(level, "source_path")?;
        if !path.starts_with("AION2/Content/UI/Map/")
            || !path.ends_with(".umap")
            || !paths.insert(path)
            || !sha(text(level, "payload_sha256")?)
        {
            return Err(invalid("invalid or duplicate map level source").into());
        }
        let mut actors = BTreeSet::new();
        for actor in array(level, "actors", LIMIT)? {
            actors_count += 1;
            if actors_count > 65536
                || !actors.insert(text(actor, "name")?)
                || !known.contains(text(actor, "texture_reference")?)
            {
                return Err(invalid("duplicate, missing or excessive map sector reference").into());
            }
            text(actor, "actor_class")?;
            match actor["index_state"].as_str() {
                Some("explicit")
                    if ["sector_x", "sector_y"]
                        .iter()
                        .all(|k| actor[k].as_i64().is_some_and(|v| (0..=4095).contains(&v))) => {}
                Some("inherited_unresolved")
                    if actor["sector_x"].is_null() && actor["sector_y"].is_null() => {}
                _ => return Err(invalid("invalid map sector index state").into()),
            }
        }
        let mut level = level.clone();
        level["kind"] = json!("level");
        rows.push(level);
    }
    for error in errors {
        text(error, "source_path")?;
        text(error, "error")?;
        let mut error = error.clone();
        error["kind"] = json!("error");
        rows.push(error);
    }
    if manifest.get("texture_errors").is_some() {
        for error in array(&manifest, "texture_errors", LIMIT)? {
            text(error, "source_path")?;
            text(error, "error")?;
            let mut error = error.clone();
            error["kind"] = json!("texture_error");
            rows.push(error);
        }
    }
    inputs.insert(path, hash(&bytes));
    Ok(Some(TableProbe {
        mapping_sha256: mapping.into(),
        mapping_version: 0,
        mapping_ambiguous_structures: BTreeSet::new(),
        row_struct: "WorldMapImageManifestV1".into(),
        payload_sha256: hash(&bytes),
        payload_bytes: bytes.len(),
        layout: super::super::schema::LayoutProbe {
            compatibility_verified: false,
            table_version: 1,
            column_hashes: vec![],
            rows,
        },
    }))
}

pub(super) fn write(
    p: &mut catalogue_projection::Projection<'_>,
    root: Option<&Path>,
) -> Result<()> {
    let rows = catalogue_projection::rows(p.batch, SOURCE);
    if rows.is_empty() {
        return coverage(
            p.db,
            "world.images",
            "not_researched",
            "No world-maps.json extraction manifest supplied",
        );
    }
    let mut textures = BTreeMap::new();
    for (pos, r) in rows
        .iter()
        .enumerate()
        .filter(|(_, r)| r["kind"] == "texture")
    {
        let object = text(r, "object_path")?;
        let id = world::stable(&[SOURCE, "texture", object]);
        let data = read_bounded(
            &safe_file(
                root.ok_or_else(|| invalid("missing map image root"))?,
                text(r, "file")?,
            )?,
            16 * 1024 * 1024,
        )?;
        if hash(&data) != text(r, "sha256")? {
            return Err(invalid("map texture changed during import").into());
        }
        let reference = text(r, "reference")?;
        let icon: i64 = p.db.query_row(
            "SELECT COALESCE(MAX(id),0)+1 FROM catalogue_icons",
            [],
            |r| r.get(0),
        )?;
        p.db.execute(
            "INSERT INTO catalogue_icons VALUES(?1,?2,?3,'image/png','available',?4)",
            params![icon, reference, data, text(r, "sha256")?],
        )?;
        p.db.execute(
            "INSERT INTO world_map_textures VALUES(?1,?2,?3,?4,?5,?6,?7)",
            params![
                id,
                p.build,
                object,
                number(r, "width"),
                number(r, "height"),
                text(r, "pixel_format")?,
                icon
            ],
        )?;
        p.db.execute(
            "INSERT INTO catalogue_entity_icons VALUES('world_map_textures',?1,?2)",
            params![id, icon],
        )?;
        p.source("world_map_textures", id, SOURCE, pos)?;
        textures.insert(reference, id);
    }
    for (pos, r) in rows
        .iter()
        .enumerate()
        .filter(|(_, r)| r["kind"] == "level")
    {
        let path = text(r, "source_path")?;
        let id = world::stable(&[SOURCE, "level", path]);
        let actors = array(r, "actors", LIMIT)?;
        p.db.execute(
            "INSERT INTO world_map_levels VALUES(?1,?2,?3,?4,?5)",
            params![
                id,
                p.build,
                path,
                text(r, "payload_sha256")?,
                actors.len() as i64
            ],
        )?;
        p.source("world_map_levels", id, SOURCE, pos)?;
        for actor in actors {
            p.db.execute(
                "INSERT INTO world_map_sectors VALUES(?1,?2,?3,?4,?5,?6,?7)",
                params![
                    id,
                    text(actor, "name")?,
                    text(actor, "actor_class")?,
                    number(actor, "sector_x"),
                    number(actor, "sector_y"),
                    text(actor, "index_state")?,
                    textures[text(actor, "texture_reference")?]
                ],
            )?;
        }
        // Discovery candidate only; no proof of coordinate/unit equivalence.
        let key = Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .and_then(|s| {
                s.strip_prefix("WorldMap_")
                    .or_else(|| s.strip_prefix("SharedMap_"))
            });
        if let Some(key) = key {
            p.db.execute("INSERT INTO world_map_level_candidates SELECT id,?1,'candidate' FROM world_map_assets WHERE source_path=?2 AND kind='WorldMap'",params![id,format!("WorldMap/{key}.bin")])?;
        }
    }
    let errors = rows
        .iter()
        .filter(|r| r["kind"] == "error" || r["kind"] == "texture_error")
        .count();
    let levels = rows.iter().filter(|r| r["kind"] == "level").count();
    let unresolved: i64 = p.db.query_row(
        "SELECT count(*) FROM world_map_sectors WHERE index_state='inherited_unresolved'",
        [],
        |r| r.get(0),
    )?;
    let unmatched:i64=p.db.query_row("SELECT count(*) FROM world_map_layouts l WHERE NOT EXISTS(SELECT 1 FROM world_map_level_candidates c WHERE c.map_asset_id=l.map_asset_id)",[],|r|r.get(0))?;
    coverage(
        p.db,
        "world.images.associations",
        if unmatched == 0 {
            "uninterpreted"
        } else {
            "partial"
        },
        &format!(
            "Exact WorldMap_/SharedMap_ basename candidates only; {unmatched} WorldMap configurations without a candidate level; world-to-image transforms remain unverified"
        ),
    )?;
    coverage(
        p.db,
        "world.images",
        if errors == 0 { "verified" } else { "partial" },
        &format!(
            "{levels} Unreal levels, {} decoded textures, {errors} level/asset errors, {unresolved} inherited sector indices unresolved; independent textures retained without fabricated level links; map/coordinate associations remain candidates",
            textures.len()
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn partial_image_exports_do_not_report_missing_levels_as_game_deletions() {
        use super::super::tests::{add, batch, item, project};
        let mut data = batch(vec![item(1)]);
        add(
            &mut data,
            SOURCE,
            vec![
                json!({"kind":"level","source_path":"AION2/Content/UI/Map/WorldMap/Test/WorldMap_Test.umap","payload_sha256":"a".repeat(64),"actors":[]}),
            ],
        );
        let (first, _) = project(&data, None, "100", "fr-FR");
        data.get_mut(SOURCE).unwrap().layout.rows = vec![
            json!({"kind":"error","source_path":"AION2/Content/UI/Map/WorldMap/Test/WorldMap_Test.umap","error":"unreadable"}),
        ];
        let (second, summary) = project(&data, Some(&first), "200", "fr-FR");
        assert_eq!(summary.deleted, 0);
        let state:(bool,String)=second.query_row("SELECT present,comparison_state FROM catalogue_entries WHERE family='world_map_levels'",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
        assert_eq!(state, (true, "impossible".into()));
    }
    fn fixture(tag: &str) -> (PathBuf, Value) {
        let root = std::env::temp_dir().join(format!("world-images-{tag}-{}", std::process::id()));
        fs::create_dir_all(root.join("WorldMaps/Textures")).unwrap();
        let png = include_bytes!("../../resources/item-card/can-storage.png");
        let object = "/Game/UI/Map/WorldMap/Test/Res/tile.tile";
        let reference = format!("WorldMaps/Textures/{}", hash(object.as_bytes()));
        let file = format!("{reference}.png");
        fs::write(root.join(&file), png).unwrap();
        let manifest = json!({"format":1,"mapping_sha256":"mapping","packages":1,"errors":[],
            "images":[{"reference":reference,"object_path":object,"file":file,"width":u32::from_be_bytes(png[16..20].try_into().unwrap()),"height":u32::from_be_bytes(png[20..24].try_into().unwrap()),"pixel_format":"PF_B8G8R8A8","sha256":hash(png)}],
            "levels":[{"source_path":"AION2/Content/UI/Map/WorldMap/Test/WorldMap_Test.umap","payload_sha256":"a".repeat(64),"actors":[
                {"name":"plane_a","actor_class":"BP_WorldMapPlane_C","sector_x":1,"sector_y":0,"index_state":"explicit","texture_reference":reference},
                {"name":"plane_b","actor_class":"BP_WorldMapPlane_C","sector_x":null,"sector_y":null,"index_state":"inherited_unresolved","texture_reference":reference}]}]});
        (root, manifest)
    }
    fn save(root: &Path, manifest: &Value) {
        fs::write(
            root.join("world-maps.json"),
            serde_json::to_vec(manifest).unwrap(),
        )
        .unwrap();
    }
    #[test]
    fn exact_images_are_deduplicated_and_inherited_indices_stay_unknown() {
        let (root, mut manifest) = fixture("bindings");
        manifest["levels"][0]["source_path"] =
            json!("AION2/Content/UI/Map/WorldMap/SharedMap/Test/SharedMap_Test.umap");
        let mut independent = manifest["images"][0].clone();
        independent["object_path"] =
            json!("/Game/UI/Map/WorldMap/Test/Res/unreferenced.unreferenced");
        let reference = format!(
            "WorldMaps/Textures/{}",
            hash(independent["object_path"].as_str().unwrap().as_bytes())
        );
        independent["reference"] = json!(reference);
        independent["file"] = json!(format!("{reference}.png"));
        fs::copy(
            root.join(manifest["images"][0]["file"].as_str().unwrap()),
            root.join(independent["file"].as_str().unwrap()),
        )
        .unwrap();
        manifest["images"].as_array_mut().unwrap().push(independent);
        manifest["texture_errors"] = json!([{"source_path":"AION2/Content/UI/Map/WorldMap/Test/unsupported.uasset","error":"unsupported pixel format"}]);
        save(&root, &manifest);
        let mut inputs = BTreeMap::new();
        let table = read_manifest(Some(&root), "mapping", &mut inputs)
            .unwrap()
            .unwrap();
        assert_eq!(inputs.len(), 3);
        let batch = BTreeMap::from([(SOURCE.into(), table)]);
        let db = Connection::open_in_memory().unwrap();
        for sql in [
            include_str!("items.sql"),
            include_str!("catalogue.sql"),
            include_str!("catalogue_projection.sql"),
            include_str!("world.sql"),
        ] {
            db.execute_batch(sql).unwrap();
        }
        db.execute("INSERT INTO client_version VALUES(1,'26581260')", [])
            .unwrap();
        db.execute("INSERT INTO catalogue_sources VALUES(?1,1,'mapping','payload','manifest',0,'decoded',5,X'00')",[SOURCE]).unwrap();
        db.execute("INSERT INTO world_map_assets VALUES(1,1,'WorldMap/Test.bin','WorldMap',NULL,NULL,'hash','source','uninterpreted')",[]).unwrap();
        db.execute("INSERT INTO world_map_assets VALUES(2,1,'WorldMap/Test_similar.bin','WorldMap',NULL,NULL,'hash','source','uninterpreted')",[]).unwrap();
        let loc = Localization {
            namespace: String::new(),
            entries: BTreeMap::new(),
        };
        let mut p = catalogue_projection::Projection::new(&db, &batch, &loc, &loc, 1);
        write(&mut p, Some(&root)).unwrap();
        assert_eq!(
            db.query_row("SELECT count(*) FROM world_map_textures", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            2
        );
        assert_eq!(db.query_row("SELECT count(*) FROM world_map_textures t WHERE NOT EXISTS(SELECT 1 FROM world_map_sectors s WHERE s.texture_id=t.id)",[],|r|r.get::<_,i64>(0)).unwrap(),1);
        assert_eq!(
            db.query_row(
                "SELECT state FROM catalogue_coverage WHERE subject='world.images'",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
            "partial"
        );
        assert_eq!(
            db.query_row("SELECT count(*) FROM world_map_sectors", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            2
        );
        assert_eq!(
            db.query_row(
                "SELECT sector_x FROM world_map_sectors WHERE actor_name='plane_b'",
                [],
                |r| r.get::<_, Option<i64>>(0)
            )
            .unwrap(),
            None
        );
        assert_eq!(
            db.query_row("SELECT count(*) FROM world_map_level_candidates", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            db.query_row(
                "SELECT interpretation_state FROM world_map_level_candidates WHERE map_asset_id=1",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
            "candidate"
        );
        assert!(
            !db.prepare("PRAGMA foreign_key_check")
                .unwrap()
                .exists([])
                .unwrap()
        );
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn rejects_mismatched_mapping_hash_dimensions_identity_and_missing_bindings() {
        let (root, manifest) = fixture("invalid");
        for variant in 0..8 {
            let mut m = manifest.clone();
            match variant {
                0 => m["mapping_sha256"] = json!("other"),
                1 => m["images"][0]["sha256"] = json!("0".repeat(64)),
                2 => m["images"][0]["width"] = json!(1),
                3 => m["images"][0]["file"] = json!("../outside.png"),
                4 => m["levels"][0]["actors"][0]["texture_reference"] = json!("missing"),
                5 => m["levels"][0]["actors"][1]["name"] = json!("plane_a"),
                6 => m["packages"] = json!(2),
                _ => m["images"][0]["object_path"] = json!("/Game/unrelated.texture"),
            }
            save(&root, &m);
            assert!(
                read_manifest(Some(&root), "mapping", &mut BTreeMap::new()).is_err(),
                "variant {variant}"
            );
        }
        fs::remove_dir_all(root).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn rejects_symlinked_texture_directories() {
        let (root, manifest) = fixture("symlink");
        save(&root, &manifest);
        fs::rename(root.join("WorldMaps"), root.join("actual")).unwrap();
        std::os::unix::fs::symlink(root.join("actual"), root.join("WorldMaps")).unwrap();
        assert!(read_manifest(Some(&root), "mapping", &mut BTreeMap::new()).is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
