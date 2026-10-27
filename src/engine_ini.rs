//! Structure-preserving editing of Unreal Engine `.ini` files.

use std::{
    env, fmt, fs,
    path::{Path, PathBuf},
};

pub mod hardware;
pub mod numeric;
pub mod settings;
pub mod storage;

const AION2_ENGINE_INI: [&str; 5] = ["AION2", "Saved_Steam", "Config", "Windows", "Engine.ini"];

/// Finds Aion 2's `Engine.ini` for the current Windows user, then for the
/// other profiles installed on the same machine.
///
/// Native Windows paths and Windows drives mounted by WSL are supported.
pub fn discover_engine_ini() -> Option<PathBuf> {
    let local_app_data = env::var_os("LOCALAPPDATA").map(PathBuf::from);
    let mut users_roots = Vec::new();

    if let Some(local_app_data) = local_app_data.as_deref()
        && let Some(users_root) = local_app_data
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
    {
        users_roots.push(users_root.to_owned());
    }

    if cfg!(windows) {
        let system_drive = env::var_os("SystemDrive").unwrap_or_else(|| "C:".into());
        users_roots.push(PathBuf::from(system_drive).join("Users"));
    }

    let wsl_users = PathBuf::from("/mnt/c/Users");
    if wsl_users.is_dir() {
        users_roots.push(wsl_users);
    }

    discover_engine_ini_in(local_app_data.as_deref(), &users_roots)
}

fn discover_engine_ini_in(
    local_app_data: Option<&Path>,
    users_roots: &[PathBuf],
) -> Option<PathBuf> {
    if let Some(path) = local_app_data.map(engine_ini_below)
        && path.is_file()
    {
        return Some(path);
    }

    for users_root in users_roots {
        let Ok(entries) = fs::read_dir(users_root) else {
            continue;
        };
        let mut profiles = entries
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_dir())
            .collect::<Vec<_>>();
        profiles.sort();

        for profile in profiles {
            let candidate = engine_ini_below(&profile.join("AppData").join("Local"));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    None
}

fn engine_ini_below(local_app_data: &Path) -> PathBuf {
    AION2_ENGINE_INI
        .iter()
        .fold(local_app_data.to_owned(), |path, part| path.join(part))
}

/// Checks whether a manually selected path is an existing `Engine.ini` file.
pub fn validate_engine_ini_path(path: PathBuf) -> Result<PathBuf, &'static str> {
    if !path.is_file() {
        return Err("Le fichier sélectionné n’existe pas.");
    }

    if !path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("Engine.ini"))
    {
        return Err("Le fichier sélectionné doit s’appeler Engine.ini.");
    }

    Ok(path)
}

/// A parsed `Engine.ini` document.
///
/// Unknown lines, comments, blank lines and ordering are preserved when the
/// document is rendered again. Property lookup is case-insensitive, as it is
/// in Unreal configuration files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineIni {
    lines: Vec<Line>,
    newline: String,
    trailing_newline: bool,
    bom: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Line {
    Section(String),
    Property {
        section: String,
        key: String,
        value: String,
        source: Option<String>,
    },
    Raw(String),
}

impl EngineIni {
    /// Parses a document while preserving content that Aetheris does not edit.
    pub fn parse(input: &str) -> Self {
        let bom = input.starts_with('\u{feff}');
        let input = input.trim_start_matches('\u{feff}');
        let mut section = String::new();
        let lines = input
            .lines()
            .map(|source| {
                let trimmed = source.trim();

                if let Some(name) = trimmed
                    .strip_prefix('[')
                    .and_then(|value| value.strip_suffix(']'))
                {
                    section = name.trim().to_owned();
                    Line::Section(source.to_owned())
                } else if !trimmed.starts_with([';', '#']) {
                    match source.split_once('=') {
                        Some((key, value)) if !key.trim().is_empty() => Line::Property {
                            section: section.clone(),
                            key: key.trim().to_owned(),
                            value: value.trim().to_owned(),
                            source: Some(source.to_owned()),
                        },
                        _ => Line::Raw(source.to_owned()),
                    }
                } else {
                    Line::Raw(source.to_owned())
                }
            })
            .collect();

        Self {
            lines,
            newline: if input.contains("\r\n") { "\r\n" } else { "\n" }.into(),
            trailing_newline: input.ends_with('\n'),
            bom,
        }
    }

