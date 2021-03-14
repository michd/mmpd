use clap::ArgMatches;
use crate::init::midi_setup::get_midi_setup;
use crate::init::get_config;
use mmpd_lib::macros::event_matching::{get_event_bus, Event};
use mmpd_lib::midi::MidiMessage;

pub fn task_monitor(cli_matches: Option<&ArgMatches>) {
    let (config, config_filename) = get_config(cli_matches).map_or(
        (None, None),
        |(c,n)| (Some(c), Some(n))
    );

    let midi_setup = get_midi_setup(cli_matches, config.as_ref());

    if midi_setup.is_none() {
        return;
    }

    let (mut midi_adapter, midi_device_name) = midi_setup.unwrap();
    println!("Starting mmpd.");

    if let Some(config_filename) = config_filename {
        println!("Using config file: {}", config_filename);
    }


    let (tx, rx) = get_event_bus();

    let handle = midi_adapter.start_listening(&midi_device_name, tx);

    if let None = handle {
        eprintln!("Unable to start listening for MIDI events.");
        return;
    }

    println!("Monitoring MIDI events on: \n{}\n", midi_device_name);

    for msg in rx {
        if let Event::Midi(msg) = msg {
            match msg {
                MidiMessage::Other => {},
                _ => println!("{:?}", msg)
            }
        }
    }
}

