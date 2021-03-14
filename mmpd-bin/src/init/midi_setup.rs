use clap::ArgMatches;
use mmpd_lib::match_checker::{StringMatcher, MatchChecker};
use mmpd_lib::midi::adapters::MidiAdapter;
use mmpd_lib::midi;
use mmpd_lib::config::Config;

pub (crate) fn get_midi_setup(
    cli_matches: Option<&ArgMatches>,
    config: Option<&Config>
) -> Option<(Box<dyn MidiAdapter>, String)> {
    let midi_adapter = midi::get_adapter();

    if midi_adapter.is_none() {
        eprintln!("Error: Unable to set up MIDI adapter.");
        return None;
    }

    let midi_adapter = midi_adapter.unwrap();

    let midi_device_name = get_selected_midi_device_name(
        cli_matches,
        config,
        &midi_adapter
    );

    if midi_device_name.is_none() {
        eprintln!("Error: No matching MIDI device found.");
        return None;
    }

    let midi_device_name = midi_device_name.unwrap();

    Some((midi_adapter, midi_device_name))
}

fn get_midi_port_matcher(
    cli_matches: Option<&ArgMatches>,
    config: Option<&Config>
) -> Option<StringMatcher> {
    cli_matches.map(|matches| {
        matches.value_of("midi-device").map_or_else(|| {
            if let Some(config) = config {
                config.midi_device_matcher.clone()
            } else {
                eprintln!("Specify a midi device with --midi-device (part of it is enough)");
                None
            }
        }, |str_pattern| {
            Some(StringMatcher::Contains(str_pattern.to_string()))
        })
    }).flatten()
}

fn get_selected_midi_device_name(
    cli_matches: Option<&ArgMatches>,
    config: Option<&Config>,
    midi_adapter: &Box<dyn MidiAdapter>
) -> Option<String> {

    let midi_port_pattern = get_midi_port_matcher(cli_matches, config);

    if midi_port_pattern.is_none() {
        return None;
    }

    let midi_port_pattern = midi_port_pattern.unwrap();

    let ports = midi_adapter.list_ports();

    ports.iter().find(|d| {
        // Don't even ask, I don't know
        midi_port_pattern.matches(&&***d)
    }).map(|s| s.to_string())
}