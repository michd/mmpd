use crate::midi::types::MidiMessage;
use midir::{MidiInput, MidiInputPort};
use std::sync::mpsc::SyncSender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use crate::midi::adapters::MidiAdapter;
use crate::midi::parse_message;

pub struct Midir {
    active: Arc<Mutex<bool>>,
}

const CLIENT_NAME: &str = "Midir client";

impl Midir {
    pub fn new() -> Option<impl MidiAdapter> {
        Some(Midir {
            active: Arc::new(Mutex::new(false)),
        })
    }

    fn get_port(&self, pattern: &str) -> Option<MidiInputPort> {
        let midi_in = MidiInput::new(CLIENT_NAME).ok()?;

        midi_in
            .ports()
            .iter()
            .find(|p|
                midi_in.port_name(p)
                    .unwrap_or(String::from(""))
                    .contains(pattern)
            ).cloned()
    }
}

impl MidiAdapter for Midir {

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

    fn start_listening(
        &mut self,
        port_pattern: String,
        tx: SyncSender<MidiMessage>,
    ) -> Option<thread::JoinHandle<()>> {
        let active = Arc::clone(&self.active);
        let mut is_active = active.lock().unwrap();
        if *is_active {
            return None;
        }
        *is_active = true;

        let active = Arc::clone(&self.active);


        let tx: SyncSender<MidiMessage> = tx.clone();
        let port = self.get_port(&port_pattern)?;

        let midi_in = MidiInput::new(CLIENT_NAME).ok()?;

        let handle = thread::spawn(move || {
            let port_name = midi_in
                .port_name(&port)
                .unwrap_or(String::from("(unknown port)"));

            // Assignment here is necessary to keep the connection alive.
            let _connection = midi_in.connect(
                &port,
                port_name.as_str(),
                move |_, bytes, _| {
                    if let Some(msg) = parse_message(bytes) {
                        let _ = tx.send(msg);
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

    fn stop_listening(&self) {
        let active = Arc::clone(&self.active);
        let mut is_active = active.lock().unwrap();
        *is_active = false;
    }
}

// TODO: move out of here
