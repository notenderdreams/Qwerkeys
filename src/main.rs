mod iokit;
mod mapping;
mod midi;
mod scale;
mod state;
mod ui;

use std::env;
use std::error::Error;
use std::ffi::{c_int, c_void};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

use iokit::{try_seize_device, unseize_device, HidSystem};
use midi::MidiEngine;
use state::AppState;
use ui::NekotoApp;

static SEIZED_DEVICE: AtomicUsize = AtomicUsize::new(0);

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let hid = HidSystem::new().map_err(Box::<dyn Error>::from)?;
    let keyboards = hid.enumerate_keyboards();

    // CLI List mode
    if args.iter().any(|a| a == "--list" || a == "-l") {
        println!("Connected Keyboards:");
        for k in &keyboards {
            let kind = if k.is_internal { "Built-in" } else { "External" };
            println!(
                "  [{}] {:<32} ({}) [{}]",
                k.index, k.name, k.transport, kind
            );
        }
        return Ok(());
    }

    let state = Arc::new(AppState::new());
    if let Ok(mut devs) = state.devices.lock() {
        *devs = keyboards.clone();
    }

    // Target selection: user argument or first external keyboard
    let chosen = if let Some(pos) = args.iter().position(|a| a == "--device" || a == "-d") {
        if let Some(target) = args.get(pos + 1) {
            if let Ok(idx) = target.parse::<usize>() {
                keyboards.iter().find(|k| k.index == idx)
            } else {
                keyboards.iter().find(|k| k.name.to_lowercase().contains(&target.to_lowercase()))
            }
        } else {
            None
        }
    } else {
        keyboards.iter().find(|k| !k.is_internal).or_else(|| keyboards.first())
    };

    let target_ptr = chosen.map(|d| d.device_ptr).unwrap_or(0);
    let is_external = chosen.map(|d| !d.is_internal).unwrap_or(false);

    let is_seized = if is_external && target_ptr != 0 {
        try_seize_device(target_ptr)
    } else {
        false
    };

    if is_seized {
        SEIZED_DEVICE.store(target_ptr, Ordering::SeqCst);
        state.is_seized.store(true, Ordering::SeqCst);
    }
    state.set_selected_device(target_ptr);
    iokit::set_suppression_enabled(is_external);

    setup_signal_handler();

    let port_name = "nekoto-input";
    let midi_engine = Arc::new(MidiEngine::new(port_name, Arc::clone(&state))?);

    // Clean startup banner
    println!("nekoto • in-scale dedicated midi keyboard");
    if let Some(dev) = chosen {
        println!("  Device:  {} ({})", dev.name, dev.transport);
        println!(
            "  Mode:    {}",
            if is_external {
                if is_seized {
                    "Dedicated (hardware seized)"
                } else {
                    "Dedicated (keystrokes suppressed)"
                }
            } else {
                "Shared (internal keyboard)"
            }
        );
    }
    println!("  Port:    {}", port_name);
    println!("Starting GUI window...\n");

    // Start background IOHID event listener thread
    let listener_engine = Arc::clone(&midi_engine);
    thread::spawn(move || {
        hid.start_listener(target_ptr, move |usage, is_down| {
            if is_down {
                listener_engine.handle_key_down(usage);
            } else {
                listener_engine.handle_key_up(usage);
            }
        });
    });

    // Run native macOS GUI on main thread
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1040.0, 530.0])
            .with_min_inner_size([920.0, 480.0])
            .with_title("Nekoto • Dedicated MIDI Keyboard"),
        ..Default::default()
    };

    let app_state = Arc::clone(&state);
    let app_midi = Arc::clone(&midi_engine);

    eframe::run_native(
        "Nekoto",
        native_options,
        Box::new(move |cc| Ok(Box::new(NekotoApp::new(cc, app_state, app_midi)))),
    )?;

    // Clean up upon GUI close
    let dev = SEIZED_DEVICE.load(Ordering::SeqCst);
    if dev != 0 {
        unseize_device(dev);
    }
    iokit::stop_hid_system();

    Ok(())
}

unsafe extern "C" {
    fn signal(sig: c_int, handler: extern "C" fn(c_int)) -> *mut c_void;
}

extern "C" fn sigint_handler(_: c_int) {
    let dev = SEIZED_DEVICE.load(Ordering::SeqCst);
    if dev != 0 {
        unseize_device(dev);
    }
    iokit::stop_hid_system();
    println!("\n[nekoto] Stopped.");
    std::process::exit(0);
}

fn setup_signal_handler() {
    unsafe {
        signal(2, sigint_handler);  // SIGINT
        signal(15, sigint_handler); // SIGTERM
    }
    let _ = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {
        let dev = SEIZED_DEVICE.load(Ordering::SeqCst);
        if dev != 0 {
            unseize_device(dev);
        }
        iokit::stop_hid_system();
    }));
}
