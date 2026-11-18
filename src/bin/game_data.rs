use aetheris::game_data::{
    self,
    catalog::{self, Build, Catalog},
    schema::Schema,
};
use std::{env, fs, path::Path, process::ExitCode, sync::atomic::AtomicBool};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Game data: {error}");
            ExitCode::FAILURE
        }
    }
}

fn json(value: &impl serde::Serialize) -> catalog::Result<()> {
    serde_json::to_writer_pretty(std::io::stdout().lock(), value)?;
    Ok(())
}

fn run() -> catalog::Result<()> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let command = args.first().and_then(|a| a.to_str()).unwrap_or("");
    match (command, &args[usize::from(!args.is_empty())..]) {
        ("probe-iostore",[toc,key,baseline @ ..]) if baseline.len() <= 1 => {
            let key=game_data::archives::read_key(Path::new(key))?;
            let expected = baseline.first().map(|value| value.to_str().ok_or("TOC SHA-256 must be UTF-8")).transpose()?;
            let archive=game_data::archives::iostore::IoStore::open_with_hash(Path::new(toc),Some(key),expected)?;
            let sample=if archive.summary().chunks>0 {Some(archive.read_chunk(0)?.len())}else{None};
            json(&serde_json::json!({"toc":Path::new(toc).display().to_string(),"summary":archive.summary(),"first_chunk_bytes":sample}))
        }
        ("decrypt-data",[raw,material,output])=> {
            let material=game_data::archives::read_key(Path::new(material))?;
            let report=game_data::assets::decrypt_extraction(Path::new(raw),&material,Path::new(output))?;
            let failed=!report.errors.is_empty();json(&report)?;
            if failed {return Err("Some protected assets could not be decoded; see decryption.json".into());}Ok(())
        }
        ("extract-data",[paks,key,output])=> {
            let key=game_data::archives::read_key(Path::new(key))?;
            let report=game_data::archives::extract_data(Path::new(paks),Path::new(output),key)?;
            let failed=!report.errors.is_empty();json(&report)?;
            if failed {return Err("Some files could not be extracted; see extraction.json".into());}Ok(())
        }
        ("inspect", [directory]) => json(&game_data::inventory(Path::new(directory))?),
        ("inspect-steam-manifest", [manifest]) => json(&game_data::discovery::steam_manifest(Path::new(manifest))?),
        ("discover", roots) => {
            let roots = if roots.is_empty() { game_data::discovery::default_roots() } else { roots.iter().map(Into::into).collect() };
            json(&game_data::discovery::installations(&roots))
        }
        ("decode", [schema, identity, payload]) => {
            let schema: Schema = serde_json::from_slice(&fs::read(schema)?)?;
            let identity = serde_json::from_slice(&fs::read(identity)?)?;
            json(&schema.decode(&identity, &fs::read(payload)?)?)
        }
        ("decode-cached", [schema, identity, payload, store]) => {
            let schema_bytes=fs::read(schema)?;
            let schema:Schema=serde_json::from_slice(&schema_bytes)?;
            let identity=serde_json::from_slice(&fs::read(identity)?)?;
            let payload=fs::read(payload)?;
            let entry=game_data::cache::decode(Path::new(store),&payload,&schema_bytes,&identity,"aetheris-schema-v1",&AtomicBool::new(false),|| Ok(serde_json::to_vec(&schema.decode(&identity,&payload)?)?))?;
            use std::io::Write;
            std::io::stdout().lock().write_all(&entry.bytes)?;
            Ok(())
        }
        ("import-reviewed", [build, source, store]) => {
            let build: Build = serde_json::from_slice(&fs::read(build)?)?;
            let store = Path::new(store);
            json(&catalog::publish(&build, Path::new(source), store, &AtomicBool::new(false))?)
        }
        ("purge-cache", [store]) => { catalog::purge_cache(Path::new(store))?; Ok(()) }
        ("install-resources", [bundle, store, identity]) => {
            let identity=serde_json::from_slice(&fs::read(identity)?)?;
            let key=aetheris::companion::manifest::public_key()?;
            let bundle=game_data::resources::install(Path::new(bundle),Path::new(store),&identity,&key)?;
            json(&bundle.manifest)
        }
        ("resource-status", [store]) => {
            let key=aetheris::companion::manifest::public_key()?;
            json(&game_data::resources::active(Path::new(store),&key)?.map(|b| b.manifest))
        }
        ("status", [store]) => {
            match Catalog::active(Path::new(store))? {
                Some(catalog) => json(&serde_json::json!({ "client": catalog.build.inventory.client, "fingerprint": catalog.build.inventory.fingerprint, "schema_set": catalog.build.schema_set, "tables": catalog.build.tables })),
                None => json(&serde_json::json!({ "active": false })),
            }
        }
        _ => Err("Usage: aetheris-game-data extract-data <paks> <key-file> <output>\n       aetheris-game-data decrypt-data <extraction> <manifest-material-file> <output>\n       aetheris-game-data probe-iostore <utoc> <key-file> [expected-toc-sha256]\n       aetheris-game-data inspect <client-directory>\n       aetheris-game-data inspect-steam-manifest <appmanifest.acf>\n       aetheris-game-data discover [search-root]...\n       aetheris-game-data decode <schema.json> <identity.json> <decrypted.bin>\n       aetheris-game-data decode-cached <schema.json> <identity.json> <decrypted.bin> <store>\n       aetheris-game-data import-reviewed <build.json> <client-directory> <store>\n       aetheris-game-data status <store>\n       aetheris-game-data purge-cache <store>\n       aetheris-game-data install-resources <bundle> <resource-store> <identity.json>\n       aetheris-game-data resource-status <resource-store>".into()),
    }
}
