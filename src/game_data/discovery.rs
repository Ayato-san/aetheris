//! Local installation discovery and evidence from Steam's on-disk metadata.
//! No launcher or network is required; a copied directory is always selectable.
use super::{SourceMetadata, discover};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Default, Serialize)]
pub struct Discovery {
    pub installations: Vec<PathBuf>,
    pub problems: Vec<String>,
}

/// Search configured roots and all libraries declared by a Steam installation.
/// Missing optional default locations are ignored; failures to read existing
/// metadata are surfaced and never mistaken for an empty library.
pub fn installations(roots: &[PathBuf]) -> Discovery {
    let mut report = Discovery::default();
    let mut searches = std::collections::BTreeSet::new();
    for root in roots {
        searches.insert(root.clone());
        searches.insert(root.join("steamapps/common"));
        let folders = root.join("steamapps/libraryfolders.vdf");
        match read_document(&folders) {
            Ok((_, document)) => {
                if let Some(folders) = object(&document, "libraryfolders") {
                    for value in folders.values() {
                        let path = match value {
                            Node::Object(library) => string(library, "path"),
                            Node::String(path) => Some(path.as_str()),
                        };
                        if let Some(path) = path {
                            searches.insert(PathBuf::from(path).join("steamapps/common"));
                        }
                    }
                } else {
                    report.problems.push(format!(
                        "{}: missing libraryfolders object",
                        folders.display()
                    ));
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => (),
            Err(error) => report
                .problems
                .push(format!("{}: {error}", folders.display())),
        }
    }
    let mut found = std::collections::BTreeSet::new();
    for root in searches {
        match fs::read_dir(&root) {
            Ok(entries) => {
                for entry in entries {
                    if let Err(error) = entry {
                        report.problems.push(format!("{}: {error}", root.display()));
                    }
                }
                found.extend(discover(&[root]));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => (),
            Err(error) => report.problems.push(format!("{}: {error}", root.display())),
        }
    }
    report.installations = found.into_iter().collect();
    report.problems.sort();
    report.problems.dedup();
    report
}

pub fn default_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(paths) = std::env::var_os("AETHERIS_GAME_ROOTS") {
        roots.extend(std::env::split_paths(&paths));
    }
    #[cfg(windows)]
    {
        if let Some(root) = steam_registry_path() {
            roots.push(root);
        }
        for variable in ["ProgramFiles(x86)", "ProgramFiles"] {
            if let Some(root) = std::env::var_os(variable) {
                roots.push(PathBuf::from(root).join("Steam"));
            }
        }
    }
    #[cfg(not(windows))]
    {
        if let Some(home) = std::env::var_os("HOME") {
            let home = PathBuf::from(home);
            roots.push(home.join(".local/share/Steam"));
            roots.push(home.join(".steam/steam"));
        }
    }
    roots
}

#[cfg(windows)]
fn steam_registry_path() -> Option<PathBuf> {
    use windows_sys::Win32::System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_SZ, RegGetValueW};
    let subkey: Vec<_> = "Software\\Valve\\Steam\0".encode_utf16().collect();
    let name: Vec<_> = "SteamPath\0".encode_utf16().collect();
    let mut buffer = vec![0u16; 32768];
    let mut size = (buffer.len() * 2) as u32;
    // RegGetValueW writes at most `size` bytes to the owned UTF-16 buffer.
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            name.as_ptr(),
            RRF_RT_REG_SZ,
            std::ptr::null_mut(),
            buffer.as_mut_ptr().cast(),
            &mut size,
        )
    };
    if status != 0 || size < 2 || size as usize > buffer.len() * 2 {
        return None;
    }
    buffer.truncate(size as usize / 2);
    if buffer.last() == Some(&0) {
        buffer.pop();
    }
    Some(PathBuf::from(String::from_utf16(&buffer).ok()?))
}

