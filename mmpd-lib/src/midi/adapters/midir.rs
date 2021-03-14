use midir::{MidiInput, MidiInputPort};
use std::sync::mpsc::SyncSender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::midi::adapters::MidiAdapter;
use crate::midi::parse_message;
use crate::macros::event_matching::Event;

/// Handle for interfacing with the implementation from outside
pub struct Midir {
    /// Whether we are currently listening for incoming messages
    /// Set to false to stop.
    active: Arc<Mutex<bool>>,
}

/// Client name provided to Midir library when creating a new instance
const CLIENT_NAME: &str = "Midir client";

impl Midir {
    /// Creates a new instance of Midir
    pub fn new() -> Option<impl MidiAdapter> {
        Some(Midir {
            active: Arc::new(Mutex::new(false)),
        })
    }

    /// Searches available MIDI inputs for a port which' name includes the contents of `pattern`.
    /// If no such port can be found, or the Midir library can't instantiate, returns None,
    /// otherwise returns a MidiInputPort.
    ///
    /// If more than one MIDI input port matches the pattern, the first one encountered will be
    /// used.
    fn get_port(&self, pattern: &str) -> Option<MidiInputPort> {
        let midi_in = MidiInput::new(CLIENT_NAME).ok()?;

        midi_in
            .ports()
            .iter()
            .find(|p|
                midi_in.port_name(p)
                    .unwrap_or(String::from(""))
                    .contains(pattern)
            )
            .cloned()
    }
}

impl MidiAdapter for Midir {

    /// Queries Midir for available MIDI inputs and returns them as a list of Strings.
    /// These strings can be used to match a pattern against in `start_listening`'s port_pattern
    /// parameter.
    fn list_ports(&self) -> Vec<String> {
        let midi_in = MidiInput::new(CLIENT_NAME);

        if let Err(_e) = midi_in {
            return Vec::new();
        }

        let midi_in = midi_in.unwrap();
        let ports = midi_in.ports();

        ports
            .iter()
            .map(|p| {
                midi_in
                    .port_name(p)
                    .unwrap_or(String::from("(unknown port)"))
            })
            .collect()
    }


    /// Starts a thread listening for incoming MIDI messages, sending incoming MIDI messages as
    /// MidiMessage structs along tx.
    ///
    /// Returns None if for any reason we cannot start listening; otherwise returns a thread join
    /// handle. Further returns None if we're already listening.
    fn start_listening(
        &mut self,
        port_pattern: String,
        tx: SyncSender<Event>,
    ) -> Option<thread::JoinHandle<()>> {
        let active = Arc::clone(&self.active);
        let mut is_active = active.lock().unwrap();
        if *is_active {
            return None;
        }
        *is_active = true;

        let active = Arc::clone(&self.active);

        let tx = tx.clone();
        let port = self.get_port(&port_pattern)?;

        let midi_in = MidiInput::new(CLIENT_NAME).ok()?;

        let handle = thread::spawn(move || {
            let port_name = midi_in
                .port_name(&port)
                .unwrap_or(String::from("(unknown port)"));

            // Assignment here is necessary to keep the connection alive, since Midir unsubscribes
            // the callback when _connection is destroyed. If it wasn't assigned to anything, it
            // would be destroyed as soon as the `midi_in.connect` call concludes, not when the
            // scope of this thread ends.
            let _connection = midi_in.connect(
                &port,
                port_name.as_str(),
                move |_, bytes, _| {
                    if let Some(msg) = parse_message(bytes) {
                        let _ = tx.send(Event::Midi(msg));
                    }
                },
                (),
            );

            // Keep the thread alive until stop() is called
            loop {
                thread::sleep(Duration::from_micros(100));
                let is_active = active.lock().unwrap();

                if !*is_active {
                    break;
                }
            }
        });

        Some(handle)
    }


    /// Instructs the implementation to abort the thread on which it is listening for incoming
    /// messages, if any.
    fn stop_listening(&self) {
        let active = Arc::clone(&self.active);
        let mut is_active = active.lock().unwrap();
        *is_active = false;
    }
}
