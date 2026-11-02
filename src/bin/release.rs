//! Offline release tooling. The secret is read only while signing, never logged.
use aetheris::companion::{
    Result, error,
    manifest::{self, MANIFEST, Manifest, PROGRAMS, SIGNATURE},
};
use ed25519_dalek::{Signer, SigningKey};
use std::{collections::BTreeMap, fs, io::Write, path::Path};

fn main() {
    if let Err(cause) = run() {
        eprintln!("{cause}");
        std::process::exit(1);
    }
}
fn run() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("keygen") if args.len() == 2 => {
            let path = Path::new(&args[1]);
            let parent = fs::canonicalize(
                path.parent()
                    .filter(|p| !p.as_os_str().is_empty())
                    .unwrap_or(Path::new(".")),
            )?;
            if parent
                .ancestors()
                .any(|directory| directory.join(".git").exists())
            {
                return Err(error(
                    "La clé privée doit être créée hors de tout dépôt Git.",
                ));
            }
            let mut seed = [0; 32];
            getrandom::fill(&mut seed).map_err(|e| error(e.to_string()))?;
            let key = SigningKey::from_bytes(&seed);
            let mut options = fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options.open(&args[1])?;
            file.write_all(hex::encode(seed).as_bytes())?;
            file.sync_all()?;
            fs::write(
                format!("{}.pub", args[1]),
                hex::encode(key.verifying_key().as_bytes()),
            )?;
            println!(
                "Clé privée créée. Conservez-la hors du dépôt. La clé publique est dans le fichier .pub."
            );
        }
        Some("package") if args.len() == 3 => package(Path::new(&args[1]), Path::new(&args[2]))?,
        _ => {
            return Err(error(
                "Usage : aetheris-release keygen <fichier-privé> | package <dossier-binaires> <archive.zip>",
            ));
        }
    }
    Ok(())
}
fn package(directory: &Path, output: &Path) -> Result<()> {
    let encoded = std::env::var("AETHERIS_SIGNING_KEY")
        .map_err(|_| error("Secret AETHERIS_SIGNING_KEY absent."))?;
    let seed: [u8; 32] = hex::decode(encoded.trim())?
        .try_into()
        .map_err(|_| error("Clé privée Ed25519 invalide."))?;
    let key = SigningKey::from_bytes(&seed);
    if key.verifying_key() != manifest::public_key()? {
        return Err(error(
            "La clé privée ne correspond pas à la clé publique intégrée.",
        ));
    }
    let mut files = BTreeMap::new();
    for name in PROGRAMS {
        files.insert(name.to_owned(), manifest::hash(&directory.join(name))?);
    }
    let manifest = Manifest {
        version: semver::Version::parse(env!("CARGO_PKG_VERSION"))?,
        target: "x86_64-pc-windows-msvc".into(),
        files,
    };
    let bytes = serde_json::to_vec_pretty(&manifest)?;
    fs::write(directory.join(MANIFEST), &bytes)?;
    fs::write(directory.join(SIGNATURE), key.sign(&bytes).to_bytes())?;
    manifest::verify(directory, &key.verifying_key())?;
    let mut zip = zip::ZipWriter::new(fs::File::create(output)?);
    for name in PROGRAMS.into_iter().chain([MANIFEST, SIGNATURE]) {
        zip.start_file(
            name,
            zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated),
        )?;
        let mut input = fs::File::open(directory.join(name))?;
        std::io::copy(&mut input, &mut zip)?;
    }
    zip.finish()?.sync_all()?;
    Ok(())
}
