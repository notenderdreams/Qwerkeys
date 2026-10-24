//! In-app auto-updater module for Qwerkeys.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

use serde::Deserialize;

pub const REPO_OWNER: &str = "notenderdreams";
pub const REPO_NAME: &str = "qwerkeys";
pub const DEV_UPDATE_URL: &str = "http://127.0.0.1:8999/releases/latest";

#[derive(Debug, Clone, Deserialize)]
pub struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub body: Option<String>,
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Clone, Default)]
pub enum UpdateStatus {
    #[default]
    Idle,
    Checking,
    UpToDate,
    Available {
        version: String,
        notes: String,
        download_url: String,
        total_size: u64,
    },
    Downloading {
        version: String,
        downloaded: u64,
        total: u64,
    },
    Extracting,
    ReadyToRestart {
        version: String,
        script_path: PathBuf,
    },
    DevBuildSkipped {
        version: String,
    },
    Error(String),
}

pub struct UpdateState {
    pub status: UpdateStatus,
}

impl Default for UpdateState {
    fn default() -> Self {
        Self {
            status: UpdateStatus::Idle,
        }
    }
}

pub fn system_arch() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" => "arm64",
        "x86_64" => "x86_64",
        other => other,
    }
}

pub fn current_app_bundle() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let mut current = exe.as_path();
    while let Some(parent) = current.parent() {
        if parent.extension().and_then(|s| s.to_str()) == Some("app") {
            return Some(parent.to_path_buf());
        }
        current = parent;
    }
    let installed = PathBuf::from("/Applications/Qwerkeys.app");
    if installed.exists() {
        return Some(installed);
    }
    None
}

pub fn parse_version(tag: &str) -> Option<(u64, u64, u64)> {
    let s = tag.strip_prefix('v').unwrap_or(tag);
    let mut parts = s.split('.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next()?.parse::<u64>().ok()?;
    let patch = parts.next()?.parse::<u64>().ok()?;
    Some((major, minor, patch))
}

pub fn is_newer(remote_tag: &str, current_version: &str) -> bool {
    let Some(remote) = parse_version(remote_tag) else {
        return false;
    };
    let Some(current) = parse_version(current_version) else {
        return false;
    };
    remote > current
}

pub fn find_matching_asset<'a>(
    assets: &'a [ReleaseAsset],
    version: &str,
    arch: &str,
) -> Option<&'a ReleaseAsset> {
    let clean_ver = version.strip_prefix('v').unwrap_or(version);
    let expected_name = format!("Qwerkeys-{clean_ver}-{arch}.zip");
    assets.iter().find(|a| a.name == expected_name)
}

pub fn generate_swap_script(pid: u32, staged: &Path, target: &Path) -> String {
    let staged_str = staged.to_string_lossy();
    let target_str = target.to_string_lossy();
    let parent_staged = staged.parent().unwrap_or(staged).to_string_lossy();

    format!(
        r#"#!/bin/sh
while kill -0 {pid} 2>/dev/null; do
    sleep 0.1
done
xattr -r -d com.apple.quarantine "{staged_str}" 2>/dev/null || true
rm -rf "{target_str}"
cp -R "{staged_str}" "{target_str}"
rm -rf "{parent_staged}"
open "{target_str}"
"#
    )
}

pub fn get_update_url() -> String {
    if cfg!(debug_assertions)
        && ureq::get(DEV_UPDATE_URL)
            .timeout(std::time::Duration::from_millis(150))
            .call()
            .is_ok()
    {
        return DEV_UPDATE_URL.to_string();
    }

    format!("https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/releases/latest")
}

