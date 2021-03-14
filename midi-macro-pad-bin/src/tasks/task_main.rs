use clap::ArgMatches;
use crate::init::midi_setup::get_midi_setup;
use midi_macro_pad_lib::{focus, state};
use midi_macro_pad_lib::macros::actions::ActionRunner;
use crate::init::get_config;
use midi_macro_pad_lib::macros::event_matching::get_event_bus;

pub fn task_main(cli_matches: Option<&ArgMatches>) {
    let config = get_config(cli_matches);

    let midi_setup= get_midi_setup(cli_matches, config.as_ref());

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

    if config.is_none() {
        return;
    }

    let config = config.unwrap();

    let (tx, rx) = get_event_bus();
    let handle = midi_adapter.start_listening(midi_device_name, tx);

    if handle.is_none() {
        eprintln!("Error: unable to start listening for MIDI events.");
    }

    println!("Starting mmpd. Have {} configured macros.\n", config.macros.len());

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
