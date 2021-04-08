use crate::match_checker::NumMatch;

/// Precondition to be checked against MidiState
#[derive(Debug, PartialEq)]
pub enum MidiPrecondition {
    /// A note with a channel matching the channel matcher and the key matcher is currently held
    NoteOn { channel_match: NumMatch, key_match: NumMatch },

    /// A control matching the channel and control matchers matches the value matcher
    Control { channel_match: NumMatch, control_match: NumMatch, value_match: NumMatch },

    /// A channel matching the channel matcher has its program set to a program matching the
    /// program matcher
    Program { channel_match: NumMatch, program_match: NumMatch },

    /// A channel matching the channel matcher has a pitch bend value matching the value matcher
    PitchBend { channel_match: NumMatch, value_match: NumMatch },
}