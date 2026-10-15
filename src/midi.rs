use std::collections::HashSet;
use std::error::Error;
use std::sync::atomic::{AtomicBool, AtomicI8, AtomicU8, Ordering};
use std::sync::Mutex;

use midir::{os::unix::VirtualOutput, MidiOutput, MidiOutputConnection};

use crate::mapping::{format_note_name, usage_to_midi_base};

pub struct MidiEngine {
    conn: Mutex<MidiOutputConnection>,
    active_notes: Mutex<HashSet<u8>>,
    octave_offset: AtomicI8,
    velocity: AtomicU8,
    sustain_held: AtomicBool,
}

impl MidiEngine {
    pub fn new(port_name: &str) -> Result<Self, Box<dyn Error>> {
        let midi_out = MidiOutput::new("nekoto-client")?;
        let conn = midi_out.create_virtual(port_name)?;

        Ok(Self {
            conn: Mutex::new(conn),
            active_notes: Mutex::new(HashSet::new()),
            octave_offset: AtomicI8::new(0),
            velocity: AtomicU8::new(100),
            sustain_held: AtomicBool::new(false),
        })
    }

    pub fn handle_key_down(&self, usage: u32) {
        // Octave down (Z)
        if usage == 0x1D {
            let current = self.octave_offset.load(Ordering::SeqCst);
            if current > -3 {
                let next = current - 1;
                self.octave_offset.store(next, Ordering::SeqCst);
                println!("  [octave] {:+}", next);
            }
            return;
        }

        // Octave up (X)
        if usage == 0x1B {
            let current = self.octave_offset.load(Ordering::SeqCst);
            if current < 3 {
                let next = current + 1;
                self.octave_offset.store(next, Ordering::SeqCst);
                println!("  [octave] {:+}", next);
            }
            return;
        }

        // Velocity down (C)
        if usage == 0x06 {
            let current = self.velocity.load(Ordering::SeqCst);
            let next = current.saturating_sub(10).max(10);
            self.velocity.store(next, Ordering::SeqCst);
            println!("  [velocity] {}", next);
            return;
        }

        // Velocity up (V)
        if usage == 0x19 {
            let current = self.velocity.load(Ordering::SeqCst);
            let next = (current + 10).min(127);
            self.velocity.store(next, Ordering::SeqCst);
            println!("  [velocity] {}", next);
            return;
        }

        // Sustain pedal on (Space)
        if usage == 0x2C {
            if !self.sustain_held.swap(true, Ordering::SeqCst) {
                self.send_cc(64, 127);
                println!("  [sustain] on");
            }
            return;
        }

        // Note triggers
        if let Some(base_note) = usage_to_midi_base(usage) {
            let octave = self.octave_offset.load(Ordering::SeqCst);
            let note = (base_note as i16 + (octave as i16 * 12)).clamp(0, 127) as u8;

            if let Ok(mut active) = self.active_notes.lock() {
                if active.insert(note) {
                    let vel = self.velocity.load(Ordering::SeqCst);
                    self.send_raw(&[0x90, note, vel]);
                    println!("  ♪ {} on (vel {})", format_note_name(note), vel);
                }
            }
        }
    }

    pub fn handle_key_up(&self, usage: u32) {
        // Sustain pedal off (Space)
        if usage == 0x2C {
            if self.sustain_held.swap(false, Ordering::SeqCst) {
                self.send_cc(64, 0);
                println!("  [sustain] off");
            }
            return;
        }

        // Note release
        if let Some(base_note) = usage_to_midi_base(usage) {
            let octave = self.octave_offset.load(Ordering::SeqCst);
            let note = (base_note as i16 + (octave as i16 * 12)).clamp(0, 127) as u8;

            if let Ok(mut active) = self.active_notes.lock() {
                if active.remove(&note) {
                    self.send_raw(&[0x80, note, 64]);
                    println!("  ♪ {} off", format_note_name(note));
                }
            }
        }
    }

    fn send_raw(&self, bytes: &[u8]) {
        if let Ok(mut conn) = self.conn.lock() {
            let _ = conn.send(bytes);
        }
    }

    fn send_cc(&self, cc: u8, val: u8) {
        self.send_raw(&[0xB0, cc, val]);
    }
}
