extern crate portmidi as pm;
extern crate libxdo;

use std::process::Command;
use std::str;
use std::vec::Vec;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use libxdo::XDo;

#[derive(FromPrimitive)]
enum ChannelMessageType {
    NoteOff = 0b1000isize,
    NoteOn = 0b1001isize,
    PolyAftertouch = 0b1010isize,
    ControlChange = 0b1011isize,
    ProgramChange = 0b1100isize,
    ChannelAfterTouch = 0b1101isize,
    PitchBendChange = 0b1110isize,
    System = 0b1111isize,
}

#[derive(Debug)]
enum FormattedMidiMessage {
    NoteOff { channel: u8, key: u8, velocity: u8 },
    NoteOn { channel: u8, key: u8, velocity: u8 },
    PolyAftertouch { channel: u8, key: u8, value: u8 },
    ControlChange { channel: u8, control: u8, value: u8 },
    ProgramChange { channel: u8, program: u8 },
    ChannelAftertouch { channel: u8, value: u8 },
    PitchBendChange { channel: u8, value: u16 },
    //TimingClock,
    //Start,
    //Continue,
    //Stop,
    Other
}

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

//fn print_devices(pm: &pm::PortMidi) {
//    for dev in pm.devices().unwrap() {
//        println!("{}", dev);
//    }
//}

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

fn parse_message(msg: &pm::MidiMessage) -> FormattedMidiMessage {
    let chan: u8 = msg.status & 0x0F;

    return match FromPrimitive::from_u8((msg.status & 0xF0) >> 4) {
        Some(ChannelMessageType::NoteOff) => {
            FormattedMidiMessage::NoteOff {
                channel: chan,
                key: msg.data1,
                velocity: msg.data2
            }
        }

        Some(ChannelMessageType::NoteOn) => FormattedMidiMessage::NoteOn {
            channel: chan,
            key: msg.data1,
            velocity: msg.data2
        },


        Some(ChannelMessageType::PolyAftertouch) => FormattedMidiMessage::PolyAftertouch {
            channel: chan,
            key: msg.data1,
            value: msg.data2
        },

        Some(ChannelMessageType::ControlChange) => FormattedMidiMessage::ControlChange {
            channel: chan,
            control: msg.data1,
            value: msg.data2
        },

        Some(ChannelMessageType::ProgramChange) => FormattedMidiMessage::ProgramChange {
            channel: chan,
            program: msg.data1
        },

        Some(ChannelMessageType::ChannelAfterTouch) => FormattedMidiMessage::ChannelAftertouch {
            channel: chan,
            value: msg.data1
        },

        Some(ChannelMessageType::PitchBendChange) => FormattedMidiMessage::PitchBendChange {
            channel: chan,
            value: ((msg.data1 & 0b01111111u8) as u16) | (((msg.data2 as u16 & 0b01111111u16) << 7) as u16)
        },

        Some(ChannelMessageType::System) => FormattedMidiMessage::Other,
        None => FormattedMidiMessage::Other,
    }
}

fn monitor(port: &pm::InputPort) {
    let xdo = XDo::new(None).unwrap();

    while let Ok(_) = port.poll() {
        if let Ok(Some(events)) = port.read_n(1024) {
            for event in events.iter() {
                let fmsg = parse_message(&event.message);
                println!("{:?}", fmsg);

                if let FormattedMidiMessage::NoteOff { channel: _, key, velocity: _ } = fmsg {
                    match key {
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
            }
        }
    }
}

fn main() {
    println!("Hello, world!");
    let context = pm::PortMidi::new().unwrap();
    let device_info = context.device(7).unwrap();
    let in_port = context.input_port(device_info, 1024).unwrap();
    monitor(&in_port);
}
