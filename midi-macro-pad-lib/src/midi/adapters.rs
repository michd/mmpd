use std::sync::mpsc::SyncSender;
use std::thread;
use crate::midi::adapters::midir::Midir;
use crate::macros::event_matching::Event;

mod midir;

/// Provides an adapter implementing MidiAdapter based on platform
/// At the moment it just provides the Midir implementation.
pub fn get_adapter() -> Option<Box<dyn MidiAdapter>> {
    // TODO: select what to use based on platform, if needed

    let adapter = Midir::new();

    match adapter {
        Some(a) => Some(Box::new(a)),
        None => None
    }
}

/// Adapters implementing this trait can be used by the binary to get a list of available
/// MIDI inputs, as well as instructed to start listening for MIDI messages on a port matching
/// a pattern.
pub trait MidiAdapter {
    /// Queries the implementation for available MIDI inputs and returns them as a list of Strings.
    /// These strings can be used to match a pattern against in `start_listening`'s port_pattern
    /// parameter.
    fn list_ports(&self) -> Vec<String>;

    /// Instructs the implementation to start a thread listening for incoming MIDI messages,
    /// providing a SyncSender to send received messages down. The implementation must present
    /// incoming messages as MidiMessage structs, and can use the `parse_message` function to
    /// convert from raw bytes to this struct.
    ///
    /// Returns None if for any reason we cannot start listening; otherwise returns a thread join
    /// handle.
    fn start_listening(
        &mut self,
        port_pattern: String,
        tx: SyncSender<Event>
    ) -> Option<thread::JoinHandle<()>>;

    /// Instructs the implementation to abort the thread on which it is listening for incoming
    /// messages, if any.
    fn stop_listening(&self);
}

