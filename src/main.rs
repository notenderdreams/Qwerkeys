mod iokit;
mod mapping;
mod midi;

use std::env;
use std::error::Error;
use std::ffi::{c_int, c_void};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use iokit::{try_seize_device, unseize_device, HidSystem};
use midi::MidiEngine;

static SEIZED_DEVICE: AtomicUsize = AtomicUsize::new(0);

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let hid = HidSystem::new().map_err(|e| Box::<dyn Error>::from(e))?;
    let keyboards = hid.enumerate_keyboards();

    // List mode
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

    let Some(device) = chosen else {
        eprintln!("No keyboard found.");
        return Ok(());
    };

    let is_external = !device.is_internal;
    let target_ptr = device.device_ptr;

    // Attempt hardware seize on external keyboards
    let is_seized = if is_external {
        try_seize_device(target_ptr)
    } else {
        false
    };

    if is_seized {
        SEIZED_DEVICE.store(target_ptr, Ordering::SeqCst);
    }
    setup_signal_handler();

    let port_name = "nekoto-input";
    let midi_engine = Arc::new(MidiEngine::new(port_name)?);

    // Clean, readable startup output
    println!("nekoto • dedicated midi keyboard");
    println!("  Device:  {} ({})", device.name, device.transport);
    println!("  Port:    {}", port_name);
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
    println!("Ready. Press Ctrl+C to stop.\n");

    let engine = Arc::clone(&midi_engine);
    hid.start_listener(target_ptr, is_external, is_seized, move |usage, is_down| {
        if is_down {
            engine.handle_key_down(usage);
        } else {
            engine.handle_key_up(usage);
        }
    });

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
    }));
}
