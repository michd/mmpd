pub mod adapters;
pub use adapters::get_adapter;
use regex::Regex;
use std::ops::Range;

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

/// Parses a string describing a note into MIDI note number(s)
///
/// key_str should be in the format "<note name>[accidental][octave]" where angle brackets denote
/// required content, and square brackets optional content.
///
/// Note name is a letter in the range A-G (case-independent)
/// accidental is either 'b' or '#' and is case sensitive
/// octave is a single digit decimal number with or without `-` prefix
///
/// If no octave is specified, returns a `Vec` with all MIDI note numbers matching that note
/// If an octave is specified and the resulting note number falls within the MIDI note number
/// boundaries, a `Vec` with just that number is returned.
///
/// An empty `Vec` is returned in the following circumstances:
/// - The format didn't match that what was described
/// - The calculated note is out of range of valid MIDI notes
///
/// ## Examples
/// ```
/// use mmpd_lib::midi::parse_keys_from_str;
///
/// let parsed_note = parse_keys_from_str("C3");
/// assert_eq!(parsed_note, vec![48]);
///
/// let parsed_note = parse_keys_from_str("F#4");
/// assert_eq!(parsed_note, vec![66]);
///
/// let parsed_note = parse_keys_from_str("Gb4");
/// assert_eq!(parsed_note, vec![66]);
///
/// let parsed_notes = parse_keys_from_str("C");
/// assert_eq!(
///     parsed_notes,
///     vec![0, 12, 24, 36, 48, 60, 72, 84, 96, 108, 120]
/// )
/// ```
pub fn parse_keys_from_str(key_str: &str) -> Vec<u8> {
    const NOTES_PER_OCTAVE: i16 = 12;
    const VALID_NOTES: Range<i16> = 0..128;

    // Breaking apart the regular expression:
    // ^              - Start of input
    // ([A-Ga-g])     - Note name capture group: single char that's in range A-G or a-g, required
    // ([b#]{1,100})? - Accidental capture group: single char that's either 'b' or '#', optional
    // (-?[0-9])?     - Octave capture group: single digit 0-9, may be prefixed with '-', optional
    // $              - End of input
    let key_regex = Regex::new(
        r"^(?P<note_name>[A-Ga-g])(?P<accidentals>[b#]{1,100})?(?P<octave>-?[0-9])?$"
    ).unwrap();

    let captures = key_regex.captures(key_str);
    if captures.is_none() { return vec![]; }
    let captures = captures.unwrap();

    let note_name = captures.name("note_name");
    if note_name.is_none() { return vec![]; }
    let note_name = note_name.unwrap().as_str().to_uppercase();

    let accidentals = captures.name("accidentals").map(|a| a.as_str());

    let octave = captures.name("octave").map(|oct_match| {
        // Unwrap should never fails, since regex ensures only valid numbers
        oct_match.as_str().parse::<i16>().unwrap()
    });

    // base_note_num is the number of the note if the octave were 0.
    // C-1 is MIDI note 0, so C# starts at 12.
    let base_note_num: i16 = match note_name.as_str() {
        "C" => 12,
        "D" => 14,
        "E" => 16,
        "F" => 17,
        "G" => 19,
        "A" => 21,
        "B" => 23,
        _ => panic!("Invalid note name, which shouldn't be possible thanks to the regex")
    };

    // Add offset if an accidental was given
    let base_note_num = base_note_num + match accidentals {
        Some(accidentals) => {
            let flat_cnt = accidentals.chars().filter(|c| *c == 'b').count() as isize;
            let sharp_cnt = accidentals.chars().filter(|c| *c == '#').count() as isize;

            (sharp_cnt - flat_cnt) as i16
        }

        None => 0
    };

    match octave {
        Some(octave) => {
            // If octave is specified, return only the actual note number, provided it's in range.
            let actual_note = base_note_num + (NOTES_PER_OCTAVE * octave);
            VALID_NOTES
                .filter(|n| *n == actual_note)
                .map(|n| n as u8)
                .collect()
        }

        None =>  {
            // If no octave is specified, return a vec of all the notes that match the base note
            // and any accidental
            let note_num_remainder = base_note_num % NOTES_PER_OCTAVE;

            VALID_NOTES
                .filter(|n| n % NOTES_PER_OCTAVE == note_num_remainder)
                .map(|n| n as u8)
                .collect()
        }
    }
}