    /// Returns the last value assigned to a property in a section.
    pub fn get(&self, section: &str, key: &str) -> Option<&str> {
        self.lines.iter().rev().find_map(|line| match line {
            Line::Property {
                section: candidate_section,
                key: candidate_key,
                value,
                ..
            } if candidate_section.eq_ignore_ascii_case(section)
                && candidate_key.eq_ignore_ascii_case(key) =>
            {
                Some(value.as_str())
            }
            _ => None,
        })
    }

    /// Updates the last matching property, or appends it to the section.
    /// The section is created when it does not exist.
    pub fn set(&mut self, section: &str, key: &str, value: impl Into<String>) {
        let value = value.into();

        if let Some(Line::Property {
            value: current_value,
            source,
            ..
        }) = self.lines.iter_mut().rev().find(|line| {
            matches!(
                line,
                Line::Property {
                    section: candidate_section,
                    key: candidate_key,
                    ..
                } if candidate_section.eq_ignore_ascii_case(section)
                    && candidate_key.eq_ignore_ascii_case(key)
            )
        }) {
            *current_value = value;
            *source = None;
            return;
        }

        let insertion_index = self.section_end(section);
        let property = Line::Property {
            section: section.to_owned(),
            key: key.to_owned(),
            value,
            source: None,
        };

        if let Some(index) = insertion_index {
            self.lines.insert(index, property);
        } else {
            if self
                .lines
                .last()
                .is_some_and(|line| !matches!(line, Line::Raw(raw) if raw.is_empty()))
            {
                self.lines.push(Line::Raw(String::new()));
            }
            self.lines.push(Line::Section(format!("[{section}]")));
            self.lines.push(property);
        }
    }

    /// Restores just the requested keys, including the reference's spelling,
    /// directly below the section header. Unmanaged content is preserved.
    pub(crate) fn restore_properties_from(
        &mut self,
        baseline: &Self,
        section: &str,
        keys: &[&str],
    ) {
        for key in keys {
            self.remove(section, key);
        }
        let properties: Vec<_> = keys
            .iter()
            .filter_map(|key| {
                baseline
                    .lines
                    .iter()
                    .rev()
                    .find(|line| {
                        matches!(line,
                Line::Property {section: s, key: k, ..}
                if s.eq_ignore_ascii_case(section) && k.eq_ignore_ascii_case(key))
                    })
                    .cloned()
            })
            .collect();
        if properties.is_empty() {
            return;
        }
        if let Some(start) = self.lines.iter().position(|line| matches!(line,
            Line::Section(raw) if section_name(raw).is_some_and(|s| s.eq_ignore_ascii_case(section)))) {
            let end = self.lines[start + 1..].iter().position(|line| matches!(line, Line::Section(_)))
                .map_or(self.lines.len(), |offset| start + 1 + offset);
            if self.lines[start + 1..end].iter().all(|line| matches!(line, Line::Raw(raw) if raw.trim().is_empty())) {
                self.lines.drain(start + 1..end);
            }
            self.lines.splice(start + 1..start + 1, properties);
        } else {
            if self.lines.last().is_some_and(|line| !matches!(line, Line::Raw(raw) if raw.trim().is_empty())) {
                self.lines.push(Line::Raw(String::new()));
            }
            self.lines.push(Line::Section(format!("[{section}]")));
            self.lines.extend(properties);
        }
    }

