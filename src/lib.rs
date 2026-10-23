//! qwerkeys — keyboard-to-MIDI converter for macOS.
//!
//! [`app::run`] is the single public entry point everything else is
//! internal implementation detail.

pub mod app;
pub(crate) mod iokit;
pub(crate) mod mapping;
pub(crate) mod midi;
pub(crate) mod permissions;
pub(crate) mod scale;
pub(crate) mod settings;
pub(crate) mod state;
pub(crate) mod ui;
pub(crate) mod utils;
