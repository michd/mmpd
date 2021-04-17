#[cfg(target_os = "linux")]
mod xdo;

#[cfg(target_os = "linux")]
pub use xdo::get_adapter;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub use windows::get_adapter;

use std::fmt::{self, Display, Formatter};

/// Adapters implementing this trait can send key sequences and text as if they were entered on
/// a keyboard.
#[cfg_attr(test, automock)]
pub trait KeyboardControlAdapter {
    /// Sends a sequence of keys with a delay between keys specified by delay_microsecs.
    /// Format for the sequence is that of X Keysyms.
    /// TODO: some link for a reference of X Keysyms
    fn send_keysequence(&self, sequence: &str, delay_microsecs: u32) -> KeyboardResult;

    /// Sends a sequence of keys with a delay between keys specified by delay_microsecs.
    /// The keys are specified as plain text being typed, rather than a way to describe key
    /// combinations. For key combinations with modifiers like ctrl, alt, shift etc, use
    /// send_keysequence.
    fn send_text(&self, text: &str, delay_microsecs: u32) -> KeyboardResult;
}

/// Result type for KeyboardControlAdapter functions indicating whether they worked correctly
pub type KeyboardResult = Result<(), KeyboardControlError>;

/// Errors that may occur when trying to use a KeyboardControlAdapter
pub enum KeyboardControlError {
    /// An unknown/unsupported/invalid key was passed to `send_keysequence`
    InvalidKey(
        /// The unsupported key string representation
        String
    ),

    /// Any other error
    Other(
        /// Description of the error
        String
    )
}

impl Display for KeyboardControlError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            KeyboardControlError::InvalidKey(s) => {
                write!(f, "Keyboard Control Error: Invalid key '{}'", s)
            }

            KeyboardControlError::Other(description) => {
                write!(f, "Keyboard Control Error: {}", description)
            }
        }
    }
}