    /// Reference paths are documentary only; never capture other local values.
    pub(crate) fn copy_reference_paths_from(&mut self, other: &Self) {
        let paths: Vec<_> = other
            .lines
            .iter()
            .filter(|line| {
                matches!(line,
            Line::Property {section, key, ..}
            if section.eq_ignore_ascii_case("Core.System") && key.eq_ignore_ascii_case("Paths"))
            })
            .cloned()
            .collect();
        if paths.is_empty() {
            return;
        }
        let current: Vec<_> = self
            .lines
            .iter()
            .filter(|line| {
                matches!(line,
            Line::Property {section, key, ..}
            if section.eq_ignore_ascii_case("Core.System") && key.eq_ignore_ascii_case("Paths"))
            })
            .collect();
        if current.iter().copied().eq(paths.iter()) {
            return;
        }
        self.remove("Core.System", "Paths");
        let index = self.lines.iter().position(|line| matches!(line,
            Line::Section(raw) if section_name(raw).is_some_and(|s| s.eq_ignore_ascii_case("Core.System"))))
            .expect("reference contains Core.System") + 1;
        self.lines.splice(index..index, paths);
    }

    /// Removes every occurrence, including duplicate assignments in repeated sections.
    pub fn remove(&mut self, section: &str, key: &str) {
        self.lines.retain(|line| {
            !matches!(line, Line::Property { section: s, key: k, .. }
            if s.eq_ignore_ascii_case(section) && k.eq_ignore_ascii_case(key))
        });
    }

    /// Removes only empty occurrences of the named section. Comments and any
    /// other nonblank content keep their section, including repeated sections.
    pub(crate) fn remove_empty_sections(&mut self, section: &str) {
        let mut index = 0;
        while index < self.lines.len() {
            let matches = matches!(&self.lines[index], Line::Section(raw)
                if section_name(raw).is_some_and(|name| name.eq_ignore_ascii_case(section)));
            if !matches {
                index += 1;
                continue;
            }
            let end = self.lines[index + 1..]
                .iter()
                .position(|line| matches!(line, Line::Section(_)))
                .map_or(self.lines.len(), |offset| index + 1 + offset);
            if self.lines[index + 1..end]
                .iter()
                .all(|line| matches!(line, Line::Raw(raw) if raw.trim().is_empty()))
            {
                self.lines.drain(index..end);
            } else {
                index = end;
            }
        }
    }

    pub(crate) fn remove_marker(&mut self, marker: &str) {
        self.lines
            .retain(|line| !matches!(line, Line::Raw(raw) if raw.trim() == marker));
    }

    /// Moves only this category's existing properties into one labeled block.
    pub(crate) fn label_block(&mut self, section: &str, keys: &[&str], marker: &str) {
        let belongs = |line: &Line| {
            matches!(line, Line::Property {section: s, key, ..}
            if s.eq_ignore_ascii_case(section) && keys.iter().any(|k| k.eq_ignore_ascii_case(key)))
        };
        // Preserve the raw representation (and duplicate assignments) of keys
        // the player has not explicitly changed, including unrecognized values.
        let properties: Vec<_> = self
            .lines
            .iter()
            .filter(|line| belongs(line))
            .cloned()
            .collect();
        self.remove_marker(marker);
        self.lines.retain(|line| !belongs(line));
        if properties.is_empty() {
            return;
        }
        let index = self
            .section_end(section)
            .expect("managed properties have a section");
        self.lines.splice(
            index..index,
            std::iter::once(Line::Raw(marker.to_owned())).chain(properties),
        );
        self.space_category_blocks(section);
    }

    /// Keeps one blank line between labeled categories in the same section.
    pub(crate) fn space_category_blocks(&mut self, section: &str) {
        let mut inside = false;
        let mut previous_category = false;
        for line in std::mem::take(&mut self.lines) {
            if let Line::Section(raw) = &line {
                inside = section_name(raw).is_some_and(|s| s.eq_ignore_ascii_case(section));
                previous_category = false;
            }
            if inside && matches!(&line, Line::Raw(raw) if raw.trim().starts_with("; Aetheris — "))
            {
                let last_content = self
                    .lines
                    .iter()
                    .rposition(|line| !matches!(line, Line::Raw(raw) if raw.trim().is_empty()));
                let after_header =
                    last_content.is_some_and(|index| matches!(self.lines[index], Line::Section(_)));
                if previous_category || after_header {
                    self.lines
                        .truncate(last_content.map_or(0, |index| index + 1));
                    if previous_category {
                        self.lines.push(Line::Raw(String::new()));
                    }
                }
                previous_category = true;
            }
            self.lines.push(line);
        }
    }

