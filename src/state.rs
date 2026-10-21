use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicI8, AtomicU8, AtomicUsize, Ordering};
use std::sync::Mutex;

use crate::iokit::KeyboardDevice;
use crate::mapping::LayoutPreset;
use crate::scale::{RootNote, ScaleType};

pub struct AppState {
    pub root: Mutex<RootNote>,
    pub scale: Mutex<ScaleType>,
    pub layout_preset: Mutex<LayoutPreset>,
    pub base_octave: AtomicI8,
    pub mapping_padding: AtomicI8,
    pub left_base_octave: AtomicI8,
    pub right_base_octave: AtomicI8,
    pub velocity: AtomicU8,
    pub sustain_held: AtomicBool,
    pub show_unmapped_note_names: AtomicBool,

    pub devices: Mutex<Vec<KeyboardDevice>>,
    pub selected_device_ptr: AtomicUsize,

    pub active_usages: Mutex<HashSet<u32>>,
    pub active_notes: Mutex<HashSet<u8>>,
    pub held_notes: Mutex<HashMap<u32, Vec<u8>>>,

    pub last_event_info: Mutex<String>,
    pub egui_ctx: Mutex<Option<egui::Context>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            root: Mutex::new(RootNote::C),
            scale: Mutex::new(ScaleType::NaturalMinor),
            layout_preset: Mutex::new(LayoutPreset::LinearSplit),
            base_octave: AtomicI8::new(4),
            mapping_padding: AtomicI8::new(0),
            left_base_octave: AtomicI8::new(3),
            right_base_octave: AtomicI8::new(5),
            velocity: AtomicU8::new(100),
            sustain_held: AtomicBool::new(false),
            show_unmapped_note_names: AtomicBool::new(true),

            devices: Mutex::new(Vec::new()),
            selected_device_ptr: AtomicUsize::new(0),

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

    pub fn get_layout_preset(&self) -> LayoutPreset {
        *self.layout_preset.lock().unwrap()
    }

    pub fn set_layout_preset(&self, preset: LayoutPreset) {
        *self.layout_preset.lock().unwrap() = preset;
        self.request_repaint();
    }

    pub fn set_base_octave(&self, oct: i8) {
        let clamped = oct.clamp(0, 7);
        self.base_octave.store(clamped, Ordering::SeqCst);
        self.left_base_octave.store(clamped - 1, Ordering::SeqCst);
        self.right_base_octave.store(clamped, Ordering::SeqCst);
        self.request_repaint();
    }

    pub fn set_mapping_padding(&self, pad: i8) {
        let clamped = pad.clamp(-7, 7);
        self.mapping_padding.store(clamped, Ordering::SeqCst);
        self.request_repaint();
    }

    pub fn get_show_unmapped_note_names(&self) -> bool {
        self.show_unmapped_note_names.load(Ordering::SeqCst)
    }

    pub fn set_show_unmapped_note_names(&self, show: bool) {
        self.show_unmapped_note_names.store(show, Ordering::SeqCst);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapping_padding_clamp() {
        let state = AppState::new();
        state.set_mapping_padding(10);
        assert_eq!(state.mapping_padding.load(Ordering::SeqCst), 7);

        state.set_mapping_padding(-10);
        assert_eq!(state.mapping_padding.load(Ordering::SeqCst), -7);

        state.set_mapping_padding(3);
        assert_eq!(state.mapping_padding.load(Ordering::SeqCst), 3);
    }
}
