mod iokit;
mod mapping;
mod midi;
mod scale;
mod state;
mod ui;

use std::env;
use std::error::Error;
use std::ffi::{c_int, c_void};
use std::sync::Arc;
use std::thread;

use iokit::{DeviceTargetMode, HidSystem};
use midi::MidiEngine;
use state::AppState;
use ui::QwerkeysApp;

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

    // Target selection: user argument, first external keyboard, or omni/internal fallback
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
        keyboards.iter().find(|k| !k.is_internal)
    };

    let (target_mode, is_suppressed, display_desc) = if let Some(dev) = chosen {
        let is_ext = !dev.is_internal;
        state.set_selected_device(dev.device_ptr);
        (
            DeviceTargetMode::Specific {
                name: dev.name.clone(),
                vid: dev.vendor_id,
                pid: dev.product_id,
            },
            is_ext,
            format!("{} ({})", dev.name, dev.transport),
        )
    } else {
        state.set_selected_device(0);
        (
            DeviceTargetMode::Omni,
            false,
            "All Keyboards (Omni)".to_string(),
        )
    };

    iokit::set_suppression_enabled(is_suppressed);
    setup_signal_handler();

    let port_name = "qwerkeys-input";
    let midi_engine = Arc::new(MidiEngine::new(port_name, Arc::clone(&state))?);
    midi_engine.panic_all_notes();

    // Clean startup banner
    println!("qwerkeys • in-scale dedicated midi keyboard");
    println!("  Target:  {display_desc}");
    println!(
        "  Mode:    {}",
        if is_suppressed {
            "Dedicated (keystrokes suppressed via CGEventTap)"
        } else {
            "Shared (internal / omni keyboard)"
        }
    );
    println!("  Port:    {}", port_name);
    println!("Starting GUI window...\n");

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

    // Run native macOS GUI on main thread
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1160.0, 740.0])
            .with_min_inner_size([1040.0, 663.4])
            .with_title("Qwerkeys • In-Scale MIDI Performance Keyboard"),
        ..Default::default()
    };

    let app_state = Arc::clone(&state);
    let app_midi = Arc::clone(&midi_engine);

    eframe::run_native(
        "Qwerkeys",
        native_options,
        Box::new(move |cc| Ok(Box::new(QwerkeysApp::new(cc, app_state, app_midi)))),
    )?;

    // Clean up upon GUI close
    iokit::stop_hid_system();

    Ok(())
}

unsafe extern "C" {
    fn signal(sig: c_int, handler: extern "C" fn(c_int)) -> *mut c_void;
}

extern "C" fn sigint_handler(_: c_int) {
    iokit::stop_hid_system();
    println!("\n[qwerkeys] Stopped.");
    std::process::exit(0);
}

fn setup_signal_handler() {
    unsafe {
        signal(2, sigint_handler);  // SIGINT
        signal(15, sigint_handler); // SIGTERM
    }
    let _ = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {
        iokit::stop_hid_system();
    }));
}
