use std::vec::Vec;
use std::env;
use midi_macro_pad_lib::midi::{self, MidiMessage};
use midi_macro_pad_lib::focus;
use midi_macro_pad_lib::keyboard_control;

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

    let kb_adapter = keyboard_control::get_adapter();

    if let None = kb_adapter {
        eprintln!("Unable to set up keyboard adapter - can't send key sequences.");
        return;
    }

    let kb_adapter = kb_adapter.unwrap();

    let handle = midi_adapter.start_listening(String::from(port_pattern), tx);

    if let None = handle {
        eprintln!("Unable to start listening for MIDI events.");
    }

    for msg in rx {
        println!("{:?}", msg);

        if let MidiMessage::NoteOff { channel: _, key, velocity: _ } = msg {
            match key {
                // Some hardcoded test actions for now
                48 => {
                    // TODO: handle None
                    let fw = focus_adapter.get_focused_window().unwrap();
                    if fw.window_name.ends_with("Inkscape") {
                        println!("in inkscape, executing centre on horizontal axis.");
                        kb_adapter.send_keysequence("ctrl+shift+a", 100);
                        for _ in 0..6 {
                            kb_adapter.send_keysequence("Tab", 100);
                        }
                        kb_adapter.send_keysequence("Return", 100);
                    } else {
                        println!("not in inkscape, doing nothing.");
                    }
                }

                60 => { kb_adapter.send_text("Hello world!", 250); }

                61 => { kb_adapter.send_keysequence("ctrl+c", 0); }

                62 => {
                    let fw = focus_adapter.get_focused_window();
                    println!("focused window: {:?}", fw);
                }

                _ => {
                    println!("no action configured");
                }
            }
        }

        // "Stop" button on Arturia Keystep, exit the program
        if let MidiMessage::ControlChange { channel: _, control: 51, value: 127 } = msg {
            println!("Stopping");
            midi_adapter.stop_listening();
        }
    }

    println!("Exiting.");
}
