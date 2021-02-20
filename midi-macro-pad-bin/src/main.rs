extern crate libxdo;

use std::process::Command;
use std::str;
use std::vec::Vec;
use std::env;
use libxdo::XDo;
use midi_macro_pad_lib::midi::{self, types::MidiMessage};

#[derive(Debug)]
struct FocusedWindow {
    window_class: Vec<String>,
    window_name: String,
}

impl FocusedWindow {
    fn blank() -> FocusedWindow {
        return FocusedWindow {
            window_class: vec![],
            window_name: String::from(""),
        }
    }
}

fn parse_quoted_list(list: &str) -> Vec<String> {
    let split = list.split("\", \"");

    let result: Vec<&str> = split.collect();
    let mut converted_result: Vec<String> = vec![];

    for item in result.iter() {
        converted_result.push(String::from(item.to_owned()))
    }

    return converted_result
}

fn get_focused_window() -> FocusedWindow {
    let raw_window_id = Command::new("xdotool")
        .arg("getwindowfocus")
        .output()
        .expect("couldn't get window id");

    let focused_window_id = str::from_utf8(raw_window_id.stdout.as_slice()).unwrap().lines().next().unwrap();

    let raw_output = Command::new("xprop")
        .arg("-root")
        .arg("-id")
        .arg(focused_window_id)
        .arg("WM_CLASS")
        .arg("WM_NAME")
        .output()
        .expect("couldn't get focused window info");

    let output = str::from_utf8(raw_output.stdout.as_slice()).unwrap();

    let mut fw = FocusedWindow::blank();

    for line in output.lines() {
        if line.starts_with("WM_CLASS(STRING) = \"") {
            let len = line.len();
            fw.window_class = parse_quoted_list(&line[20..len - 1]);
        }

        if line.starts_with("WM_NAME(STRING) = \"") {
            let len = line.len();
            fw.window_name = String::from(&line[19..len - 1]);
        }

        if line.starts_with("WM_NAME(COMPOUND_TEXT) = \"") {
            let len = line.len();
            fw.window_name = String::from(&line[26..len - 1]);
        }
    }

    return fw;
}

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

    let handle = midi_adapter.start_listening(String::from(port_pattern), tx);

    if let None = handle {
        eprintln!("Unable to start listening for MIDI events.");
    }

    let xdo = XDo::new(None).unwrap();
    for msg in rx {
        println!("{:?}", msg);

        if let MidiMessage::NoteOff { channel: _, key, velocity: _ } = msg {
            match key {
                // Some hardcoded test actions for now
                48 => {
                    let fw = get_focused_window();
                    if fw.window_name.ends_with("Inkscape") {
                        println!("in inkscape, executing centre on horizontal axis.");
                        xdo.send_keysequence("ctrl+shift+a", 100).unwrap();
                        for _ in 0..6 {
                            xdo.send_keysequence("Tab", 100).unwrap();
                        }
                        xdo.send_keysequence("Return", 100).unwrap();
                    } else {
                        println!("not in inkscape, doing nothing.");
                    }
                }

                60 => { xdo.enter_text("Hello world!", 250).unwrap(); }

                61 => { xdo.send_keysequence("ctrl+c", 0).unwrap(); }

                62 => {
                    let fw = get_focused_window();
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
