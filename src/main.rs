extern crate portmidi as pm;
extern crate libxdo;

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

//fn print_devices(pm: &pm::PortMidi) {
//    for dev in pm.devices().unwrap() {
//        println!("{}", dev);
//    }
//}

fn parse_message(msg: &pm::MidiMessage) -> FormattedMidiMessage {
    let chan: u8 = msg.status & 0x0F;

    match FromPrimitive::from_u8((msg.status & 0xF0) >> 4) {
        Some(ChannelMessageType::NoteOff) => {
            return FormattedMidiMessage::NoteOff {
                channel: chan,
                key: msg.data1,
                velocity: msg.data2
            };
        }

        Some(ChannelMessageType::NoteOn) => {
            return FormattedMidiMessage::NoteOn {
                channel: chan,
                key: msg.data1,
                velocity: msg.data2
            };
        }

        Some(ChannelMessageType::PolyAftertouch) => {
            return FormattedMidiMessage::PolyAftertouch {
                channel: chan,
                key: msg.data1,
                value: msg.data2
            };
        }

        Some(ChannelMessageType::ControlChange) => {
            return FormattedMidiMessage::ControlChange {
                channel: chan,
                control: msg.data1,
                value: msg.data2
            };
        }

        Some(ChannelMessageType::ProgramChange) => {
            return FormattedMidiMessage::ProgramChange {
                channel: chan,
                program: msg.data1
            };
        }

        Some(ChannelMessageType::ChannelAfterTouch) => {
            return FormattedMidiMessage::ChannelAftertouch {
                channel: chan,
                value: msg.data1
            };
        }

        Some(ChannelMessageType::PitchBendChange) => {
            return FormattedMidiMessage::PitchBendChange {
                channel: chan,
                value: ((msg.data1 & 0b01111111u8) as u16) | (((msg.data2 as u16 & 0b01111111u16) << 7) as u16)
            };
        }

        Some(ChannelMessageType::System) => {
            return FormattedMidiMessage::Other;
        }

        None => {
            return FormattedMidiMessage::Other;
        }

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
                        60 => { xdo.enter_text("Hello world!", 250).unwrap(); }
                        61 => { xdo.send_keysequence("ctrl+c", 0).unwrap(); }
                        _ => {}
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
