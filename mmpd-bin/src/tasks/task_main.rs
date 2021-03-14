use clap::ArgMatches;
use mmpd_lib::{focus, state};
use mmpd_lib::macros::actions::ActionRunner;
use mmpd_lib::macros::event_matching::get_event_bus;
use crate::init::get_config;
use crate::init::midi_setup::get_midi_setup;

pub fn task_main(cli_matches: Option<&ArgMatches>) {
    let config = get_config(cli_matches);

    if config.is_none() {
        return;
    }

    let (config, config_filename) = config.unwrap();

    let midi_setup= get_midi_setup(cli_matches, Some(&config));

    if midi_setup.is_none() {
        return;
    }

    let (mut midi_adapter, midi_device_name) = midi_setup.unwrap();

    let focus_adapter = focus::get_adapter();

    if focus_adapter.is_none() {
        eprintln!("Unable to set up focus adapter - can't detect focused window.");
        return;
    }

    let focus_adapter = focus_adapter.unwrap();

    let action_runner = ActionRunner::new();

    if action_runner.is_none() {
        eprintln!("Unable to get an action runner.");
        return;
    }

    let action_runner = action_runner.unwrap();
    let state = state::new(focus_adapter);

    let (tx, rx) = get_event_bus();
    let handle = midi_adapter.start_listening(&midi_device_name, tx);

    if handle.is_none() {
        eprintln!("Error: unable to start listening for MIDI events.");
    }

    println!("Starting mmpd.");
    println!("Using config file: {}", config_filename);
    println!("Listening for MIDI events on '{}'", midi_device_name);
    let macro_count = config.macros.len();

    if macro_count == 1 {
        println!("There is 1 configured macro.");
    } else {
        println!("There are {} configured macros.", macro_count);
    }

    if config.macros.is_empty() {
        println!("\nYou can set up some macros by editing the config file.");
        println!("Find documentation on the config file format here:");
        println!("https://github.com/michd/mmpd/blob/main/docs/config.md");

        println!("\nSince there are no macros configured, there is nothing to do; exiting.");
    }

    for event in rx {
        for macro_item in config.macros.iter() {
            if let Some(actions) = macro_item.evaluate(&event, &state) {
                if let Some(macro_name) = macro_item.name() {
                    println!("Executing macro named: '{}'", macro_name);
                } else {
                    println!("Executing macro. (No name given)");
                }

                for action in actions {
                    // TODO: action_runner.run could return any actions that are for this main
                    // application to evaluate, for control actions.
                    action_runner.run(action);
                }

                break;
            }
        }
    }
}
