use crate::keyboard_control::adapters::xdo::Xdo;

mod xdo;

pub fn get_adapter() -> Option<Box<dyn KeyboardControlAdapter>> {
    // TODO: select what to use based on platform, if needed

    let adapter = Xdo::new();

    match adapter {
        Some(a) => Some(Box::new(a)),
        None => None
    }
}

pub trait KeyboardControlAdapter {
    fn send_keysequence(&self, sequence: &str, delay_microsecs: u32);
    fn send_text(&self, text: &str, delay_microsecs: u32);
}