#[cfg(test)]
mod parse_message_tests {
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

#[cfg(test)]
mod parse_key_from_str_tests {
    use crate::midi::parse_keys_from_str;

    #[test]
    fn parses_without_accidentals() {
        assert_eq!(parse_keys_from_str("C-1"), vec![0]);
        assert_eq!(parse_keys_from_str("B-1"), vec![11]);
        assert_eq!(parse_keys_from_str("D0"), vec![14]);
        assert_eq!(parse_keys_from_str("E0"), vec![16]);
        assert_eq!(parse_keys_from_str("F1"), vec![29]);
        assert_eq!(parse_keys_from_str("G1"), vec![31]);
        assert_eq!(parse_keys_from_str("A2"), vec![45]);
        assert_eq!(parse_keys_from_str("B2"), vec![47]);
        assert_eq!(parse_keys_from_str("C3"), vec![48]);
        assert_eq!(parse_keys_from_str("B3"), vec![59]);
        assert_eq!(parse_keys_from_str("F9"), vec![125]);
        assert_eq!(parse_keys_from_str("G9"), vec![127]);
    }

    #[test]
    fn parses_with_accidentals() {
        assert_eq!(parse_keys_from_str("C#-1"), vec![1]);
        assert_eq!(parse_keys_from_str("Db-1"), vec![1]);
        assert_eq!(parse_keys_from_str("E#-1"), vec![5]);
        assert_eq!(parse_keys_from_str("Fb-1"), vec![4]);
        assert_eq!(parse_keys_from_str("G#0"), vec![20]);
        assert_eq!(parse_keys_from_str("Ab0"), vec![20]);
        assert_eq!(parse_keys_from_str("F#1"), vec![30]);
        assert_eq!(parse_keys_from_str("Gb1"), vec![30]);
        assert_eq!(parse_keys_from_str("F#9"), vec![126]);
    }

    #[test]
    fn returns_nothing_when_resulting_note_is_not_in_midi_range() {
        assert_eq!(parse_keys_from_str("A-2"), vec![]);
        assert_eq!(parse_keys_from_str("G#-2"), vec![]);
        assert_eq!(parse_keys_from_str("A9"), vec![]);
        assert_eq!(parse_keys_from_str("A#9"), vec![]);
        assert_eq!(parse_keys_from_str("Bb9"), vec![]);
    }

    #[test]
    fn accepts_lowercase_note_names() {
        assert_eq!(parse_keys_from_str("b3"), vec![59]);
        assert_eq!(parse_keys_from_str("f#1"), vec![30]);
    }

    #[test]
    fn returns_all_matching_notes_if_no_octave_given() {
        assert_eq!(
            parse_keys_from_str("C"),
            vec![0, 12, 24, 36, 48, 60, 72, 84, 96, 108, 120]
        );

        assert_eq!(
            parse_keys_from_str("D"),
            vec![2, 14, 26, 38, 50, 62, 74, 86, 98, 110, 122]
        );

        assert_eq!(
            parse_keys_from_str("Eb"),
            vec![3, 15, 27, 39, 51, 63, 75, 87, 99, 111, 123]
        );

        assert_eq!(
            parse_keys_from_str("A"),
            vec![9, 21, 33, 45, 57, 69, 81, 93, 105, 117]
        );
    }

    #[test]
    fn processes_compound_accidentals() {
        assert_eq!(parse_keys_from_str("F##3"), vec![55]); // Equivalent to G3
        assert_eq!(parse_keys_from_str("Gb#b#3"), vec![55]); // Accidentals cancel out
    }

    #[test]
    fn returns_nothing_when_format_is_invalid_or_unsupported() {
        // Gobbledygook
        assert_eq!(parse_keys_from_str("NYERGH"), vec![]);
    }
}
