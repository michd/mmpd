pub mod types;
pub mod adapters;

use std::sync::mpsc::{self, SyncSender, Receiver};
use crate::midi::types::MidiMessage;

// Todo: instead of a read method, set up a message bus, giving the midi module just
// the thing to send messages down the stream so it can go ahead and run in its own thread
// No need for a trait for a messenger then.

pub use adapters::get_adapter;

pub fn get_midi_bus() -> (SyncSender<MidiMessage>, Receiver<MidiMessage>) {
    mpsc::sync_channel(1024)
}

