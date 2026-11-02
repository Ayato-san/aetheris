use super::{
    Result, error,
    manifest::{self, Manifest},
    policy::Channel,
};
use reqwest::blocking::Client;
use semver::Version;
use serde::Deserialize;
use std::{
    fs,
    io::{Read, Write},
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

#[derive(Clone, Debug, Deserialize)]
pub struct Asset {
    pub name: String,
    pub browser_download_url: String,
}
#[derive(Clone, Debug, Deserialize)]
pub struct Release {
    pub tag_name: String,
    pub draft: bool,
    pub prerelease: bool,
    pub assets: Vec<Asset>,
}
impl Release {
    pub fn version(&self) -> Option<Version> {
        Version::parse(self.tag_name.trim_start_matches('v')).ok()
    }
    fn asset(&self) -> Result<&str> {
        self.assets
            .iter()
            .find(|asset| asset.name == "aetheris-windows-x86_64.zip")
            .map(|asset| asset.browser_download_url.as_str())
            .ok_or_else(|| error("Archive de distribution absente."))
    }
}

pub fn repository() -> Result<&'static str> {
    let repo = option_env!("AETHERIS_REPOSITORY").unwrap_or("Ayato-san/Aetheris");
    let parts: Vec<_> = repo.split('/').collect();
    if parts.len() != 2
        || parts.iter().any(|part| {
            part.is_empty()
                || !part
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
        })
    {
        return Err(error("Dépôt GitHub de distribution non configuré."));
    }
    Ok(repo)
}
fn client() -> Result<Client> {
    Ok(Client::builder()
        .https_only(true)
        .user_agent("Aetheris updater")
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(180))
        .build()?)
}
pub fn select(
    releases: impl IntoIterator<Item = Release>,
    channel: Channel,
    installed: &Version,
) -> Option<Release> {
    releases
        .into_iter()
        .filter(|r| !r.draft && (channel == Channel::Prerelease || !r.prerelease))
        .filter(|r| {
            r.version()
                .is_some_and(|v| channel.accepts(&v) && v.cmp_precedence(installed).is_gt())
        })
        .max_by(|a, b| a.version().unwrap().cmp_precedence(&b.version().unwrap()))
}
pub fn latest(channel: Channel, installed: &Version) -> Result<Option<Release>> {
    let client = client()?;
    let mut releases = Vec::new();
    for page in 1..=100 {
        let batch: Vec<Release> = client
            .get(format!(
                "https://api.github.com/repos/{}/releases?per_page=100&page={page}",
                repository()?
            ))
            .send()?
            .error_for_status()?
            .json()?;
        let done = batch.len() < 100;
        releases.extend(batch);
        if done {
            return Ok(select(releases, channel, installed));
        }
    }
    Err(error(
        "Liste de releases trop longue ; vérification incomplète.",
    ))
}
pub fn exact(version: &Version) -> Result<Release> {
    Ok(client()?
        .get(format!(
            "https://api.github.com/repos/{}/releases/tags/v{version}",
            repository()?
        ))
        .send()?
        .error_for_status()?
        .json()?)
}

/// Download cancellation is checked between bounded reads; only authenticated
/// program entries are extracted, never arbitrary ZIP paths or personal data.
pub fn download(
    release: &Release,
    destination: &Path,
    cancel: Arc<AtomicBool>,
) -> Result<Manifest> {
    fs::create_dir_all(destination)?;
    let archive_path = destination.join("download.zip");
    let mut response = client()?.get(release.asset()?).send()?.error_for_status()?;
    let mut archive = fs::File::create(&archive_path)?;
    let mut total = 0_u64;
    let mut buffer = [0; 65536];
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err(error("Téléchargement annulé."));
        }
        let count = response.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        total += count as u64;
        if total > 1024 * 1024 * 1024 {
            return Err(error("Archive trop volumineuse."));
        }
        archive.write_all(&buffer[..count])?;
    }
    archive.sync_all()?;
    drop(archive);
    let mut zip = zip::ZipArchive::new(fs::File::open(&archive_path)?)?;
    for name in [manifest::MANIFEST, manifest::SIGNATURE] {
        extract(&mut zip, name, destination, 65536)?;
    }
    let key = manifest::public_key()?;
    let manifest = manifest::authenticate(destination, &key)?;
    if Some(manifest.version.clone()) != release.version() {
        return Err(error("La version signée diffère du tag GitHub."));
    }
    for name in manifest.files.keys() {
        extract(&mut zip, name, destination, 512 * 1024 * 1024)?;
    }
    manifest::verify(destination, &key)
}
fn extract(
    zip: &mut zip::ZipArchive<fs::File>,
    name: &str,
    destination: &Path,
    limit: u64,
) -> Result<()> {
    let mut entry = zip.by_name(name)?;
    if entry.size() > limit
        || entry.is_dir()
        || entry.unix_mode().is_some_and(|m| m & 0o170000 == 0o120000)
    {
        return Err(error("Entrée ZIP invalide."));
    }
    let mut output = fs::File::create(destination.join(name))?;
    std::io::copy(&mut entry, &mut output)?;
    output.sync_all()?;
    Ok(())
}
