pub mod adapters;
pub use adapters::get_adapter;

/// MidiMessage is a parsed MIDI message, structured to be easy to work with.
/// It is parsed from 3 raw bytes of data.
#[derive(Debug, Eq, PartialEq)]
pub enum MidiMessage {
    /// Key released.
    /// channel: 0-15
    /// key: Which key was released, 0-127
    /// velocity: Speed by which the key was released, 0-127
    NoteOff { channel: u8, key: u8, velocity: u8 },

    /// Key pressed.
    /// channel: 0-15
    /// key: Which key was pressed, 0-127
    /// velocity: Speed by which the key get pressed down, 0-127
    NoteOn { channel: u8, key: u8, velocity: u8 },

    /// Pressure on a key after it was initially pressed down (can continually change)
    /// channel: 0-15
    /// key: Which key this is about, 0-127
    /// value: current pressure level on the key, 0-127
    PolyAftertouch { channel: u8, key: u8, value: u8 },

    /// A control had its value changed (a fader or rotary knob moved, button pressed, ...)
    /// channel: 0-15
    /// control: identifier for which control it is, 0-127
    /// value: new value, 0-127
    ControlChange { channel: u8, control: u8, value: u8 },

    /// The selected program/patch was changed
    /// channel: 0-15
    /// program: program identifier, 0-127
    ProgramChange { channel: u8, program: u8 },

    /// Channel-wide pressure on any of the keys (most key beds don't have key-specific aftertouch)
    /// channel: 0-15
    /// value: current pressure level on any of the keys, 0-127
    ChannelAftertouch { channel: u8, value: u8 },

    /// Pitch bender position changed
    /// channel: 0-15
    /// value: current position, 0-16,384 (14 bit)
    PitchBendChange { channel: u8, value: u16 },

    // TODO: maybe implement these later
    //TimingClock,
    //Start,
    //Continue,
    //Stop,

    /// Catch-all for any non-implemented messages
    Other
}

/// Parses raw 3-byte MIDI messages into structured MIDI messages
///
/// If there is an invalid amount of data available, or the most significant 4 bits of the first
/// byte make no sense, returns None.
///
/// See the MIDI spec's summary of MIDI messages:
/// https://www.midi.org/specifications-old/item/table-1-summary-of-midi-message
fn parse_message(bytes: &[u8]) -> Option<MidiMessage> {
    let first_byte = *bytes.get(0)?;

    // Status is 4 most significant bytes of the first byte, here shifted right by 4 bits so we can
    // Compare against 4 bit numbers.
    let status = (first_byte & 0xF0) >> 4;

    // Channel is 4 least significant bits of first byte
    let channel: u8 = first_byte & 0x0F;

    return match status {
        0b1000 => Some(MidiMessage::NoteOff {
            channel,
            key: *bytes.get(1)? & 0x7Fu8,
            velocity: *bytes.get(2)? & 0x7Fu8,
        }),

        0b1001 => Some(MidiMessage::NoteOn {
            channel,
            key: *bytes.get(1)? & 0x7Fu8,
            velocity: *bytes.get(2)? & 0x7Fu8,
        }),

        0b1010 => Some(MidiMessage::PolyAftertouch {
            channel,
            key: *bytes.get(1)? & 0x7Fu8,
            value: *bytes.get(2)? & 0x7Fu8,
        }),

        0b1011 => Some(MidiMessage::ControlChange {
            channel,
            control: *bytes.get(1)? & 0x7Fu8,
            value: *bytes.get(2)? & 0x7Fu8,
        }),

        0b1100 => Some(MidiMessage::ProgramChange {
            channel,
            program: *bytes.get(1)? & 0x7Fu8,
        }),

        0b1101 => Some(MidiMessage::ChannelAftertouch {
            channel,
            value: *bytes.get(1)? & 0x7Fu8,
        }),

        0b1110 => Some(MidiMessage::PitchBendChange {
            channel,
            // TODO: check if this is correct, getting weird data from keystep
            value: ((*bytes.get(1)? as u16 & 0x7Fu16) as u16)
                | (((*bytes.get(2)? as u16 & 0x7Fu16) << 7) as u16),
        }),

        0b1111 => Some(MidiMessage::Other),

        _ => None
    }
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


