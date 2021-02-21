use std::sync::mpsc::SyncSender;
use crate::midi::MidiMessage;
use std::thread;
use crate::midi::adapters::midir::Midir;

mod midir;

pub fn get_adapter() -> Option<Box<dyn MidiAdapter>> {
    // TODO: select what to use based on platform, if needed

    let adapter = Midir::new();

    match adapter {
        Some(a) => Some(Box::new(a)),
        None => None
    }
}

pub trait MidiAdapter {
    fn list_ports(&self) -> Vec<String>;

    fn start_listening(
        &mut self,
        port_pattern: String,
        tx: SyncSender<MidiMessage>
    ) -> Option<thread::JoinHandle<()>>;

    fn stop_listening(&self);
}

