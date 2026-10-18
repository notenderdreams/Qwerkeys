use std::error::Error;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use midir::{os::unix::VirtualOutput, MidiOutput, MidiOutputConnection};

use crate::mapping::{calculate_key_notes, find_key_by_usage};
use crate::scale::format_midi_note;
use crate::state::AppState;

pub struct MidiEngine {
    conn: Mutex<MidiOutputConnection>,
    state: Arc<AppState>,
}

impl MidiEngine {
    pub fn new(port_name: &str, state: Arc<AppState>) -> Result<Self, Box<dyn Error>> {
        let midi_out = MidiOutput::new("qwerkeys-client")?;
        let conn = midi_out.create_virtual(port_name)?;

        Ok(Self {
            conn: Mutex::new(conn),
            state,
        })
    }

    pub fn handle_key_down(&self, usage: u32) {
        // Panic shortcut: Escape key (0x29) clears all stuck notes
        if usage == 0x29 {
            self.panic_all_notes();
            if let Ok(mut info) = self.state.last_event_info.lock() {
                *info = "Panic: All Notes Off".to_string();
            }
            return;
        }

        // Sustain Pedal (Spacebar)
        if usage == 0x2C {
            if !self.state.sustain_held.swap(true, Ordering::SeqCst) {
                self.send_cc(64, 127);
                if let Ok(mut info) = self.state.last_event_info.lock() {
                    *info = "Sustain ON".to_string();
                }
                self.state.request_repaint();
            }
            return;
        }

        // In-scale Key mapping
        if let Some(key) = find_key_by_usage(usage) {
            // Ignore OS key repeats for keys already held down
            if let Ok(held) = self.state.held_notes.lock()
                && held.contains_key(&usage)
            {
                return;
            }

            let root = self.state.get_root();
            let scale = self.state.get_scale();
            let left_oct = self.state.left_base_octave.load(Ordering::SeqCst);
            let right_oct = self.state.right_base_octave.load(Ordering::SeqCst);
            let preset = self.state.get_layout_preset();

            let notes = calculate_key_notes(key, root, scale, left_oct, right_oct, preset);
            let vel = self.state.velocity.load(Ordering::SeqCst);

            // Record active state
            if let Ok(mut held) = self.state.held_notes.lock() {
                held.insert(usage, notes.clone());
            }
            if let Ok(mut active) = self.state.active_usages.lock() {
                active.insert(usage);
            }
            if let Ok(mut active_notes) = self.state.active_notes.lock() {
                for &n in &notes {
                    active_notes.insert(n);
                }
            }

            for &n in &notes {
                self.send_raw(&[0x90, n, vel]);
            }

            let note_str = notes
                .iter()
                .map(|&n| format_midi_note(n))
                .collect::<Vec<_>>()
                .join(", ");

            if let Ok(mut info) = self.state.last_event_info.lock() {
                *info = format!("♪ [{}] {note_str}", key.label);
            }
            println!("  ♪ [{}] {note_str:<8}", key.label);

            self.state.request_repaint();
        }
    }

    pub fn handle_key_up(&self, usage: u32) {
        // Sustain Pedal release (Spacebar)
        if usage == 0x2C {
            if self.state.sustain_held.swap(false, Ordering::SeqCst) {
                self.send_cc(64, 0);
                if let Ok(mut info) = self.state.last_event_info.lock() {
                    *info = "Sustain OFF".to_string();
                }
                self.state.request_repaint();
            }
            return;
        }

        // Note release
        let released_notes = if let Ok(mut held) = self.state.held_notes.lock() {
            held.remove(&usage)
        } else {
            None
        };

        if let Some(notes) = released_notes {
            if let Ok(mut active) = self.state.active_usages.lock() {
                active.remove(&usage);
            }
            if let Ok(mut active_notes) = self.state.active_notes.lock() {
                for &n in &notes {
                    active_notes.remove(&n);
                }
            }

            for &n in &notes {
                self.send_raw(&[0x80, n, 64]);
            }
            self.state.request_repaint();
        }
    }

    pub fn trigger_note_on(&self, note: u8) {
        let vel = self.state.velocity.load(Ordering::SeqCst);
        if let Ok(mut notes) = self.state.active_notes.lock() {
            notes.insert(note);
        }
        self.send_raw(&[0x90, note, vel]);
        let note_str = format_midi_note(note);
        if let Ok(mut info) = self.state.last_event_info.lock() {
            *info = format!("♪ {note_str} on (vel {vel})");
        }
        println!("  ♪ {note_str:<4} on");
        self.state.request_repaint();
    }

    pub fn trigger_note_off(&self, note: u8) {
        if let Ok(mut notes) = self.state.active_notes.lock() {
            notes.remove(&note);
        }
        self.send_raw(&[0x80, note, 64]);
        self.state.request_repaint();
    }

    pub fn panic_all_notes(&self) {
        // Send All Notes Off, All Sound Off, and Sustain Off
        self.send_cc(123, 0); // All Notes Off
        self.send_cc(120, 0); // All Sound Off
        self.send_cc(64, 0);  // Sustain Off
        self.state.sustain_held.store(false, Ordering::SeqCst);

        // Send explicit Note Off for all 128 MIDI notes
        for note in 0..128 {
            self.send_raw(&[0x80, note, 0]);
        }

        if let Ok(mut held) = self.state.held_notes.lock() {
            held.clear();
        }
        if let Ok(mut active) = self.state.active_usages.lock() {
            active.clear();
        }
        if let Ok(mut notes) = self.state.active_notes.lock() {
            notes.clear();
        }

        self.state.request_repaint();
    }

    fn send_raw(&self, bytes: &[u8]) {
        if let Ok(mut conn) = self.conn.lock()
            && let Err(e) = conn.send(bytes)
        {
            eprintln!("[qwerkeys] MIDI send error: {e}");
        }
    }

    fn send_cc(&self, cc: u8, val: u8) {
        self.send_raw(&[0xB0, cc, val]);
    }
}
