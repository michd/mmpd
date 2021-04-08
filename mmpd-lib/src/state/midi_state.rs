use crate::midi::MidiMessage;
use std::collections::{HashSet, HashMap};
use crate::macros::preconditions::midi::MidiPrecondition;
use crate::match_checker::MatchChecker;

/// State tracking container for MIDI messages.
///
/// MidiState keeps track off:
/// - Which notes are currently pressed / "on" for each channel
/// - All known values of controls per channel
/// - All known selected programs for each channel
/// - All known pitch bend values for each channel
///
/// It only starts storing values for each of these the moment a MIDI message with relevant
/// data comes in. If for example, a key was pressed before the program was running, MidiState
/// would have no record of that key currently being pressed.
pub(crate) struct MidiState {
    /// Set of notes that are currently pressed
    notes_on: HashSet<Note>,

    // Control values for any controls we've received messages about
    controls: HashMap<Control, u8>,

    // Chosen programs for each channel that we've received program change messages for.
    // The key number here is the channel.
    programs: HashMap<u8, u8>,

    // Pitch bend positions for each channel that we've received pitch bend messages for
    // The key number here is the channel.
    pitch_bend_values: HashMap<u8, u16>
}

/// Represents a unique note, scoped by channel and key
#[derive(Hash, Eq, PartialEq, Debug)]
struct Note {
    channel: u8,
    key: u8,
}

/// Represents a unique control, scoped by channel and control number
#[derive(Hash, Eq, PartialEq, Debug)]
struct Control {
    channel: u8,
    control: u8,
}

impl MidiState {
    pub fn new() -> MidiState {
        MidiState {
            notes_on: HashSet::new(),
            controls: HashMap::new(),
            programs: HashMap::new(),
            pitch_bend_values: HashMap::new()
        }
    }

    /// Processes an incoming MIDI message, mutating itself as a result.
    /// Message types that are relevant to MidiState are:
    /// - NoteOn
    /// - NoteOff
    /// - ControlChange
    /// - ProgramChange
    /// - PitchBendChange
    ///
    /// Any other messages are ignored.
    pub fn process_message(&mut self, msg: &MidiMessage) {
        match msg {
            MidiMessage::NoteOn { channel, key, .. } => {
                self.notes_on.insert(
                    Note { channel: *channel, key: *key }
                );
            }

            MidiMessage::NoteOff { channel, key, .. } => {
                self.notes_on.remove(
                    &Note { channel: *channel, key: *key }
                );
            },

            MidiMessage::ControlChange { channel, control, value } => {
                self.controls.insert(
                    Control { channel: *channel, control: *control },
                    *value
                );
            }

            MidiMessage::ProgramChange { channel, program } => {
                self.programs.insert(*channel, *program);
            }

            MidiMessage::PitchBendChange { channel, value } => {
                self.pitch_bend_values.insert(*channel, *value);
            }

            _ => {}
        }
    }

    /// Checks if a MidiPrecondition matches against this MIDI state
    pub fn matches(&self, precondition: &MidiPrecondition) -> bool {
        match precondition {
            MidiPrecondition::NoteOn { channel_match, key_match } => {
                self.notes_on.iter().any(|note| {
                    channel_match.matches(&(note.channel as u32)) &&
                        key_match.matches(&(note.key as u32))
                })
            }

            MidiPrecondition::Control {
                channel_match,
                control_match,
                value_match
            } => {
                self.controls.iter().any(|(control, value)| {
                    channel_match.matches(&(control.channel as u32)) &&
                        control_match.matches(&(control.control as u32)) &&
                        value_match.matches(&(*value as u32))
                })
            }

            MidiPrecondition::Program { channel_match, program_match } => {
                self.programs.iter().any(|(channel, program)| {
                    channel_match.matches(&(*channel as u32)) &&
                        program_match.matches(&(*program as u32))
                })
            }

            MidiPrecondition::PitchBend { channel_match, value_match } => {
                self.pitch_bend_values.iter().any(|(channel, value)| {
                    channel_match.matches(&(*channel as u32)) &&
                        value_match.matches(&(*value as u32))
                })
            }
        }
    }
}

#[cfg(test)]
mod state_keeping_tests {
    use crate::midi::MidiMessage;
    use crate::state::midi_state::{MidiState, Note, Control};

    #[test]
    fn keeps_track_of_notes_held() {
        let note1 = Note { channel: 3, key: 20 };
        let note2 = Note { channel: 7, key: 30 };

        let mut state = MidiState::new();
        assert!(state.notes_on.is_empty());

        // Pressing notes

        let note_on_msg1 = MidiMessage::NoteOn {
            channel: note1.channel,
            key: note1.key,
            velocity: 67
        };

        let note_on_msg2 = MidiMessage::NoteOn {
            channel: note2.channel,
            key: note2.key,
            velocity: 42
        };

        state.process_message(&note_on_msg1);
        assert!(state.notes_on.contains(&note1));

        state.process_message(&note_on_msg2);
        assert!(state.notes_on.contains(&note1));
        assert!(state.notes_on.contains(&note2));

        // Releasing notes

        let note_off_msg1 = MidiMessage::NoteOff {
            channel: note1.channel,
            key: note1.key,
            velocity: 64
        };

        let note_off_msg2 = MidiMessage::NoteOff {
            channel: note2.channel,
            key: note2.key,
            velocity: 120
        };

        state.process_message(&note_off_msg1);
        assert!(!state.notes_on.contains(&note1));
        assert!(state.notes_on.contains(&note2));

        state.process_message(&note_off_msg2);
        assert!(state.notes_on.is_empty());
    }

