use std::fs;

use midi_macro_pad_lib::{focus, state};
use midi_macro_pad_lib::config::Config;
use midi_macro_pad_lib::config::input_formats::get_parser_for_extension;
use midi_macro_pad_lib::macros::actions::ActionRunner;
use midi_macro_pad_lib::macros::event_matching::{Event, get_event_bus};
use midi_macro_pad_lib::match_checker::{MatchChecker, StringMatcher};
use midi_macro_pad_lib::midi;

#[macro_use]
extern crate clap;
use clap::{App, ArgMatches};

extern crate directories;
use directories::ProjectDirs;
use std::path::{Path, PathBuf};
use midi_macro_pad_lib::midi::MidiMessage;
use midi_macro_pad_lib::midi::adapters::MidiAdapter;

fn main() {
    let cli_yaml = load_yaml!("cli.yml");
    let cli_matches = App::from_yaml(cli_yaml).get_matches();

    match cli_matches.subcommand_name() {
        Some(subcommand) => {
            let arg_matches = cli_matches.subcommand_matches(subcommand);

            match subcommand {
                "monitor" => task_monitor(arg_matches),
                "list-midi-devices" => task_list_midi_devices(arg_matches),
                _ => {}
            }
        }

        None => task_main(Some(&cli_matches))
    }
}

fn get_midi_port_matcher(cli_matches: Option<&ArgMatches>) -> Option<StringMatcher> {
    cli_matches.map(|matches| {
        matches.value_of("midi-device").map_or_else(|| {
            if let Some(config) = get_config(cli_matches) {
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
    midi_adapter: &Box<dyn MidiAdapter>
) -> Option<String> {

    let midi_port_pattern = get_midi_port_matcher(cli_matches);

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

fn setup_midi(cli_matches: Option<&ArgMatches>) -> Option<(Box<dyn MidiAdapter>, String)> {
    let midi_adapter = midi::get_adapter();

    if midi_adapter.is_none() {
        eprintln!("Error: Unable to set up MIDI adapter.");
        return None;
    }

    let midi_adapter = midi_adapter.unwrap();

    let midi_device_name = get_selected_midi_device_name(cli_matches, &midi_adapter);

    if midi_device_name.is_none() {
        eprintln!("Error: No matching MIDI device found.");
        return None;
    }

    let midi_device_name = midi_device_name.unwrap();

    Some((midi_adapter, midi_device_name))
}


fn task_monitor(cli_matches: Option<&ArgMatches>) {
    let midi_setup= setup_midi(cli_matches);

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

fn task_main(cli_matches: Option<&ArgMatches>) {
    let midi_setup= setup_midi(cli_matches);

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

    // TODO: restructure config loading so it only needs to get loaded once. Currently
    // it can also load and parse config separately to figure out midi device name
    let config = get_config(cli_matches);

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

// Gets a config instance while also figuring out _where_ to get it
fn get_config(cli_matches: Option<&ArgMatches>) -> Option<Config> {
    let default_filenames = vec![
        "mmpd.yml",
        "mmpd.yaml"
    ];

    let config_file = if let Some(Some(cli_config)) = cli_matches.map(|cm| cm.value_of("config")) {
        let path = Path::new(cli_config);

        if path.exists() {
            Some(path.to_path_buf())
        } else {
            eprintln!("Config file not found: {}", cli_config);
            None
        }
    } else {
        if let Some(default_config_dir) = get_project_dir().map(|pd| pd.config_dir().to_path_buf()) {
            if default_config_dir.exists() {
                let mut first_existing: Option<PathBuf> = None;

                let default_paths: Vec<PathBuf> = default_filenames.iter().map(|filename| {
                    default_config_dir.join(Path::new(filename)).to_path_buf()
                }).collect();

                for path in &default_paths {
                    if path.exists() {
                        first_existing = Some(path.to_path_buf());
                        continue;
                    }
                }

                if first_existing.is_none() {
                    eprintln!("Error: No config file found in:");

                    for path in &default_paths {
                        eprintln!("\t{}", path.to_str().unwrap_or(""));
                    }

                    eprintln!("\nEither create one, or specify a config file with --config=<file>");
                }

                first_existing
            } else {
                None
            }
        } else {
            None
        }
    };

    if let Some(config_file) = &config_file {
        if let Ok(config_text) = fs::read_to_string(config_file) {
            let ext = config_file.extension()
                .map(|s| s.to_str().unwrap_or(""))
                .unwrap_or("");

            if let Some(parser) = get_parser_for_extension(ext) {
                match parser.parse(&config_text) {
                    Ok(rc) => {

                        match rc.process() {
                            Ok(config) => Some(config),
                            Err(e) => {
                                eprintln!(
                                    "Error: unable to parse config file {}",
                                    config_file.to_str().unwrap_or("[none]")
                                );

                                eprintln!("{}", e.description());
                                None
                            }
                        }
                    }

                    Err(e) => {
                        eprintln!(
                            "Error: unable to parse config file {}",
                            config_file.to_str().unwrap_or("[none]")
                        );

                        eprintln!("{}", e.description());

                        None
                    }
                }
            } else {
                eprintln!(
                    "Error: unknown config file format {}",
                    ext
                );

                None
            }
        } else {
            eprintln!("Unable to read config file {}", config_file.to_str().unwrap_or("[none]"));
            None
        }
    } else {
        None
    }
}

fn get_project_dir() -> Option<ProjectDirs> {
    // TODO: constants etc
    ProjectDirs::from(
        "me",
        "michd",
        "mmpd"
    )
}


fn task_list_midi_devices(_cli_matches: Option<&ArgMatches>) {
    let midi_adapter = midi::get_adapter();

    if midi_adapter.is_none() {
        eprintln!("Unable to initialize MIDI adapter.");
        return;
    }

    let port_names = midi_adapter.unwrap().list_ports();

    println!("Available MIDI input devices:");

    for port_name in port_names {
        println!("{}", port_name);
    }
}
