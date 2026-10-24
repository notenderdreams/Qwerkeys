# Qwerkeys — Agent / AI Context

## What This Is

Qwerkeys is a macOS menubar app (Rust + egui) that turns a physical keyboard into a MIDI input device. It captures keystrokes via IOKit HID, maps them to MIDI notes, and outputs through a virtual MIDI port.

## How It Works

1. **Startup** — `app::run` discovers connected keyboards via IOKit HID, creates a virtual MIDI port (`qwerkeys-input`), and checks macOS TCC permissions.
2. **Onboarding** — if permissions are missing (or it's a debug build), the onboarding UI walks the user through granting Accessibility + Input Monitoring.
3. **HID capture** — a background thread runs the IOKit event loop, intercepting keystrokes from the target keyboard. Keystrokes are suppressed so they don't reach other apps.
4. **Mapping** — each key event is mapped to a MIDI note number using the current scale/octave/root settings from `AppState`.
5. **MIDI output** — mapped notes are sent as `NoteOn`/`NoteOff` messages through the virtual MIDI port via `midir`.
6. **GUI** — the egui/eframe window shows a visual keyboard, piano roll, device selector, and scale controls. State is shared with the HID thread via `Arc<Mutex<AppState>>`.
7. **Settings** — user preferences are persisted to disk on change via `settings.rs`.

## Architecture

```
src/
├── main.rs          — entry point, delegates to app::run
├── app.rs           — CLI parsing, hardware discovery, MIDI setup, GUI loop
├── lib.rs           — crate root, re-exports modules
├── state.rs         — shared AppState (Arc<Mutex>) across threads
├── settings.rs      — persisted user config (serialized to disk)
├── permissions.rs   — macOS TCC permission checks (Accessibility, ListenEvent)
├── midi.rs          — virtual MIDI port via midir
├── scale.rs         — music theory: scales, intervals, note names
├── utils.rs         — misc helpers
├── iokit/           — IOKit HID bindings
│   ├── ffi.rs       — raw C FFI declarations
│   ├── device.rs    — KeyboardDevice discovery
│   └── mod.rs       — HidSystem, event loop, DeviceTargetMode
├── mapping/
│   ├── keys.rs      — keyboard key → MIDI note mapping
│   └── mod.rs
└── ui/
    ├── mod.rs        — QwerkeysApp (eframe App impl), top-level layout
    ├── onboarding.rs — first-run permission grant flow
    ├── keyboard.rs   — visual keyboard display
    ├── piano.rs      — piano roll / note visualization
    ├── status_bar.rs — status bar
    ├── setup_bar.rs  — device/scale/octave controls
    ├── settings.rs   — settings panel UI
    └── theme.rs      — egui visual theme
```

## Key Facts for Agents

- **Bundle ID**: `com.notenderdreams.qwerkeys`
- **macOS permissions needed**: `Accessibility` + `ListenEvent` (Input Monitoring) — both TCC-gated
- **Build**: `cargo build` (debug) / `cargo build --release`
- **Bundle**: `just bundle v0.0.0-debug` or `just install-debug`
- **Reset TCC entries**: `just reset-permissions` — run *after* install, not before
- **Settings file**: persisted via `settings.rs`; location follows macOS conventions
- **MIDI port name**: `qwerkeys-input` (virtual, created at startup)

## Common Tasks

| Task | Command |
|------|---------|
| Run debug | `just run` |
| Fake update server | `just fake-update-server` |
| Build debug | `just build` |
| Install debug + reset perms | `just install-debug` |
| Reset TCC only | `just reset-permissions` |
| Lint | `just lint` |
| Test | `just test` |
| Full CI | `just ci` |

## UI Conventions
- **Icons**: use `egui-phosphor` — it's already a dependency. Never use Unicode symbols (⚙, ✓, ▶, etc.) for icons. Pick the right Phosphor glyph from the `egui_phosphor::regular` / `egui_phosphor::bold` / `egui_phosphor::fill` modules.

## graphify

This project has a knowledge graph at `graphify-out/` with god nodes, community structure, and cross-file relationships.

When the user types `/graphify`, use the installed graphify skill or instructions before doing anything else.

Rules:
- For codebase questions, first run `graphify query "<question>"` when `graphify-out/graph.json` exists. Use `graphify path "<A>" "<B>"` for relationships and `graphify explain "<concept>"` for focused concepts.
- If `graphify-out/wiki/index.md` exists, use it for broad navigation instead of raw source browsing.
- Read `graphify-out/GRAPH_REPORT.md` only for broad architecture review or when query/path/explain do not surface enough context.
- After modifying code, run `graphify update .` to keep the graph current (AST-only, no API cost).

