use clap::ArgMatches;
use mmpd_lib::{focus, state};
use mmpd_lib::macros::actions::{ActionRunner, ControlAction};
use mmpd_lib::macros::event_matching::{get_event_bus, Event};
use crate::init::{get_config_file, read_config};
use crate::init::midi_setup::get_midi_setup;
use std::sync::mpsc::Receiver;
use mmpd_lib::config::Config;
use mmpd_lib::state::State;
use std::path::PathBuf;

pub fn task_main(cli_matches: Option<&ArgMatches>) -> bool {
    let config_file = get_config_file(cli_matches);
    if config_file.is_none() {
        return false;
    }

    let config_file = config_file.unwrap();
    let config = read_config(config_file.to_path_buf());

    if config.is_none() {
        return false;
    }

    let config = config.unwrap();

    let midi_setup= get_midi_setup(cli_matches, Some(&config));

    if midi_setup.is_none() {
        return false;
    }

    let (mut midi_adapter, midi_device_name) = midi_setup.unwrap();

    let focus_adapter = focus::get_adapter();

    if focus_adapter.is_none() {
        eprintln!("Unable to set up focus adapter - can't detect focused window.");
        return false;
    }

    let focus_adapter = focus_adapter.unwrap();

    let action_runner = ActionRunner::new();

    if action_runner.is_none() {
        eprintln!("Unable to get an action runner.");
        return false;
    }

    let action_runner = action_runner.unwrap();

    let (tx, rx) = get_event_bus();
    let handle = midi_adapter.start_listening(&midi_device_name, tx);

    if handle.is_none() {
        eprintln!("Error: unable to start listening for MIDI events.");
    }

    let config_filename = config_file.to_str().unwrap_or("[none]");
    println!("Starting mmpd.");
    println!("Using config file: {}", config_filename);
    println!("Listening for MIDI events on '{}'", midi_device_name);

    print_macro_info(&config);

    if config.macros.is_empty() {
        return false;
    }

    // Now we've verified all the required data and conditions, we can kick off the main loop that
    // does the work.
    return main_loop(
        config_file.to_path_buf(),
        config,
        state::new(focus_adapter),
        rx,
        action_runner
    );
}

/// Prints info and help on macros found in config
fn print_macro_info(config: &Config) {
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
}

fn main_loop(
    config_file: PathBuf,
    mut config: Config,
    mut state: Box<dyn State>,
    rx: Receiver<Event>,
    action_runner: ActionRunner
)-> bool {

    let mut should_stop_rx_loop = false;
    let mut should_restart = false;
    let mut should_reload_config = false;

    for event in rx {
        state.process_event(&event);

        for macro_item in config.macros.iter() {
            if let Some(actions) = macro_item.evaluate(&event, &state) {
                if let Some(macro_name) = macro_item.name() {
                    println!("Executing macro named: '{}'", macro_name);
                } else {
                    println!("Executing macro. (No name given)");
                }

                for action in actions {
                    let control_action = action_runner.run(action);

                    if let Some(control_action) = control_action {
                        match control_action {
                            ControlAction::ReloadMacros => {
                                println!("Reloading macros from file");
                                should_reload_config = true;
                            }

                            ControlAction::Restart => {
                                println!("Restarting.");
                                should_stop_rx_loop = true;
                                should_restart = true;
                            }

                            ControlAction::Exit => {
                                println!("Exiting.");
                                should_stop_rx_loop = true;
                                should_restart = false;
                            }
                        }
                    }
                }

                break;
            }
        }

        if should_reload_config {
            should_reload_config = false;
            let new_config = read_config(config_file.to_path_buf());

            match new_config {
                Some(new_config) => {
                    config = new_config;
                    println!("Reloaded config.");

                    print_macro_info(&config);

                    if config.macros.is_empty() {
                        // No macros found, exit. print_macro_info prints a message to that
                        // effect too.
                        should_stop_rx_loop = true;
                    }
                }

                None => eprintln!(
                    "Failed to reload configured macros, \
                    using previously loaded config's macros instead."
                )
            }
        }

        if should_stop_rx_loop {
            break;
        }
    }

    return should_restart;
}
