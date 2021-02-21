///! Library providing the dependencies for the binary program.
///!
///! Includes MIDI adapter and message parsing, getting info on focused window, and sending key
///! sequences.

pub mod midi;
pub mod focus;
pub mod keyboard_control;
pub mod actions;