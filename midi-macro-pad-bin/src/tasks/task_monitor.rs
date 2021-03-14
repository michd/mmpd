use clap::ArgMatches;
use crate::init::midi_setup::get_midi_setup;
use midi_macro_pad_lib::macros::event_matching::{get_event_bus, Event};
use midi_macro_pad_lib::midi::MidiMessage;
use crate::init::get_config;

pub fn task_monitor(cli_matches: Option<&ArgMatches>) {
    let config = get_config(cli_matches);
    let midi_setup = get_midi_setup(cli_matches, config.as_ref());

    if midi_setup.is_none() {
        return;
    }

    let (mut midi_adapter, midi_device_name) = midi_setup.unwrap();

    println!("Monitoring '{}'...", midi_device_name);

    let (tx, rx) = get_event_bus();

    let handle = midi_adapter.start_listening(String::from(midi_device_name), tx);

    if let None = handle {
        eprintln!("Unable to start listening for MIDI events.");
        return;
    }

    for msg in rx {
        if let Event::Midi(msg) = msg {
            match msg {
                MidiMessage::Other => {},
                _ => println!("{:?}", msg)
            }
        }
    }
}

