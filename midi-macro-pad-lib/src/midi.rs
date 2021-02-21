pub mod adapters;
pub use adapters::get_adapter;

use num_derive::FromPrimitive;
use std::sync::mpsc::{self, SyncSender, Receiver};
use num_traits::FromPrimitive;

pub fn get_midi_bus() -> (SyncSender<MidiMessage>, Receiver<MidiMessage>) {
    mpsc::sync_channel(1024)
}

#[derive(Debug, Eq, PartialEq)]
pub enum MidiMessage {
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

fn parse_message(bytes: &[u8]) -> Option<MidiMessage> {
    let status = *bytes.get(0)?;
    let channel: u8 = status & 0x0F;

    return match FromPrimitive::from_u8((status & 0xF0) >> 4) {
        Some(ChannelMessageType::NoteOff) => Some(MidiMessage::NoteOff {
            channel,
            key: *bytes.get(1)? & 0x7Fu8,
            velocity: *bytes.get(2)? & 0x7Fu8,
        }),

        Some(ChannelMessageType::NoteOn) => Some(MidiMessage::NoteOn {
            channel,
            key: *bytes.get(1)? & 0x7Fu8,
            velocity: *bytes.get(2)? & 0x7Fu8,
        }),

        Some(ChannelMessageType::PolyAftertouch) => Some(MidiMessage::PolyAftertouch {
            channel,
            key: *bytes.get(1)? & 0x7Fu8,
            value: *bytes.get(2)? & 0x7Fu8,
        }),

        Some(ChannelMessageType::ControlChange) => Some(MidiMessage::ControlChange {
            channel,
            control: *bytes.get(1)? & 0x7Fu8,
            value: *bytes.get(2)? & 0x7Fu8,
        }),

        Some(ChannelMessageType::ProgramChange) => Some(MidiMessage::ProgramChange {
            channel,
            program: *bytes.get(1)? & 0x7Fu8,
        }),

        Some(ChannelMessageType::ChannelAfterTouch) => Some(MidiMessage::ChannelAftertouch {
            channel,
            value: *bytes.get(1)? & 0x7Fu8,
        }),

        Some(ChannelMessageType::PitchBendChange) => Some(MidiMessage::PitchBendChange {
            channel,
            value: ((*bytes.get(1)? & 0b01111111u8) as u16)
                | (((*bytes.get(2)? as u16 & 0b01111111u16) << 7) as u16),
        }),

        Some(ChannelMessageType::System) => Some(MidiMessage::Other),
        None => None,
    };
}

#[cfg(test)]
mod tests {
    use crate::midi::MidiMessage;
    use crate::midi::parse_message;

    #[test]
    fn parses_note_messages() {
        let note_on_ch0 = [0b1001_0000u8, 63u8, 120u8];
        let note_on_ch1 = [0b1001_0001u8, 127u8, 1u8];
        let note_off_ch2 = [0b1000_0010u8, 42u8, 53u8];
        let note_off_ch3 = [0b1000_0011u8, 78u8, 12u8];

        assert_eq!(
            MidiMessage::NoteOn { channel: 0, key: 63, velocity: 120 },
            parse_message(&note_on_ch0).unwrap()
        );

        assert_eq!(
            MidiMessage::NoteOn { channel: 1, key: 127, velocity: 1 },
            parse_message(&note_on_ch1).unwrap()
        );

        assert_eq!(
            MidiMessage::NoteOff { channel: 2, key: 42, velocity: 53 },
            parse_message(&note_off_ch2).unwrap()
        );

        assert_eq!(
            MidiMessage::NoteOff { channel: 3, key: 78, velocity: 12 },
            parse_message(&note_off_ch3).unwrap()
        );
    }

    #[test]
    fn parses_control_change() {
        let cc_ch0 = [0b1011_0000u8, 20u8, 120u8];
        let cc_ch15 = [0b1011_1111u8, 48u8, 24u8];

        assert_eq!(
            MidiMessage::ControlChange { channel: 0, control: 20, value: 120 },
            parse_message(&cc_ch0).unwrap()
        );

        assert_eq!(
            MidiMessage::ControlChange { channel: 15, control: 48, value: 24 },
            parse_message(&cc_ch15).unwrap()
        );
    }

    // TODO more tests for other types of messages, as well as invalid messages

    #[test]
    fn disregards_msb_in_values() {
        // Ensures values remain in range of 0-127 even if input data is broken
        let note_on_ch0 = [0b1001_0000u8, 201u8, 202u8];
        let note_off_ch0 = [0b1000_0000u8, 200u8, 255u8];
        let poly_at_ch0 = [0b1010_0000u8, 200u8, 255u8];
        let cc_ch0 = [0b1011_0000u8, 220u8, 255u8];
        let pc_ch0 = [0b1100_0000u8, 200u8];
        let cat_ch0 = [0b1101_0000u8, 200u8];

        assert_eq!(
            MidiMessage::NoteOn { channel: 0, key: 201 & 0x7F, velocity: 202 & 0x7F },
            parse_message(&note_on_ch0).unwrap()
        );

        assert_eq!(
            MidiMessage::NoteOff { channel: 0, key: 200 & 0x7F, velocity: 127 & 0x7F },
            parse_message(&note_off_ch0).unwrap()
        );

        assert_eq!(
            MidiMessage::PolyAftertouch { channel: 0, key: 200 & 0x7F, value: 255 & 0x7F },
            parse_message(&poly_at_ch0).unwrap()
        );

        assert_eq!(
            MidiMessage::ControlChange { channel: 0, control: 220 & 0x7F, value: 255 & 0x7F },
            parse_message(&cc_ch0).unwrap()
        );

        assert_eq!(
            MidiMessage::ProgramChange { channel: 0, program: 200 & 0x7F },
            parse_message(&pc_ch0).unwrap()
        );

        assert_eq!(
            MidiMessage::ChannelAftertouch { channel: 0, value: 200 & 0x7F },
            parse_message(&cat_ch0).unwrap()
        )
    }
}


