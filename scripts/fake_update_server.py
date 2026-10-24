#!/usr/bin/env python3
"""
Fake GitHub Releases server for testing Qwerkeys auto-updater and UI progress bar.
Automatically reads Cargo.toml and bumps version so it is ALWAYS newer than current app.

Serves:
  GET /releases/latest -> GitHub release JSON (version dynamically higher than Cargo.toml)
  GET /downloads/Qwerkeys-<ver>-<arch>.zip -> Throttled zip stream
"""

import http.server
import json
import os
import re
import shutil
import sys
import time
import zipfile
from pathlib import Path

PORT = 8999
RATE_LIMIT_MB_PER_SEC = 3.0  # ~3 MB/s so progress bar is visible in UI

REPO_ROOT = Path(__file__).resolve().parent.parent
CACHE_DIR = REPO_ROOT / ".tmp"


def get_current_app_version() -> tuple[int, int, int]:
    cargo_toml = REPO_ROOT / "Cargo.toml"
    if cargo_toml.exists():
        text = cargo_toml.read_text(encoding="utf-8")
        m = re.search(r'^\s*version\s*=\s*"(\d+)\.(\d+)\.(\d+)"', text, re.MULTILINE)
        if m:
            return int(m.group(1)), int(m.group(2)), int(m.group(3))
    return 0, 1, 0


def get_target_version() -> str:
    # Allow manual override via FAKE_RELEASE_VERSION if desired
    env_ver = os.environ.get("FAKE_RELEASE_VERSION")
    if env_ver:
        return env_ver.lstrip("v")
    major, minor, patch = get_current_app_version()
    # Always bump minor version so it is guaranteed newer than local build
    return f"{major}.{minor + 1}.0"


def get_asset_path(version: str, arch: str) -> Path:
    CACHE_DIR.mkdir(parents=True, exist_ok=True)
    target = CACHE_DIR / f"Qwerkeys-{version}-{arch}.zip"
    if target.exists() and target.stat().st_size > 0:
        return target

    # If any previous zip exists for this arch, copy it over to preserve real build bundle
    existing = list(CACHE_DIR.glob(f"Qwerkeys-*-{arch}.zip"))
    if existing:
        print(f"[fake-server] Copying existing bundle {existing[0].name} -> {target.name}...")
        shutil.copyfile(existing[0], target)
        return target

    print(f"[fake-server] Creating dummy {arch} zip at {target}...")
    with zipfile.ZipFile(target, "w", zipfile.ZIP_DEFLATED) as zf:
        zf.writestr("Qwerkeys.app/Contents/Info.plist", b"<plist></plist>")
        zf.writestr("Qwerkeys.app/Contents/MacOS/qwerkeys", b"#!/bin/sh\nexit 0\n")
        # 12MB dummy payload so download takes ~4s at 3 MB/s
        zf.writestr("Qwerkeys.app/Contents/Resources/dummy.bin", b"\0" * (12 * 1024 * 1024))
    return target


class FakeGitHubHandler(http.server.BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        sys.stderr.write(f"[fake-server] {self.address_string()} - {format % args}\n")

    def do_GET(self):
        if self.path in ("/releases/latest", "/releases/latest/"):
            self.send_release_json()
        elif self.path.startswith("/downloads/"):
            filename = self.path.split("/")[-1]
            self.send_asset(filename)
        else:
            self.send_error(404, "Not Found")

    def send_release_json(self):
        version = get_target_version()
        tag = f"v{version}"
        arm_path = get_asset_path(version, "arm64")
        x86_path = get_asset_path(version, "x86_64")

        cur_v = ".".join(map(str, get_current_app_version()))
        payload = {
            "tag_name": tag,
            "name": f"Qwerkeys {tag}",
            "body": f"## What's New in {tag}\n- Test release simulating GitHub endpoint\n- Auto-bumped above current app version ({cur_v})\n- UI progress bar verification",
            "assets": [
                {
                    "name": f"Qwerkeys-{version}-arm64.zip",
                    "browser_download_url": f"http://127.0.0.1:{PORT}/downloads/Qwerkeys-{version}-arm64.zip",
                    "size": arm_path.stat().st_size,
                },
                {
                    "name": f"Qwerkeys-{version}-x86_64.zip",
                    "browser_download_url": f"http://127.0.0.1:{PORT}/downloads/Qwerkeys-{version}-x86_64.zip",
                    "size": x86_path.stat().st_size,
                },
            ],
        }
        body = json.dumps(payload, indent=2).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def send_asset(self, filename: str):
        filepath = CACHE_DIR / filename
        if not filepath.exists():
            self.send_error(404, f"File {filename} not found")
            return

        total_size = filepath.stat().st_size
        self.send_response(200)
        self.send_header("Content-Type", "application/zip")
        self.send_header("Content-Length", str(total_size))
        self.end_headers()

        chunk_size = 128 * 1024  # 128 KB
        delay = chunk_size / (RATE_LIMIT_MB_PER_SEC * 1024 * 1024)

        sent = 0
        with open(filepath, "rb") as f:
            while True:
                chunk = f.read(chunk_size)
                if not chunk:
                    break
                try:
                    self.wfile.write(chunk)
                    self.wfile.flush()
                    sent += len(chunk)
                    if delay > 0:
                        time.sleep(delay)
                except (BrokenPipeError, ConnectionResetError):
                    print(f"\n[fake-server] Client disconnected at {sent}/{total_size} bytes")
                    return

        print(f"\n[fake-server] Finished streaming {filename} ({total_size} bytes)")


def main():
    cur_v = ".".join(map(str, get_current_app_version()))
    target_v = get_target_version()
    get_asset_path(target_v, "arm64")
    get_asset_path(target_v, "x86_64")

    server = http.server.ThreadingHTTPServer(("127.0.0.1", PORT), FakeGitHubHandler)
    print(f"Fake GitHub Releases server running at http://127.0.0.1:{PORT}")
    print(f"  Current Cargo.toml: v{cur_v}")
    print(f"  Target Release:     v{target_v} (auto-bumped)")
    print(f"  Endpoint:           http://127.0.0.1:{PORT}/releases/latest")
    print(f"  Throttle:           ~{RATE_LIMIT_MB_PER_SEC} MB/s")
    print("Press Ctrl+C to stop.")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nStopping fake server.")


if __name__ == "__main__":
    main()
