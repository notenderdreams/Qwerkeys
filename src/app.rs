//! Application entry point: CLI argument handling, hardware discovery,
//! MIDI engine setup, the background HID listener, and the GUI event loop.

use std::error::Error;
use std::ffi::{c_int, c_void};
use std::sync::Arc;
use std::thread;

use crate::iokit;
use crate::iokit::{DeviceTargetMode, HidSystem, KeyboardDevice};
use crate::midi::MidiEngine;
use crate::state::AppState;
use crate::ui::QwerkeysApp;
use crate::utils::print_banner;

const MIDI_PORT_NAME: &str = "qwerkeys-input";
const APP_TITLE: &str = "Qwerkeys";
const WINDOW_SIZE: [f32; 2] = [1160.0, 770.0];

/// Parsed command-line flags for [`run`].
struct Cli {
    /// `--list` / `-l`: print connected keyboards and exit.
    list: bool,
    /// `--device <n|name>` / `-d <n|name>`: keyboard index or name substring.
    device: Option<String>,
}

fn parse_cli(args: &[String]) -> Cli {
    let mut cli = Cli {
        list: false,
        device: None,
    };
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--list" | "-l" => cli.list = true,
            "--device" | "-d" => cli.device = it.next().cloned(),
            _ => {}
        }
    }
    cli
}

fn print_keyboards(keyboards: &[KeyboardDevice]) {
    println!("Connected Keyboards:");
    for k in keyboards {
        let kind = if k.is_internal {
            "Built-in"
        } else {
            "External"
        };
        println!("  [{}] {:<32} ({}) [{kind}]", k.index, k.name, k.transport);
    }
}

/// Pick the target keyboard: explicit `--device` (index or name substring),
/// then the first external keyboard, else omni. Also records the choice in
/// `state`. Returns `(target mode, keystroke suppression, display name)`.
fn resolve_target(
    cli: &Cli,
    state: &AppState,
    keyboards: &[KeyboardDevice],
) -> (DeviceTargetMode, bool, String) {
    let requested = cli
        .device
        .as_deref()
        .and_then(|target| match target.parse::<usize>() {
            Ok(index) => keyboards.iter().find(|k| k.index == index),
            Err(_) => keyboards
                .iter()
                .find(|k| k.name.to_lowercase().contains(&target.to_lowercase())),
        });

    match requested.or_else(|| keyboards.iter().find(|k| !k.is_internal)) {
        Some(dev) => {
            let is_external = !dev.is_internal;
            state.set_selected_device(dev.device_ptr);
            (
                DeviceTargetMode::Specific {
                    name: dev.name.clone(),
                    vid: dev.vendor_id,
                    pid: dev.product_id,
                },
                is_external,
                format!("{} ({})", dev.name, dev.transport),
            )
        }
        None => {
            state.set_selected_device(0);
            (
                DeviceTargetMode::Omni,
                false,
                "All Keyboards (Omni)".to_string(),
            )
        }
    }
}

fn run_gui(state: Arc<AppState>, midi: Arc<MidiEngine>) -> Result<(), eframe::Error> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size(WINDOW_SIZE)
        .with_title(APP_TITLE);
    if let Ok(icon) = eframe::icon_data::from_png_bytes(crate::asset_bytes!("app-icon.png")) {
        viewport = viewport.with_icon(icon);
    }

    eframe::run_native(
        APP_TITLE,
        eframe::NativeOptions {
            viewport,
            ..Default::default()
        },
        Box::new(move |cc| Ok(Box::new(QwerkeysApp::new(cc, state, midi)))),
    )
}

/// Application entry point: hardware setup, MIDI engine, background
/// HID listener, and the GUI event loop.
pub fn run(args: &[String]) -> Result<(), Box<dyn Error>> {
    let hid = HidSystem::new()?;
    let keyboards = hid.enumerate_keyboards();
    let cli = parse_cli(args);

    if cli.list {
        print_keyboards(&keyboards);
        return Ok(());
    }

    let state = Arc::new(AppState::new());
    if let Ok(mut devs) = state.devices.lock() {
        *devs = keyboards.clone();
    }

    let (target_mode, is_suppressed, display_desc) = resolve_target(&cli, &state, &keyboards);
    iokit::set_suppression_enabled(is_suppressed);
    install_shutdown_hooks();

    let midi_engine = Arc::new(MidiEngine::new(MIDI_PORT_NAME, Arc::clone(&state))?);
    midi_engine.panic_all_notes();
    print_banner(&display_desc, is_suppressed, MIDI_PORT_NAME);

    // Start background IOHID event listener thread
    let listener_engine = Arc::clone(&midi_engine);
    thread::spawn(move || {
        hid.start_listener(target_mode, move |usage, is_down| {
            if is_down {
                listener_engine.handle_key_down(usage);
            } else {
                listener_engine.handle_key_up(usage);
            }
        });
    });

    run_gui(state, midi_engine)?;

    // Clean up upon GUI close
    iokit::stop_hid_system();

    Ok(())
}

const SIGINT: c_int = 2;
const SIGTERM: c_int = 15;

unsafe extern "C" {
    fn signal(sig: c_int, handler: extern "C" fn(c_int)) -> *mut c_void;
}

extern "C" fn sigint_handler(_: c_int) {
    iokit::stop_hid_system();
    println!("\n[qwerkeys] Stopped.");
    std::process::exit(0);
}

/// Ensure SIGINT/SIGTERM/panics restore the event tap and MIDI port state.
fn install_shutdown_hooks() {
    unsafe {
        signal(SIGINT, sigint_handler);
        signal(SIGTERM, sigint_handler);
    }
    std::panic::set_hook(Box::new(|_| iokit::stop_hid_system()));
}
