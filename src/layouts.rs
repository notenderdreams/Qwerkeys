//! Saved/imported mapping layouts.
//!
//! Saved layouts are small JSON files in
//! `~/Library/Application Support/Qwerkeys/layouts/`. Each file contains a
//! user-chosen name plus a direct `usage → MIDI note` table that is loaded
//! into the app state as the Custom layout when selected.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::mapping::CustomLayoutMap;

/// JSON document behind one `.json` layout file.
#[derive(Serialize, Deserialize)]
struct LayoutFile {
    name: String,
    /// usage (decimal, string keys to stay JSON-friendly) → MIDI note.
    #[serde(rename = "layout")]
    layout: BTreeMap<String, u8>,
}

/// One saved layout as read from disk (or about to be written).
pub struct SavedLayout {
    pub name: String,
    pub map: CustomLayoutMap,
}

fn layouts_dir() -> Option<PathBuf> {
    let mut path = PathBuf::from(std::env::var_os("HOME")?);
    path.push("Library/Application Support/Qwerkeys/layouts");
    fs::create_dir_all(&path).ok()?;
    Some(path)
}

/// Strip everything unlikely to survive a filename: path separators,
/// control chars, leading/trailing dots and whitespace.
pub fn sanitize_name(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' => '-',
            c if c.is_control() => ' ',
            c => c,
        })
        .collect();
    let trimmed = cleaned.trim_matches(|c: char| c.is_whitespace() || c == '.' || c == '-');
    if trimmed.is_empty() {
        "Imported Layout".to_string()
    } else {
        trimmed.chars().take(60).collect()
    }
}

/// On-disk name of a layout (its `name` field sanitized).
fn file_name(name: &str) -> String {
    format!("{}.json", sanitize_name(name))
}

/// Every saved layout in the layouts directory, sorted by name.
pub fn list() -> Vec<SavedLayout> {
    let Some(dir) = layouts_dir() else {
        return Vec::new();
    };
    let Ok(entries) = fs::read_dir(&dir) else {
        return Vec::new();
    };

    let mut layouts = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|ext| ext != "json") {
            continue;
        }
        if let Ok(content) = fs::read_to_string(&path)
            && let Ok(file) = serde_json::from_str::<LayoutFile>(&content)
        {
            layouts.push(SavedLayout {
                name: file.name,
                map: file
                    .layout
                    .into_iter()
                    .filter_map(|(k, v)| k.parse::<u32>().ok().map(|usage| (usage, v)))
                    .collect(),
            });
        }
    }
    layouts.sort_by_key(|a| a.name.to_lowercase());
    layouts
}

/// Persist a layout under `name`, overwriting a same-named layout.
pub fn store(name: &str, map: &CustomLayoutMap) {
    let Some(dir) = layouts_dir() else {
        return;
    };
    let file = LayoutFile {
        name: sanitize_name(name),
        layout: map
            .iter()
            .map(|(usage, note)|(usage.to_string(), *note))
            .collect(),
    };
    if let Ok(json) = serde_json::to_string(&file) {
        let path = dir.join(file_name(name));
        let _ = fs::write(path, json);
    }
}

/// Write a layout to a user-chosen path (Export). `path` comes from a
/// file dialog and is used as-is.
pub fn write_to(path: &std::path::Path, name: &str, map: &CustomLayoutMap) -> bool {
    let file = LayoutFile {
        name: name.to_string(),
        layout: map
            .iter()
            .map(|(usage, note)| (usage.to_string(), *note))
            .collect(),
    };
    serde_json::to_string(&file)
        .ok()
        .and_then(|json| fs::write(path, json).ok())
        .is_some()
}

/// Read a `.json` layout from an arbitrary path (Import). Returns the
/// layout as parsed — the caller decides whether to adopt it.
pub fn read_from(path: &std::path::Path, fallback_name: &str) -> Option<SavedLayout> {
    let content = fs::read_to_string(path).ok()?;
    let mut file: LayoutFile = serde_json::from_str(&content).ok()?;
    if file.name.trim().is_empty() {
        file.name = sanitize_name(fallback_name);
    }
    Some(SavedLayout {
        name: sanitize_name(&file.name),
        map: file
            .layout
            .into_iter()
            .filter_map(|(k, v)| k.parse::<u32>().ok().map(|usage| (usage, v)))
            .collect(),
    })
}

/// Remove a layout from the layouts directory. Returns whether a file was
/// actually removed.
pub fn delete(name: &str) -> bool {
    let Some(dir) = layouts_dir() else {
        return false;
    };
    fs::remove_file(dir.join(file_name(name))).is_ok()
}

/// Open native macOS file picker to choose a JSON layout file.
pub fn pick_import_file() -> Option<PathBuf> {
    let output = std::process::Command::new("osascript")
        .args([
            "-e",
            "POSIX path of (choose file of type {\"public.json\", \"json\"} with prompt \"Select Layout JSON\")",
        ])
        .output()
        .ok()?;
    if output.status.success() {
        let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path_str.is_empty() {
            return Some(PathBuf::from(path_str));
        }
    }
    None
}

/// Open native macOS save dialog to pick destination for exporting a JSON layout.
pub fn pick_export_file(default_name: &str) -> Option<PathBuf> {
    let clean = sanitize_name(default_name);
    let filename = format!("{clean}.json");
    let script = format!(
        "POSIX path of (choose file name default name \"{filename}\" with prompt \"Export Layout JSON\")"
    );
    let output = std::process::Command::new("osascript")
        .args(["-e", &script])
        .output()
        .ok()?;
    if output.status.success() {
        let mut path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !path_str.is_empty() {
            if !path_str.ends_with(".json") {
                path_str.push_str(".json");
            }
            return Some(PathBuf::from(path_str));
        }
    }
    None
}

/// Prompt user to pick a layout file, import it into the layouts directory,
/// and return the saved layout.
pub fn import_dialog() -> Option<SavedLayout> {
    let path = pick_import_file()?;
    let fallback = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Imported Layout");
    let layout = read_from(&path, fallback)?;
    store(&layout.name, &layout.map);
    Some(layout)
}

/// Prompt user to export a layout to a file.
pub fn export_dialog(name: &str, map: &CustomLayoutMap) -> bool {
    let Some(path) = pick_export_file(name) else {
        return false;
    };
    write_to(&path, name, map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("My Layout"), "My Layout");
        assert_eq!(sanitize_name("a/b\\c:d"), "a-b-c-d");
        assert_eq!(sanitize_name("  ..leading dots  "), "leading dots");
        assert_eq!(sanitize_name(""), "Imported Layout");
        assert_eq!(sanitize_name("///"), "Imported Layout");
    }

    #[test]
    fn test_parse_layout_json() {
        let json = r#"{"name":"Test","layout":{"4":60,"5":61}}"#;
        let file = serde_json::from_str::<LayoutFile>(json).unwrap();
        assert_eq!(file.name, "Test");
        assert_eq!(file.layout.get("4"), Some(&60));
    }

    #[test]
    fn test_write_read_roundtrip() {
        let tmp_path = std::env::temp_dir().join("qwerkeys_test_layout.json");
        let mut map = CustomLayoutMap::new();
        map.insert(4, 60);
        map.insert(5, 62);
        assert!(write_to(&tmp_path, "Custom Test", &map));

        let loaded = read_from(&tmp_path, "Fallback").expect("read layout");
        assert_eq!(loaded.name, "Custom Test");
        assert_eq!(loaded.map.get(&4), Some(&60));
        assert_eq!(loaded.map.get(&5), Some(&62));

        let _ = fs::remove_file(tmp_path);
    }
}
