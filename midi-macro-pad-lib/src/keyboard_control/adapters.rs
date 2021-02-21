use crate::keyboard_control::adapters::xdo::Xdo;

mod xdo;

/// Provides an adapter implementing KeyboardControlAdapter based on platform
/// At the moment it just provides the xdo implementation.
pub fn get_adapter() -> Option<Box<dyn KeyboardControlAdapter>> {
    // TODO: select what to use based on platform, if needed

    let adapter = Xdo::new();

    match adapter {
        Some(a) => Some(Box::new(a)),
        None => None
    }
}

/// Adapters implementing this trait can send key sequences and text as if they were entered on
/// a keyboard.
pub trait KeyboardControlAdapter {
    /// Sends a sequence of keys with a delay between keys specified by delay_microsecs.
    /// Format for the sequence is that of X Keysyms.
    /// TODO: some link for a reference of X Keysyms
    fn send_keysequence(&self, sequence: &str, delay_microsecs: u32);

    /// Sends a sequence of keys with a delay between keys specified by delay_microsecs.
    /// The keys are specified as plain text being typed, rather than a way to describe key
    /// combinations. For key combinations with modifiers like ctrl, alt, shift etc, use
    /// send_keysequence.
    fn send_text(&self, text: &str, delay_microsecs: u32);
}