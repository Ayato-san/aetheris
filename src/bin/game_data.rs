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
        ("inspect", [directory]) => json(&game_data::inventory(Path::new(directory))?),
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
        ("status", [store]) => {
            match Catalog::active(Path::new(store))? {
                Some(catalog) => json(&serde_json::json!({ "client": catalog.build.inventory.client, "fingerprint": catalog.build.inventory.fingerprint, "schema_set": catalog.build.schema_set, "tables": catalog.build.tables })),
                None => json(&serde_json::json!({ "active": false })),
            }
        }
        _ => Err("Usage: aetheris-game-data inspect <client-directory>\n       aetheris-game-data discover [search-root]...\n       aetheris-game-data decode <schema.json> <identity.json> <decrypted.bin>\n       aetheris-game-data decode-cached <schema.json> <identity.json> <decrypted.bin> <store>\n       aetheris-game-data import-reviewed <build.json> <client-directory> <store>\n       aetheris-game-data status <store>\n       aetheris-game-data purge-cache <store>".into()),
    }
}