pub fn check_for_updates_async(state: Arc<Mutex<UpdateState>>, ctx: egui::Context) {
    {
        let mut s = state.lock().unwrap();
        s.status = UpdateStatus::Checking;
    }
    ctx.request_repaint();

    thread::spawn(move || {
        let current_ver = env!("CARGO_PKG_VERSION");
        let url = get_update_url();
        eprintln!("[updater] Checking for updates at: {url} (current: {current_ver})");

        let response = match ureq::get(&url)
            .set("User-Agent", "Qwerkeys-Updater")
            .set("Accept", "application/vnd.github+json")
            .call()
        {
            Ok(res) => res,
            Err(e) => {
                let mut s = state.lock().unwrap();
                s.status = UpdateStatus::Error(format!("Network error: {e}"));
                ctx.request_repaint();
                return;
            }
        };

        let release: GitHubRelease = match serde_json::from_reader(response.into_reader()) {
            Ok(rel) => rel,
            Err(e) => {
                let mut s = state.lock().unwrap();
                s.status = UpdateStatus::Error(format!("Invalid response: {e}"));
                ctx.request_repaint();
                return;
            }
        };

        let mut s = state.lock().unwrap();
        if is_newer(&release.tag_name, current_ver) {
            let arch = system_arch();
            if let Some(asset) = find_matching_asset(&release.assets, &release.tag_name, arch) {
                let clean_ver = release
                    .tag_name
                    .strip_prefix('v')
                    .unwrap_or(&release.tag_name)
                    .to_string();
                eprintln!("[updater] New version available: {clean_ver}");
                s.status = UpdateStatus::Available {
                    version: clean_ver,
                    notes: release.body.unwrap_or_default(),
                    download_url: asset.browser_download_url.clone(),
                    total_size: asset.size,
                };
            } else {
                eprintln!("[updater] No release asset for arch {arch}");
                s.status =
                    UpdateStatus::Error(format!("No release bundle found for architecture {arch}"));
            }
        } else {
            eprintln!("[updater] App is up to date ({current_ver})");
            s.status = UpdateStatus::UpToDate;
        }
        ctx.request_repaint();
    });
}

pub fn download_and_install_async(state: Arc<Mutex<UpdateState>>, ctx: egui::Context) {
    let (download_url, version, total_size) = {
        let s = state.lock().unwrap();
        match &s.status {
            UpdateStatus::Available {
                download_url,
                version,
                total_size,
                ..
            } => (download_url.clone(), version.clone(), *total_size),
            _ => return,
        }
    };

    {
        let mut s = state.lock().unwrap();
        s.status = UpdateStatus::Downloading {
            version: version.clone(),
            downloaded: 0,
            total: total_size,
        };
    }
    ctx.request_repaint();

    thread::spawn(move || {
        let temp_dir = std::env::temp_dir().join("qwerkeys-update");
        let _ = fs::remove_dir_all(&temp_dir);
        if let Err(e) = fs::create_dir_all(&temp_dir) {
            let mut s = state.lock().unwrap();
            s.status = UpdateStatus::Error(format!("Failed to create temp dir: {e}"));
            ctx.request_repaint();
            return;
        }

        let zip_path = temp_dir.join("update.zip");
        let response = match ureq::get(&download_url)
            .set("User-Agent", "Qwerkeys-Updater")
            .call()
        {
            Ok(res) => res,
            Err(e) => {
                let mut s = state.lock().unwrap();
                s.status = UpdateStatus::Error(format!("Download failed: {e}"));
                ctx.request_repaint();
                return;
            }
        };

        let mut reader = response.into_reader();
        let mut file = match File::create(&zip_path) {
            Ok(f) => f,
            Err(e) => {
                let mut s = state.lock().unwrap();
                s.status = UpdateStatus::Error(format!("Failed to save archive: {e}"));
                ctx.request_repaint();
                return;
            }
        };

        let mut buffer = [0u8; 65536];
        let mut downloaded = 0u64;

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    if let Err(e) = file.write_all(&buffer[..n]) {
                        let mut s = state.lock().unwrap();
                        s.status = UpdateStatus::Error(format!("Write error: {e}"));
                        ctx.request_repaint();
                        return;
                    }
                    downloaded += n as u64;
                    {
                        let mut s = state.lock().unwrap();
                        s.status = UpdateStatus::Downloading {
                            version: version.clone(),
                            downloaded,
                            total: total_size,
                        };
                    }
                    ctx.request_repaint();
                }
                Err(e) => {
                    let mut s = state.lock().unwrap();
                    s.status = UpdateStatus::Error(format!("Stream error: {e}"));
                    ctx.request_repaint();
                    return;
                }
            }
        }

        {
            let mut s = state.lock().unwrap();
            s.status = UpdateStatus::Extracting;
        }
        ctx.request_repaint();

        let staged_dir = temp_dir.join("staged");
        let _ = fs::create_dir_all(&staged_dir);

        let ditto = Command::new("/usr/bin/ditto")
            .args(["-x", "-k"])
            .arg(&zip_path)
            .arg(&staged_dir)
            .output();

        match ditto {
            Ok(output) if output.status.success() => {}
            Ok(output) => {
                let err = String::from_utf8_lossy(&output.stderr);
                let mut s = state.lock().unwrap();
                s.status = UpdateStatus::Error(format!("Extraction error: {err}"));
                ctx.request_repaint();
                return;
            }
            Err(e) => {
                let mut s = state.lock().unwrap();
                s.status = UpdateStatus::Error(format!("Failed to run ditto: {e}"));
                ctx.request_repaint();
                return;
            }
        }

        let staged_app = staged_dir.join("Qwerkeys.app");
        if !staged_app.exists() {
            let mut s = state.lock().unwrap();
            s.status = UpdateStatus::Error("Archive did not contain Qwerkeys.app".into());
            ctx.request_repaint();
            return;
        }

        let Some(target_app) = current_app_bundle() else {
            let mut s = state.lock().unwrap();
            s.status = UpdateStatus::DevBuildSkipped { version };
            ctx.request_repaint();
            return;
        };

        let pid = std::process::id();
        let script_content = generate_swap_script(pid, &staged_app, &target_app);
        let script_path = temp_dir.join("swap.sh");

        if let Err(e) = fs::write(&script_path, script_content) {
            let mut s = state.lock().unwrap();
            s.status = UpdateStatus::Error(format!("Failed to write swap script: {e}"));
            ctx.request_repaint();
            return;
        }

        let _ = fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755));

        {
            let mut s = state.lock().unwrap();
            s.status = UpdateStatus::ReadyToRestart {
                version,
                script_path,
            };
        }
        ctx.request_repaint();
    });
}

