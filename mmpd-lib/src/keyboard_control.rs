#[cfg(target_os = "linux")]
mod xdo;

#[cfg(target_os = "linux")]
pub use xdo::get_adapter;


/// Adapters implementing this trait can send key sequences and text as if they were entered on
/// a keyboard.
#[cfg_attr(test, automock)]
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
