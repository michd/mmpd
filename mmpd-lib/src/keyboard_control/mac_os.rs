use crate::keyboard_control::{KeyboardControlAdapter, KeyboardResult, KeyboardControlError};

pub fn get_adapter() -> Option<Box<impl KeyboardControlAdapter>> {
    MacOs::new().map(|mac_os| Box::new(mac_os))
}

struct MacOs {}

impl MacOs {
    fn new() -> Option<impl KeyboardControlAdapter> {
        Some(MacOs {})
    }
}

impl KeyboardControlAdapter for MacOs {
    fn send_keysequence(&self, _sequence: &str, _delay_microsecs: u32) -> KeyboardResult {
        println!("Todo: send_keysequence in MacOs");
        Ok(())
    }

    fn send_text(&self, _text: &str, _delay_microsecs: u32) -> KeyboardResult {
        println!("Todo: send_text in MacOs");
        Ok(())
    }
}
