//! Library providing the dependencies for the binary program.
//!
//! Includes MIDI adapter and message parsing, getting info on focused window, sending key sequences,
//! and config parsing facilities.

pub mod midi;
pub mod focus;
pub mod keyboard_control;
pub mod macros;
mod shell;
pub mod match_checker;
pub mod state;

pub mod config;
