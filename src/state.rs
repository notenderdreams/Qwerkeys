use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicI8, AtomicU8, AtomicUsize, Ordering};
use std::sync::Mutex;

use crate::iokit::KeyboardDevice;
use crate::scale::{RootNote, ScaleType};

pub struct AppState {
    pub root: Mutex<RootNote>,
    pub scale: Mutex<ScaleType>,
    pub left_base_octave: AtomicI8,
    pub right_base_octave: AtomicI8,
    pub velocity: AtomicU8,
    pub sustain_held: AtomicBool,

    pub devices: Mutex<Vec<KeyboardDevice>>,
    pub selected_device_ptr: AtomicUsize,
    pub is_seized: AtomicBool,

    pub active_usages: Mutex<HashSet<u32>>,
    pub active_notes: Mutex<HashSet<u8>>,
    pub held_notes: Mutex<HashMap<u32, u8>>,

    pub last_event_info: Mutex<String>,
    pub egui_ctx: Mutex<Option<egui::Context>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            root: Mutex::new(RootNote::C),
            scale: Mutex::new(ScaleType::NaturalMinor),
            left_base_octave: AtomicI8::new(2),
            right_base_octave: AtomicI8::new(4),
            velocity: AtomicU8::new(100),
            sustain_held: AtomicBool::new(false),

            devices: Mutex::new(Vec::new()),
            selected_device_ptr: AtomicUsize::new(0),
            is_seized: AtomicBool::new(false),

            active_usages: Mutex::new(HashSet::new()),
            active_notes: Mutex::new(HashSet::new()),
            held_notes: Mutex::new(HashMap::new()),

            last_event_info: Mutex::new("Ready".to_string()),
            egui_ctx: Mutex::new(None),
        }
    }

    pub fn set_egui_ctx(&self, ctx: egui::Context) {
        if let Ok(mut lock) = self.egui_ctx.lock() {
            *lock = Some(ctx);
        }
    }

    pub fn request_repaint(&self) {
        if let Ok(lock) = self.egui_ctx.lock()
            && let Some(ctx) = &*lock
        {
            ctx.request_repaint();
        }
    }

    pub fn get_root(&self) -> RootNote {
        *self.root.lock().unwrap()
    }

    pub fn set_root(&self, root: RootNote) {
        *self.root.lock().unwrap() = root;
        self.request_repaint();
    }

    pub fn get_scale(&self) -> ScaleType {
        *self.scale.lock().unwrap()
    }

    pub fn set_scale(&self, scale: ScaleType) {
        *self.scale.lock().unwrap() = scale;
        self.request_repaint();
    }

    #[allow(dead_code)]
    pub fn get_selected_device(&self) -> Option<KeyboardDevice> {
        let ptr = self.selected_device_ptr.load(Ordering::SeqCst);
        let list = self.devices.lock().unwrap();
        list.iter().find(|d| d.device_ptr == ptr).cloned()
    }

    pub fn set_selected_device(&self, device_ptr: usize) {
        self.selected_device_ptr.store(device_ptr, Ordering::SeqCst);
        self.request_repaint();
    }

    pub fn is_key_active(&self, usage: u32) -> bool {
        if let Ok(active) = self.active_usages.lock() {
            active.contains(&usage)
        } else {
            false
        }
    }

    #[allow(dead_code)]
    pub fn is_note_active(&self, note: u8) -> bool {
        if let Ok(notes) = self.active_notes.lock() {
            notes.contains(&note)
        } else {
            false
        }
    }
}
