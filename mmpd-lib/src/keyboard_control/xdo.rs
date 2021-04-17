extern crate libxdo;

use crate::keyboard_control::{KeyboardControlAdapter, KeyboardResult, KeyboardControlError};
use libxdo::XDo;
use std::error::Error;

pub fn get_adapter() -> Option<Box<impl KeyboardControlAdapter>> {
    Xdo::new().map(|xdo| Box::new(xdo))
}

// Wrapper struct for the libxdo instance, used to access the KeyboardControlAdapter trait methods
struct Xdo {
    xdo: XDo
}

impl Xdo {
    /// Creates a new instance of the Xdo adapter, wrapping libxdo's.
    fn new() -> Option<impl KeyboardControlAdapter> {
        let xdo = XDo::new(None).ok()?;

        Some(Xdo {
            xdo
        })
    }
}

impl KeyboardControlAdapter for Xdo {
    /// Sends a sequence of keys with a delay between keys specified by delay_microsecs.
    /// Format for the sequence is that of X Keysyms.
    /// TODO: some link for a reference of X Keysyms
    fn send_keysequence(&self, sequence: &str, delay_microsecs: u32) -> KeyboardResult {
        self.xdo.send_keysequence(sequence, delay_microsecs).map_err(|err| {
            KeyboardControlError::Other(err.to_string())
        })
    }

    /// Sends a sequence of keys with a delay between keys specified by delay_microsecs.
    /// The keys are specified as plain text being typed, rather than a way to describe key
    /// combinations. For key combinations with modifiers like ctrl, alt, shift etc, use
    /// send_keysequence.
    fn send_text(&self, text: &str, delay_microsecs: u32) -> KeyboardResult {
        self.xdo.enter_text(text, delay_microsecs).map_err(|err| {
            KeyboardControlError::Other(err.to_string())
        })
    }
}

