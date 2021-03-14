use clap::ArgMatches;
use mmpd_lib::midi;

pub (crate) fn task_list_midi_devices(_cli_matches: Option<&ArgMatches>) {
    let midi_adapter = midi::get_adapter();

    if midi_adapter.is_none() {
        eprintln!("Unable to initialize MIDI adapter.");
        return;
    }

    let port_names = midi_adapter.unwrap().list_ports();

    println!("Available MIDI input devices:\n");

    for port_name in port_names {
        println!("{}", port_name);
    }
}