pub fn relaunch_and_swap(script_path: &Path) -> Result<(), String> {
    eprintln!("[updater] Launching swap script: {:?}", script_path);
    Command::new("/bin/sh")
        .arg(script_path)
        .spawn()
        .map_err(|e| format!("Failed to spawn updater script: {e}"))?;

    std::process::exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        assert_eq!(parse_version("v0.1.0"), Some((0, 1, 0)));
        assert_eq!(parse_version("0.1.0"), Some((0, 1, 0)));
        assert_eq!(parse_version("v1.2.34"), Some((1, 2, 34)));
        assert_eq!(parse_version("invalid"), None);
    }

    #[test]
    fn test_is_newer() {
        assert!(is_newer("v0.2.0", "0.1.0"));
        assert!(is_newer("0.1.1", "0.1.0"));
        assert!(is_newer("v1.0.0", "0.9.9"));
        assert!(!is_newer("v0.1.0", "0.1.0"));
        assert!(!is_newer("v0.0.9", "0.1.0"));
        assert!(!is_newer("invalid", "0.1.0"));
    }

    #[test]
    fn test_find_matching_asset() {
        let assets = vec![
            ReleaseAsset {
                name: "Qwerkeys-0.2.0-arm64.dmg".into(),
                browser_download_url: "https://example.com/arm64.dmg".into(),
                size: 1000,
            },
            ReleaseAsset {
                name: "Qwerkeys-0.2.0-arm64.zip".into(),
                browser_download_url: "https://example.com/arm64.zip".into(),
                size: 2000,
            },
            ReleaseAsset {
                name: "Qwerkeys-0.2.0-x86_64.zip".into(),
                browser_download_url: "https://example.com/x86_64.zip".into(),
                size: 3000,
            },
        ];

        let arm_match = find_matching_asset(&assets, "0.2.0", "arm64");
        assert_eq!(
            arm_match.map(|a| a.name.as_str()),
            Some("Qwerkeys-0.2.0-arm64.zip")
        );

        let x86_match = find_matching_asset(&assets, "0.2.0", "x86_64");
        assert_eq!(
            x86_match.map(|a| a.name.as_str()),
            Some("Qwerkeys-0.2.0-x86_64.zip")
        );

        let missing = find_matching_asset(&assets, "0.2.0", "riscv");
        assert!(missing.is_none());
    }

    #[test]
    fn test_update_status_default() {
        let status = UpdateStatus::default();
        assert!(matches!(status, UpdateStatus::Idle));
    }

    #[test]
    fn test_get_update_url_fallback() {
        let url = get_update_url();
        assert!(
            url == DEV_UPDATE_URL
                || url
                    == format!(
                        "https://api.github.com/repos/{REPO_OWNER}/{REPO_NAME}/releases/latest"
                    )
        );
    }

    #[test]
    fn test_generate_swap_script_content() {
        use std::path::Path;
        let script = generate_swap_script(
            1234,
            Path::new("/tmp/staged/Qwerkeys.app"),
            Path::new("/Applications/Qwerkeys.app"),
        );
        assert!(script.contains("while kill -0 1234 2>/dev/null; do"));
        assert!(script.contains("xattr -r -d com.apple.quarantine"));
        assert!(
            script.contains("cp -R \"/tmp/staged/Qwerkeys.app\" \"/Applications/Qwerkeys.app\"")
        );
        assert!(script.contains("open \"/Applications/Qwerkeys.app\""));
    }
}