    fn section_end(&self, section: &str) -> Option<usize> {
        let start = self.lines.iter().position(
            |line| matches!(line, Line::Section(raw) if section_name(raw).is_some_and(|name| name.eq_ignore_ascii_case(section))),
        )?;

        Some(
            self.lines[start + 1..]
                .iter()
                .position(|line| matches!(line, Line::Section(_)))
                .map_or(self.lines.len(), |offset| start + 1 + offset),
        )
    }
}

impl fmt::Display for EngineIni {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.bom {
            formatter.write_str("\u{feff}")?;
        }
        for (index, line) in self.lines.iter().enumerate() {
            if index > 0 {
                formatter.write_str(&self.newline)?;
            }

            match line {
                Line::Section(raw) | Line::Raw(raw) => formatter.write_str(raw)?,
                Line::Property {
                    source: Some(raw), ..
                } => formatter.write_str(raw)?,
                Line::Property { key, value, .. } => write!(formatter, "{key}={value}")?,
            }
        }

        if self.trailing_newline {
            formatter.write_str(&self.newline)?;
        }
        Ok(())
    }
}

fn section_name(raw: &str) -> Option<&str> {
    raw.trim()
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .map(str::trim)
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::{EngineIni, discover_engine_ini_in, engine_ini_below, validate_engine_ini_path};

    fn test_directory(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("aetheris-{name}-{}", std::process::id()))
    }

    #[test]
    fn discovers_engine_ini_in_current_local_app_data() {
        let root = test_directory("current-user");
        let local_app_data = root.join("AppData").join("Local");
        let engine_ini = engine_ini_below(&local_app_data);
        fs::create_dir_all(engine_ini.parent().unwrap()).unwrap();
        fs::write(&engine_ini, "[SystemSettings]").unwrap();

        assert_eq!(
            discover_engine_ini_in(Some(&local_app_data), &[]),
            Some(engine_ini)
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn discovers_engine_ini_in_another_windows_profile() {
        let root = test_directory("other-user");
        let engine_ini = engine_ini_below(&root.join("OtherUser").join("AppData").join("Local"));
        fs::create_dir_all(engine_ini.parent().unwrap()).unwrap();
        fs::write(&engine_ini, "[SystemSettings]").unwrap();

        assert_eq!(
            discover_engine_ini_in(None, std::slice::from_ref(&root)),
            Some(engine_ini)
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_a_manually_selected_file_with_the_wrong_name() {
        let root = test_directory("invalid-selection");
        let selected = root.join("Game.ini");
        fs::create_dir_all(&root).unwrap();
        fs::write(&selected, "").unwrap();

        assert_eq!(
            validate_engine_ini_path(selected),
            Err("Le fichier sélectionné doit s’appeler Engine.ini.")
        );

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reads_properties_case_insensitively() {
        let document = EngineIni::parse("[SystemSettings]\nr.VSync=1");

        assert_eq!(document.get("systemsettings", "r.vsync"), Some("1"));
    }

    #[test]
    fn updates_a_property_without_losing_comments() {
        let mut document = EngineIni::parse("; Keep this\n[SystemSettings]\nr.VSync=1");

        document.set("SystemSettings", "r.VSync", "0");

        assert_eq!(
            document.to_string(),
            "; Keep this\n[SystemSettings]\nr.VSync=0"
        );
    }

    #[test]
    fn appends_a_property_to_an_existing_section() {
        let mut document = EngineIni::parse("[SystemSettings]\nr.VSync=1\n\n[Audio]");

        document.set("SystemSettings", "r.MotionBlurQuality", "0");

        assert_eq!(
            document.to_string(),
            "[SystemSettings]\nr.VSync=1\n\nr.MotionBlurQuality=0\n[Audio]"
        );
    }

    #[test]
    fn creates_a_missing_section() {
        let mut document = EngineIni::parse("; Aetheris");

        document.set("SystemSettings", "r.VSync", "0");

        assert_eq!(
            document.to_string(),
            "; Aetheris\n\n[SystemSettings]\nr.VSync=0"
        );
    }
}