    #[test]
    fn keeps_only_one_record_of_a_pressed_note() {
        let note1 = Note { channel: 3, key: 20 };

        let mut state = MidiState::new();

        let note_on_msg1 = MidiMessage::NoteOn {
            channel: note1.channel,
            key: note1.key,
            velocity: 67
        };

        state.process_message(&note_on_msg1);
        state.process_message(&note_on_msg1);

        assert!(state.notes_on.contains(&note1));

        let note_off_msg1 = MidiMessage::NoteOff {
            channel: note1.channel,
            key: note1.key,
            velocity: 67
        };

        state.process_message(&note_off_msg1);

        assert!(!state.notes_on.contains(&note1));
        assert!(state.notes_on.is_empty());
    }

    #[test]
    fn deals_with_note_off_for_note_that_was_not_held() {
        // We'll press this
        let note1 = Note { channel: 3, key: 20 };

        // We'll never press this, but release it
        let note2 = Note { channel: 7, key: 30 };

        let mut state = MidiState::new();

        let note_on_msg1 = MidiMessage::NoteOn {
            channel: note1.channel,
            key: note1.key,
            velocity: 67
        };

        let note_off_msg2 = MidiMessage::NoteOff {
            channel: note2.channel,
            key: note2.key,
            velocity: 100
        };

        let note_off_msg1 = MidiMessage::NoteOff {
            channel: note1.channel,
            key: note1.key,
            velocity: 99
        };

        state.process_message(&note_on_msg1);

        assert!(state.notes_on.contains(&note1));
        assert!(!state.notes_on.contains(&note2));

        state.process_message(&note_off_msg2);

        assert!(state.notes_on.contains(&note1));
        assert!(!state.notes_on.contains(&note2));

        state.process_message(&note_off_msg1);

        assert!(!state.notes_on.contains(&note1));
        assert!(!state.notes_on.contains(&note2));
        assert!(state.notes_on.is_empty());
    }

    #[test]
    fn keeps_track_of_control_changes() {
        let control1 = Control { channel: 1, control: 3 };

        let mut state = MidiState::new();

        assert!(state.controls.get(&control1).is_none());

        let control_change1 = MidiMessage::ControlChange {
            channel: control1.channel,
            control: control1.control,
            value: 40
        };

        state.process_message(&control_change1);

        assert_eq!(state.controls.get(&control1), Some(&40));

        let control_change1 = MidiMessage::ControlChange {
            channel: control1.channel,
            control: control1.control,
            value: 50
        };

        state.process_message(&control_change1);

        assert_eq!(state.controls.get(&control1), Some(&50));
    }

    #[test]
    fn keeps_track_of_program_changes() {
        let channel = 4u8;

        let mut state = MidiState::new();

        assert!(state.programs.get(&channel).is_none());

        let program_change1 = MidiMessage::ProgramChange {
            channel,
            program: 2
        };

        state.process_message(&program_change1);

        assert_eq!(state.programs.get(&channel), Some(&2));

        let program_change1 = MidiMessage::ProgramChange {
            channel,
            program: 60
        };

        state.process_message(&program_change1);

        assert_eq!(state.programs.get(&channel), Some(&60));
    }

    #[test]
    fn keeps_track_of_pitch_bend_changes() {
        let channel = 4u8;

        let mut state = MidiState::new();

        assert!(state.pitch_bend_values.get(&channel).is_none());

        let pitchbend_change1 = MidiMessage::PitchBendChange {
            channel,
            value: 569
        };

        state.process_message(&pitchbend_change1);

        assert_eq!(state.pitch_bend_values.get(&channel), Some(&569));

        let pitchbend_change1 = MidiMessage::PitchBendChange {
            channel,
            value: 421
        };

        state.process_message(&pitchbend_change1);

        assert_eq!(state.pitch_bend_values.get(&channel), Some(&421));
    }
}

#[cfg(test)]
mod precondition_matching_tests {
    use crate::macros::preconditions::midi::MidiPrecondition;
    use crate::match_checker::NumberMatcher;
    use crate::state::midi_state::{MidiState, Note, Control};

    #[test]
    fn matches_a_held_note() {
        let condition = MidiPrecondition::NoteOn {
            channel_match: Some(NumberMatcher::Val(1)),
            key_match: Some(NumberMatcher::Val(20))
        };

        let mut state = MidiState::new();

        assert!(!state.matches(&condition));

        state.notes_on.insert(Note { channel: 1, key: 20 });

        assert!(state.matches(&condition));
    }

