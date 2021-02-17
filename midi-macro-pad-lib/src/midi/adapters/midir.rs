use crate::midi::types::MidiMessage;
use num_derive::FromPrimitive;
use midir::MidiInput;
use std::sync::mpsc::SyncSender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use num_traits::FromPrimitive;

#[derive(FromPrimitive)]
enum ChannelMessageType {
    NoteOff = 0b1000isize,
    NoteOn = 0b1001isize,
    PolyAftertouch = 0b1010isize,
    ControlChange = 0b1011isize,
    ProgramChange = 0b1100isize,
    ChannelAfterTouch = 0b1101isize,
    PitchBendChange = 0b1110isize,
    System = 0b1111isize,
}

pub struct Midir {
    active: Arc<Mutex<bool>>,
}

impl Midir {
    pub fn new() -> Midir {
        Midir {
            active: Arc::new(Mutex::new(false)),
        }
    }

    pub fn start(&mut self, tx: SyncSender<MidiMessage>) -> Option<thread::JoinHandle<()>> {
        let active = Arc::clone(&self.active);
        let mut is_active = active.lock().unwrap();
        if *is_active { return None; }
        *is_active = true;

        let active = Arc::clone(&self.active);

        let tx: SyncSender<MidiMessage> = tx.clone();

        let handle = thread::spawn(move || {

            let midi_in = MidiInput::new("MIDI input").unwrap();
            let ports = midi_in.ports();

            // TODO: dynamically pick the port
            let port = ports.get(3).unwrap();
            let port_name = midi_in.port_name(port)
                .unwrap_or(String::from("(unknown port)"));

            // Assignment here is necessary to keep the connection alive.
            let _connection = midi_in.connect(
                port,
                port_name.as_str(),
                move | _, bytes, _| {
                    if let Some(msg) = parse_message(bytes) {
                        let _ = tx.send(msg);
                    }
                },
                ()
            );

            // Keep the thread alive until stop() is called
            loop {
                thread::sleep(Duration::from_micros(100));
                let is_active = active.lock().unwrap();

                if !*is_active {
                    break;
                }
            }

            println!("Stopping in thread");
        });

        Some(handle)
    }

    pub fn stop(&self) {
        let active = Arc::clone(&self.active);
        let mut is_active = active.lock().unwrap();
        *is_active = false;
    }
}

fn parse_message(bytes: &[u8]) -> Option<MidiMessage> {
    let status = *bytes.get(0)?;

    let chan: u8 = status & 0x0F;

    return match FromPrimitive::from_u8((status & 0xF0) >> 4) {
        Some(ChannelMessageType::NoteOff) => {
            Some(MidiMessage::NoteOff {
                channel: chan,
                key: *bytes.get(1)?,
                velocity: *bytes.get(2)?
            })
        }

        Some(ChannelMessageType::NoteOn) => Some(MidiMessage::NoteOn {
            channel: chan,
            key: *bytes.get(1)?,
            velocity: *bytes.get(2)?
        }),


        Some(ChannelMessageType::PolyAftertouch) => Some(MidiMessage::PolyAftertouch {
            channel: chan,
            key: *bytes.get(1)?,
            value: *bytes.get(2)?
        }),

        Some(ChannelMessageType::ControlChange) => Some(MidiMessage::ControlChange {
            channel: chan,
            control: *bytes.get(1)?,
            value: *bytes.get(2)?
        }),

        Some(ChannelMessageType::ProgramChange) => Some(MidiMessage::ProgramChange {
            channel: chan,
            program: *bytes.get(1)?
        }),

        Some(ChannelMessageType::ChannelAfterTouch) => Some(MidiMessage::ChannelAftertouch {
            channel: chan,
            value: *bytes.get(1)?
        }),

        Some(ChannelMessageType::PitchBendChange) => Some(MidiMessage::PitchBendChange {
            channel: chan,
            value: ((*bytes.get(1)? & 0b01111111u8) as u16) | (((*bytes.get(2)? as u16 & 0b01111111u16) << 7) as u16)
        }),

        Some(ChannelMessageType::System) => Some(MidiMessage::Other),
        None => None,
    }
}
