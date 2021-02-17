extern crate libxdo;
extern crate midir;

use std::process::Command;
use std::str;
use std::vec::Vec;
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
    let (tx, rx) = midi::get_midi_bus();

    let mut mr = midi::adapters::midir::Midir::new();
    mr.start(tx);
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
            mr.stop();
        }
    }

    // TODO this is never reached event when we exit the loop in midir.
    println!("Exiting.");
}
