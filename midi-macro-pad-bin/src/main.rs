use std::env;
use std::vec::Vec;

use midi_macro_pad_lib::focus;
use midi_macro_pad_lib::macros::actions::{Action, ActionRunner};
use midi_macro_pad_lib::macros::event_matching::midi::MidiEventMatcher;
use midi_macro_pad_lib::match_checker::{MatchChecker, NumberMatcher};
use midi_macro_pad_lib::midi;
use midi_macro_pad_lib::macros::Macro;
use midi_macro_pad_lib::macros::event_matching::{EventMatcher, Event};

fn main() {
    println!("MIDI Macro Pad starting.");
    let args: Vec<String> = env::args().collect();

    println!("Running with args:\n{:?}", args);

    if let Some(cmd) = args.get(1) {
        match cmd.as_str() {
            "list-ports" => task_list_ports(),
            "listen" => task_listen(args.get(2)),

            _ => {
                eprintln!("Unrecognised argument '{}'", cmd);
                return;
            }
        }

        return;
    }

    // TODO: if no command is specified, load config file from default location
    // TODO: otherwise, allow specifying config file from args too and use that

    println!("Config file loading not yet implemented, exiting.");
}

/// Prints a list of all available MIDI input devices connected to this computer to STDOUT.
///
/// If the MIDI adapter cannot be initialized, prints an error.
///
/// The output of this is useful for specifying a port to listen to, see `task_listen`.
fn task_list_ports() {
    let midi_adapter = midi::get_adapter();

    if let None = midi_adapter {
        eprintln!("Unable to initialize MIDI adapter.");
        return;
    }

    let port_names = midi_adapter.unwrap().list_ports();

    println!("Available midi ports:");

    for port_name in port_names.iter() {
        println!("{}", port_name);
    }
}

/// Opens a connection on a port which' name contains port_pattern and begins listening for
/// MIDI messages.
///
/// Each message will be parsed and printed to STDOUT.
///
/// Some filters are hardcoded at the moment and will execute a key sequence when it occurs.
fn task_listen(port_pattern: Option<&String>) -> () {
    if let None = port_pattern {
        eprintln!("No port pattern specified");
        return ();
    }

    let port_pattern = port_pattern.unwrap();

    let (tx, rx) = midi::get_midi_bus();

    let midi_adapter = midi::get_adapter();

    if let None = midi_adapter {
        eprintln!("Unable to set up midi adapter");
        return;
    }

    let mut midi_adapter = midi_adapter.unwrap();

    let focus_adapter = focus::get_adapter();

    if let None = focus_adapter {
        eprintln!("Unable to set up focus adapter - can't detect focused window.");
        return;
    }

    let focus_adapter = focus_adapter.unwrap();

    let handle = midi_adapter.start_listening(String::from(port_pattern), tx);

    if let None = handle {
        eprintln!("Unable to start listening for MIDI events.");
        return;
    }

    let action_runner = ActionRunner::new();

    if let None = action_runner {
        eprintln!("Unable to get an action runner.");
        return;
    }

    let action_runner = action_runner.unwrap();

    let inkscape_macro = Macro::new(
        vec![
            Box::new(EventMatcher::Midi(
                Box::new(MidiEventMatcher::NoteOn {
                    channel_match: Some(NumberMatcher::Val(0)),
                    key_match: Some(NumberMatcher::Val(48)),
                    velocity_match: None
                })
            ))
        ],
        vec![
            Action::KeySequence("ctrl+shift+a", 1),
            Action::KeySequence("Tab", 6),
            Action::KeySequence("Return", 1),
        ]
    );

    let hello_world_macro = Macro::new(
        vec![
            Box::new(EventMatcher::Midi(
                Box::new(MidiEventMatcher::NoteOn {
                    channel_match: Some(NumberMatcher::Val(0)),
                    key_match: Some(NumberMatcher::Val(60)),
                    velocity_match: Some(NumberMatcher::Range { min: Some(63), max: None })
                })
            ))
        ],

        vec![Action::EnterText("Hello world!", 1)]
    );

    let ctrl_c_macro = Macro::new(
        vec![
            Box::new(EventMatcher::Midi(
              Box::new(MidiEventMatcher::NoteOn {
                  channel_match: Some(NumberMatcher::Val(0)),
                  key_match: Some(NumberMatcher::Val(61)),
                  velocity_match: None
              })
            ))
        ],

        vec![Action::KeySequence("ctrl+c", 1)]
    );

    let macro_list = vec![hello_world_macro, ctrl_c_macro];

    let stop_matcher = MidiEventMatcher::ControlChange {
        channel_match: None,
        control_match: Some(NumberMatcher::Val(51)),
        value_match: Some(NumberMatcher::Val(127))
    };

    for msg in rx {
        println!("{:?}", msg);

        let event = Event::Midi(&msg);

        if let Some(actions) = inkscape_macro.evaluate(&event) {
            let fw = focus_adapter.get_focused_window().unwrap();
            if !fw.window_name.ends_with("Inkscape") {
                println!("Not in inkscape, skipping");
                continue;
            }
            println!("in inkscape, executing macro");
            for action in actions {
                action_runner.run(action);
            }

            continue;
        }

        for macro_item in macro_list.iter() {
            if let Some(actions) = macro_item.evaluate(&event) {
                for action in actions {
                    action_runner.run(action);
                }

                break;
            }
        }

        if stop_matcher.matches(&msg) {
            midi_adapter.stop_listening();
        }
    }

    println!("Exiting.");
}
