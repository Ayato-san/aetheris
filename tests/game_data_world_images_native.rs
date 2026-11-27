use serde_json::Value;
use std::{collections::BTreeMap, fs, path::Path};
#[test]
#[ignore = "requires both locally extracted map image directories"]
fn native_world_images_match_the_offline_reference() -> Result<(), Box<dyn std::error::Error>> {
    let args = [
        std::env::var("AETHERIS_MAP_REFERENCE")?,
        std::env::var("AETHERIS_MAP_NATIVE")?,
    ];
    let a = Path::new(&args[0]);
    let b = Path::new(&args[1]);
    let x: Value = serde_json::from_slice(&fs::read(a.join("world-maps.json"))?)?;
    let y: Value = serde_json::from_slice(&fs::read(b.join("world-maps.json"))?)?;
    let rows = |m: &Value| {
        m["images"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| (x["object_path"].as_str().unwrap().to_owned(), x.clone()))
            .collect::<BTreeMap<_, _>>()
    };
    let xx = rows(&x);
    let yy = rows(&y);
    assert_eq!(xx.keys().collect::<Vec<_>>(), yy.keys().collect::<Vec<_>>());
    for (i, (key, row)) in xx.iter().enumerate() {
        let r = &yy[key];
        for field in ["reference", "width", "height", "pixel_format"] {
            assert_eq!(row[field], r[field], "{key}: {field}");
        }
        let aa = image::load_from_memory(&fs::read(a.join(row["file"].as_str().unwrap()))?)?
            .into_rgba8();
        let bb =
            image::load_from_memory(&fs::read(b.join(r["file"].as_str().unwrap()))?)?.into_rgba8();
        assert_eq!(aa.dimensions(), bb.dimensions(), "{key}");
        if aa != bb {
            let n = aa
                .as_raw()
                .iter()
                .zip(bb.as_raw())
                .filter(|(a, b)| a != b)
                .count();
            return Err(format!("{key}: {n} different channels").into());
        }
        if (i + 1) % 100 == 0 {
            println!("Verified {} textures", i + 1);
        }
    }
    let ll = |m: &Value| {
        m["levels"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| (x["source_path"].as_str().unwrap().to_owned(), x.clone()))
            .collect::<BTreeMap<_, _>>()
    };
    assert_eq!(ll(&x), ll(&y));
    for m in [&x, &y] {
        assert!(m["errors"].as_array().unwrap().is_empty());
        assert!(m["texture_errors"].as_array().unwrap().is_empty());
    }
    println!(
        "All {} textures and {} levels match",
        xx.len(),
        ll(&x).len()
    );
    Ok(())
}