    #[test]
    fn does_not_match_held_note_if_channel_does_not_match() {
        let condition = MidiPrecondition::NoteOn {
            channel_match: Some(NumberMatcher::Val(1)),
            key_match: Some(NumberMatcher::Val(20))
        };

        let mut state = MidiState::new();

        state.notes_on.insert(Note { channel: 3, key: 20 });

        assert!(!state.matches(&condition));
    }

    #[test]
    fn does_not_match_held_note_if_key_does_not_match() {
        let condition = MidiPrecondition::NoteOn {
            channel_match: Some(NumberMatcher::Val(1)),
            key_match: Some(NumberMatcher::Val(20))
        };

        let mut state = MidiState::new();
        state.notes_on.insert(Note { channel: 1, key: 40 });

        assert!(!state.matches(&condition));
    }

    #[test]
    fn matches_a_control_value() {
        let condition = MidiPrecondition::Control {
            channel_match: Some(NumberMatcher::Val(1)),
            control_match: Some(NumberMatcher::Val(20)),
            value_match: Some(NumberMatcher::Val(40))
        };

        let mut state = MidiState::new();

        assert!(!state.matches(&condition));

        state.controls.insert(Control { channel: 1, control: 20 }, 40);

        assert!(state.matches(&condition));
    }

    #[test]
    fn does_not_match_a_control_if_channel_does_not_match() {
        let condition = MidiPrecondition::Control {
            channel_match: Some(NumberMatcher::Val(1)),
            control_match: Some(NumberMatcher::Val(20)),
            value_match: Some(NumberMatcher::Val(40))
        };

        let mut state = MidiState::new();

        state.controls.insert(Control { channel: 4, control: 20 }, 40);

        assert!(!state.matches(&condition));
    }

    #[test]
    fn does_not_match_a_control_if_control_number_does_not_match() {
        let condition = MidiPrecondition::Control {
            channel_match: Some(NumberMatcher::Val(1)),
            control_match: Some(NumberMatcher::Val(20)),
            value_match: Some(NumberMatcher::Val(40))
        };

        let mut state = MidiState::new();

        state.controls.insert(Control { channel: 1, control: 30 }, 40);

        assert!(!state.matches(&condition));
    }

    #[test]
    fn does_not_match_a_control_if_value_does_not_match() {
        let condition = MidiPrecondition::Control {
            channel_match: Some(NumberMatcher::Val(1)),
            control_match: Some(NumberMatcher::Val(20)),
            value_match: Some(NumberMatcher::Val(40))
        };

        let mut state = MidiState::new();

        state.controls.insert(Control { channel: 1, control: 20 }, 60);

        assert!(!state.matches(&condition));
    }

    #[test]
    fn matches_a_program() {
        let condition = MidiPrecondition::Program {
            channel_match: Some(NumberMatcher::Val(1)),
            program_match: Some(NumberMatcher::Val(20))
        };

        let mut state = MidiState::new();

        assert!(!state.matches(&condition));

        state.programs.insert(1, 20);

        assert!(state.matches(&condition));
    }

    #[test]
    fn does_not_match_program_if_channel_does_not_match() {
        let condition = MidiPrecondition::Program {
            channel_match: Some(NumberMatcher::Val(1)),
            program_match: Some(NumberMatcher::Val(20))
        };

        let mut state = MidiState::new();

        state.programs.insert(4, 20);

        assert!(!state.matches(&condition));
    }

    #[test]
    fn does_not_match_program_if_program_does_not_match() {
        let condition = MidiPrecondition::Program {
            channel_match: Some(NumberMatcher::Val(1)),
            program_match: Some(NumberMatcher::Val(20))
        };

        let mut state = MidiState::new();

        state.programs.insert(1, 40);

        assert!(!state.matches(&condition));
    }

    #[test]
    fn matches_pitch_bend_value() {
        let condition = MidiPrecondition::PitchBend {
            channel_match: Some(NumberMatcher::Val(1)),
            value_match: Some(NumberMatcher::Val(20))
        };

        let mut state = MidiState::new();

        assert!(!state.matches(&condition));

        state.pitch_bend_values.insert(1, 20);

        assert!(state.matches(&condition));
    }

    #[test]
    fn does_not_match_pitch_bend_value_if_channel_does_not_match() {
        let condition = MidiPrecondition::PitchBend {
            channel_match: Some(NumberMatcher::Val(1)),
            value_match: Some(NumberMatcher::Val(20))
        };

        let mut state = MidiState::new();

        state.pitch_bend_values.insert(4, 20);

        assert!(!state.matches(&condition));
    }

    #[test]
    fn does_not_match_pitch_bend_value_if_value_does_not_match() {
        let condition = MidiPrecondition::PitchBend {
            channel_match: Some(NumberMatcher::Val(1)),
            value_match: Some(NumberMatcher::Val(20))
        };

        let mut state = MidiState::new();

        state.pitch_bend_values.insert(1, 40);

        assert!(!state.matches(&condition));
    }
}