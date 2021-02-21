extern crate libxdo;

use crate::keyboard_control::adapters::KeyboardControlAdapter;
use libxdo::XDo;

pub struct Xdo {
    xdo: XDo
}

impl Xdo {
    pub fn new() -> Option<impl KeyboardControlAdapter> {
        let xdo = XDo::new(None).ok()?;

        Some(Xdo {
            xdo
        })
    }
}

impl KeyboardControlAdapter for Xdo {
    fn send_keysequence(&self, sequence: &str, delay_microsecs: u32) {
        // Note: swallowing potential error
        let _ = self.xdo.send_keysequence(sequence, delay_microsecs);
    }

    fn send_text(&self, text: &str, delay_microsecs: u32) {
        // Note: swallowing potential error
        let _ = self.xdo.enter_text(text, delay_microsecs);
    }
}