/// Only a manifest whose installdir matches this exact common/<directory>
/// contributes a build ID. Region remains unknown: Steam's build ID proves no
/// regional identity. Outside steamapps/common copied clients need no manifest.
pub(super) fn steam_metadata(root: &Path) -> io::Result<Vec<SourceMetadata>> {
    let mut metadata = Vec::new();
    let Some(common) = root
        .parent()
        .filter(|p| p.file_name().is_some_and(|s| s == "common"))
    else {
        return Ok(metadata);
    };
    let Some(steamapps) = common
        .parent()
        .filter(|p| p.file_name().is_some_and(|s| s == "steamapps"))
    else {
        return Ok(metadata);
    };
    for entry in fs::read_dir(steamapps)? {
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name
            .to_str()
            .filter(|s| s.starts_with("appmanifest_") && s.ends_with(".acf"))
        else {
            continue;
        };
        let (bytes, document) = read_document(&entry.path())?;
        let app = object(&document, "AppState").ok_or_else(|| invalid("missing AppState"))?;
        let Some(directory) = string(app, "installdir") else {
            continue;
        };
        if Path::new(directory).components().count() != 1 || directory == "." || directory == ".." {
            return Err(invalid("invalid installdir"));
        }
        let Some(client_name) = root.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if directory != client_name {
            continue;
        }
        let version = string(app, "buildid")
            .filter(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
            .map(str::to_owned);
        metadata.push(SourceMetadata {
            path: format!("../../{name}"),
            sha256: hex::encode(Sha256::digest(bytes)),
            kind: "steam_appmanifest".into(),
            version,
            region: None,
        });
    }
    metadata.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(metadata)
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[derive(Debug)]
enum Node {
    String(String),
    Object(BTreeMap<String, Node>),
}
fn object<'a>(nodes: &'a BTreeMap<String, Node>, key: &str) -> Option<&'a BTreeMap<String, Node>> {
    nodes.iter().find_map(|(name, value)| {
        if name.eq_ignore_ascii_case(key) {
            if let Node::Object(value) = value {
                Some(value)
            } else {
                None
            }
        } else {
            None
        }
    })
}
fn string<'a>(nodes: &'a BTreeMap<String, Node>, key: &str) -> Option<&'a str> {
    nodes.iter().find_map(|(name, value)| {
        if name.eq_ignore_ascii_case(key) {
            if let Node::String(value) = value {
                Some(value.as_str())
            } else {
                None
            }
        } else {
            None
        }
    })
}

fn read_document(path: &Path) -> io::Result<(Vec<u8>, BTreeMap<String, Node>)> {
    use std::io::Read;
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(4 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 4 * 1024 * 1024 {
        return Err(invalid("Steam metadata exceeds limit"));
    }
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| invalid("invalid UTF-8 metadata"))?
        .trim_start_matches('\u{feff}');
    let mut parser = Vdf {
        text: text.as_bytes(),
        offset: 0,
        budget: 100_000,
    };
    let document = parser.nodes(false, 0)?;
    Ok((bytes, document))
}

struct Vdf<'a> {
    text: &'a [u8],
    offset: usize,
    budget: usize,
}
impl Vdf<'_> {
    fn skip(&mut self) {
        loop {
            while self
                .text
                .get(self.offset)
                .is_some_and(u8::is_ascii_whitespace)
            {
                self.offset += 1;
            }
            if self.text.get(self.offset..self.offset + 2) == Some(b"//") {
                while self.text.get(self.offset).is_some_and(|b| *b != b'\n') {
                    self.offset += 1;
                }
            } else {
                break;
            }
        }
    }
    fn quoted(&mut self) -> io::Result<String> {
        self.skip();
        if self.text.get(self.offset) != Some(&b'"') {
            return Err(invalid("expected quoted VDF string"));
        }
        self.offset += 1;
        let mut bytes = Vec::new();
        loop {
            let byte = *self
                .text
                .get(self.offset)
                .ok_or_else(|| invalid("unterminated VDF string"))?;
            self.offset += 1;
            match byte {
                b'"' => break,
                b'\\' => {
                    let next = *self
                        .text
                        .get(self.offset)
                        .ok_or_else(|| invalid("truncated VDF escape"))?;
                    self.offset += 1;
                    match next {
                        b'\\' | b'"' => bytes.push(next),
                        b'n' => bytes.push(b'\n'),
                        b't' => bytes.push(b'\t'),
                        _ => {
                            bytes.push(b'\\');
                            bytes.push(next);
                        }
                    }
                }
                0 => return Err(invalid("NUL in VDF string")),
                _ => bytes.push(byte),
            }
        }
        String::from_utf8(bytes).map_err(|_| invalid("invalid UTF-8 VDF string"))
    }
    fn nodes(&mut self, nested: bool, depth: usize) -> io::Result<BTreeMap<String, Node>> {
        if depth > 32 {
            return Err(invalid("VDF nesting exceeds limit"));
        }
        let mut nodes = BTreeMap::new();
        loop {
            self.skip();
            match self.text.get(self.offset) {
                None if !nested => return Ok(nodes),
                Some(b'}') if nested => {
                    self.offset += 1;
                    return Ok(nodes);
                }
                None | Some(b'}') => return Err(invalid("unbalanced VDF object")),
                _ => (),
            }
            if self.budget == 0 {
                return Err(invalid("VDF complexity exceeds limit"));
            }
            self.budget -= 1;
            let key = self.quoted()?;
            self.skip();
            let value = if self.text.get(self.offset) == Some(&b'{') {
                self.offset += 1;
                Node::Object(self.nodes(true, depth + 1)?)
            } else {
                Node::String(self.quoted()?)
            };
            if nodes.keys().any(|k: &String| k.eq_ignore_ascii_case(&key)) {
                return Err(invalid("duplicate VDF key"));
            }
            nodes.insert(key, value);
        }
    }
}
