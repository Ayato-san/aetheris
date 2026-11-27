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

fn bounded_input(path: &Path) -> catalog::Result<Vec<u8>> {
    use std::io::Read;
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(64 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 64 * 1024 * 1024 {
        return Err("Probe input exceeds 64 MiB".into());
    }
    Ok(bytes)
}

fn run() -> catalog::Result<()> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let command = args.first().and_then(|a| a.to_str()).unwrap_or("");
    match (command, &args[usize::from(!args.is_empty())..]) {
        ("draft-schema-set", [mapping, prepared, selection, output]) => {
            if Path::new(output).exists() { return Err("Schema draft requires a new output file".into()); }
            let mapping = game_data::usmap::Mapping::parse(&bounded_input(Path::new(mapping))?)?;
            let selection = serde_json::from_slice(&game_data::schema_set::read_bounded(Path::new(selection), 16 * 1024 * 1024)?)?;
            let mut progress = |_| {};
            let cancel = AtomicBool::new(false);
            let set = game_data::schema_set::draft(&mapping, Path::new(prepared), "candidate", &selection,
                &mut game_data::reader::Control::new(&cancel, &mut progress))?;
            aetheris::engine_ini::storage::atomic_replace(Path::new(output), &serde_json::to_vec_pretty(&set)?, None)?;
            json(&serde_json::json!({"entries":set.entries.len(),"compatible_clients":set.compatible_clients,"output":Path::new(output).display().to_string()}))
        }
        ("decode-schema-set", [schema, mapping, prepared, identity, output]) => {
            let set = game_data::schema_set::SchemaSet::parse(&game_data::schema_set::read_bounded(Path::new(schema),16 * 1024 * 1024)?)?;
            let mapping = game_data::usmap::Mapping::parse(&bounded_input(Path::new(mapping))?)?;
            let identity = if identity == "explore" { None } else { Some(serde_json::from_slice(&bounded_input(Path::new(identity))?)?) };
            let mut progress = |_| {};
            let cancel = AtomicBool::new(false);
            let report = game_data::schema_set::decode_prepared(&set, &mapping, Path::new(prepared), identity.as_ref(), Path::new(output),
                &mut game_data::reader::Control::new(&cancel, &mut progress))?;
            json(&report)?;
            if report.state != "complete" { return Err("Some schema entries were rejected; see decoding.json".into()); }
            Ok(())
        }
        ("export-catalogue", [mapping, tables, english, localized, manifest, rest @ ..]) if rest.len() <= 3 => {
            let output = rest.first().map(|p| Path::new(p).to_owned()).map(Ok).unwrap_or_else(game_data::database_path)?;
            let locale = rest.get(1).map(|v| v.to_str().ok_or("Locale must be UTF-8")).transpose()?.unwrap_or("fr-FR");
            json(&game_data::target_export::export(Path::new(mapping), Path::new(tables), Path::new(english), Path::new(localized), Path::new(manifest), &output, locale, rest.get(2).map(Path::new))?)
        }
        ("probe-skills", [mapping, tables, output]) => {
            if Path::new(output).exists() {
                let destination = fs::canonicalize(output)?;
                let mut inputs = game_data::skills::source_paths(Path::new(tables));
                inputs.push(Path::new(mapping).to_owned());
                for input in inputs {
                    if destination == fs::canonicalize(input)? {
                        return Err("Skill probe output must not replace its input".into());
                    }
                }
            }
            let mapping = game_data::usmap::Mapping::parse(&bounded_input(Path::new(mapping))?)?;
            let report = game_data::skills::probe(&mapping, Path::new(tables))?;
            aetheris::engine_ini::storage::atomic_replace(Path::new(output), &serde_json::to_vec(&report)?, None)?;
            json(&serde_json::json!({"compatibility_verified":false,"tables":report.tables.len(),"rows":report.tables.values().map(|t|t.layout.rows.len()).sum::<usize>(),"relation_checks":report.relations.len(),"missing_references":report.relations.iter().filter(|r|!r.target_may_be_partial).map(|r|r.missing).sum::<usize>(),"unmatched_type_metadata":report.relations.iter().filter(|r|r.target_may_be_partial).map(|r|r.missing).sum::<usize>(),"output":Path::new(output).display().to_string()}))
        }
        ("probe-table" | "probe-map", [mapping, row_struct, payload, output]) => {
            if Path::new(output).exists() {
                let output = fs::canonicalize(output)?;
                for input in [mapping, payload] {
                    if output == fs::canonicalize(input)? {
                        return Err("Probe output must not replace its input".into());
                    }
                }
            }
            let row_struct = row_struct.to_str().ok_or("Row structure must be UTF-8")?;
            let mapping = game_data::usmap::Mapping::parse(&bounded_input(Path::new(mapping))?)?;
            let bytes = bounded_input(Path::new(payload))?;
            let report = if command == "probe-map" { mapping.probe_map(row_struct, &bytes)? } else { mapping.probe_table(row_struct, &bytes)? };
            aetheris::engine_ini::storage::atomic_replace(Path::new(output), &serde_json::to_vec(&report)?, None)?;
            json(&serde_json::json!({"compatibility_verified":false,"row_struct":row_struct,"rows":report.layout.rows.len(),"columns":report.layout.column_hashes.len(),"mapping_sha256":report.mapping_sha256,"payload_sha256":report.payload_sha256,"output":Path::new(output).display().to_string()}))
        }
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
        ("read-client",[paks,archive_key,manifest_material,output])=> {
            let archive_key=game_data::archives::read_key(Path::new(archive_key))?;
            let material=game_data::archives::read_key(Path::new(manifest_material))?;
            let cancel=AtomicBool::new(false);
            let mut progress=|p:game_data::reader::Progress| {if p.completed.is_multiple_of(100)||p.completed==p.total||p.path.is_none(){eprintln!("{:?}: {}/{}",p.phase,p.completed,p.total);}};
            let report=game_data::reader::read_client(Path::new(paks),archive_key,&material,Path::new(output),&mut game_data::reader::Control::new(&cancel,&mut progress))?;
            let complete=report.state=="complete";json(&report)?;
            if !complete {return Err("Client reading is partial; see reader.json and stage reports".into());}Ok(())
        }
        ("extract-world-maps", [paks, mapping, key, output]) => {
            let report=game_data::map_textures::extract(Path::new(paks),Path::new(mapping),Path::new(key),Path::new(output))?;
            let failed=report["level_errors"]!=0 || report["texture_errors"]!=0;
            json(&report)?;
            if failed {return Err("Some map assets could not be extracted; see world-maps.json".into());}
            Ok(())
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
        _ => Err("Usage: aetheris-game-data draft-schema-set <mapping.usmap> <prepared-directory> <selection.json> <new-output.json>\n       aetheris-game-data decode-schema-set <schema-set.json> <mapping.usmap> <prepared-directory> <identity.json|explore> <fresh-output-directory>\n       aetheris-game-data export-catalogue <mapping.usmap> <decrypted-table-directory> <en-US.json> <localized.json> <appmanifest.acf> [output [locale [image-directory]]]\n       aetheris-game-data probe-skills <mapping.usmap> <decrypted-table-directory> <output.json>\n       aetheris-game-data read-client <paks> <archive-key-file> <manifest-material-file> <fresh-work-directory>\n       aetheris-game-data extract-data <paks> <key-file> <output>\n       aetheris-game-data extract-world-maps <paks> <mapping.usmap> <key-file> <output>\n       aetheris-game-data decrypt-data <extraction> <manifest-material-file> <output>\n       aetheris-game-data probe-table <mapping.usmap> <row-struct> <decrypted.bin> <output.json>\n       aetheris-game-data probe-map <mapping.usmap> <MapData|AionWorldMapExportInfo> <decrypted.bin> <output.json>\n       aetheris-game-data probe-iostore <utoc> <key-file> [expected-toc-sha256]\n       aetheris-game-data inspect <client-directory>\n       aetheris-game-data inspect-steam-manifest <appmanifest.acf>\n       aetheris-game-data discover [search-root]...\n       aetheris-game-data decode <schema.json> <identity.json> <decrypted.bin>\n       aetheris-game-data decode-cached <schema.json> <identity.json> <decrypted.bin> <store>\n       aetheris-game-data import-reviewed <build.json> <client-directory> <store>\n       aetheris-game-data status <store>\n       aetheris-game-data purge-cache <store>\n       aetheris-game-data install-resources <bundle> <resource-store> <identity.json>\n       aetheris-game-data resource-status <resource-store>".into()),
    }
}
