use crate::match_checker::{MatchChecker, NumMatch};
use crate::midi::MidiMessage;

pub enum MidiEventMatcher {
    NoteOn { channel_match: NumMatch, key_match: NumMatch, velocity_match: NumMatch },
    NoteOff { channel_match: NumMatch, key_match: NumMatch, velocity_match: NumMatch },
    PolyAftertouch { channel_match: NumMatch, key_match: NumMatch, value_match: NumMatch },
    ControlChange { channel_match: NumMatch, control_match: NumMatch, value_match: NumMatch },
    ProgramChange { channel_match: NumMatch, program_match: NumMatch },
    ChannelAftertouch { channel_match: NumMatch, value_match: NumMatch },
    PitchBendRange { channel_match: NumMatch, value_match: NumMatch },
}

// TODO: something that takes the generic format of a midi event matcher (with
// strings and arrays etc) into a MidiEventMatcher value

impl MatchChecker<&MidiMessage> for MidiEventMatcher {
    fn matches(&self, val: &MidiMessage) -> bool {
        match self {
            MidiEventMatcher::NoteOn {
                channel_match, key_match, velocity_match
            } => {
                match val {
                    MidiMessage::NoteOn { channel, key, velocity } => {
                        channel_match.matches(u32::from(*channel))
                            && key_match.matches(u32::from(*key))
                            && velocity_match.matches(u32::from(*velocity))
                    }
                    _ => false
                }
            }

            MidiEventMatcher::NoteOff {
                channel_match, key_match, velocity_match
            } => {
                match val {
                    MidiMessage::NoteOff { channel, key, velocity } => {
                        channel_match.matches(u32::from(*channel))
                            && key_match.matches(u32::from(*key))
                            && velocity_match.matches(u32::from(*velocity))
                    }
                    _ => false
                }
            }

            MidiEventMatcher::PolyAftertouch {
                channel_match, key_match, value_match
            } => {
                match val {
                    MidiMessage::PolyAftertouch { channel, key, value} => {
                        channel_match.matches(u32::from(*channel))
                            && key_match.matches(u32::from(*key))
                            && value_match.matches(u32::from(*value))
                    }
                    _ => false
                }
            }

            MidiEventMatcher::ControlChange {
                channel_match, control_match, value_match
            } => {
                match val {
                    MidiMessage::ControlChange { channel, control, value } => {
                        channel_match.matches(u32::from(*channel))
                            && control_match.matches(u32::from(*control))
                            && value_match.matches(u32::from(*value))
                    }
                    _ => false
                }
            }

            MidiEventMatcher::ProgramChange { channel_match, program_match} => {
                match val {
                    MidiMessage::ProgramChange { channel, program} => {
                        channel_match.matches(u32::from(*channel))
                            && program_match.matches(u32::from(*program))
                    }
                    _ => false
                }
            }

            MidiEventMatcher::ChannelAftertouch { channel_match, value_match} => {
                match val {
                    MidiMessage::ChannelAftertouch { channel, value} => {
                        channel_match.matches(u32::from(*channel))
                            && value_match.matches(u32::from(*value))
                    }
                    _ => false
                }
            }

            MidiEventMatcher::PitchBendRange { channel_match, value_match } => {
                match val {
                    MidiMessage::PitchBendChange { channel, value} => {
                        channel_match.matches(u32::from(*channel))
                            && value_match.matches(u32::from(*value))
                    }
                    _ => false
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::macros::event_matching::midi::MidiEventMatcher;
    use crate::match_checker::{MatchChecker, NumberMatcher};
    use crate::midi::MidiMessage;


    #[test]
    fn midi_event_match_note_on() {
        // Only channel 1, key 32-38 inclusive, don't care about velocity
        let matcher = MidiEventMatcher::NoteOn {
            channel_match: Some(NumberMatcher::Val(1)),
            key_match: Some(NumberMatcher::Range { min: Some(32), max: Some(38) }),
            velocity_match: None,
        };

        let message = MidiMessage::NoteOn { channel: 1, key: 34, velocity: 23 };
        assert!(matcher.matches(&message));

        let message = MidiMessage::NoteOn { channel: 2, key: 34, velocity: 23 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::NoteOn { channel: 1, key: 23, velocity: 92 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::NoteOff { channel: 1, key: 35, velocity: 8 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::PolyAftertouch { channel: 1, key: 35, value: 50 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::ControlChange { channel: 1, control: 35, value: 50 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::ProgramChange { channel: 1, program: 35 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::ChannelAftertouch { channel: 1, value: 35 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::PitchBendChange { channel: 1, value: 35 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::Other;
        assert!(!matcher.matches(&message));

        // Any channel, any key, but velocity at least 100
        let matcher = MidiEventMatcher::NoteOn {
            channel_match: None,
            key_match: None,
            velocity_match: Some(NumberMatcher::Range { min: Some(100), max: None }),
        };

        let message = MidiMessage::NoteOn { channel: 5, key: 3, velocity: 101 };
        assert!(matcher.matches(&message));

        let message = MidiMessage::NoteOn { channel: 10, key: 124, velocity: 120 };
        assert!(matcher.matches(&message));

        let message = MidiMessage::NoteOn { channel: 4, key: 80, velocity: 70 };
        assert!(!matcher.matches(&message));
    }

    #[test]
    fn midi_event_match_note_off() {
        // Only channel 1, key 32-38 inclusive, don't care about velocity
        let matcher = MidiEventMatcher::NoteOff {
            channel_match: Some(NumberMatcher::Val(1)),
            key_match: Some(NumberMatcher::Range { min: Some(32), max: Some(38) }),
            velocity_match: None,
        };

        let message = MidiMessage::NoteOff { channel: 1, key: 34, velocity: 23 };
        assert!(matcher.matches(&message));

        let message = MidiMessage::NoteOff { channel: 2, key: 34, velocity: 23 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::NoteOff { channel: 1, key: 23, velocity: 92 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::NoteOn { channel: 1, key: 35, velocity: 8 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::PolyAftertouch { channel: 1, key: 35, value: 50 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::ControlChange { channel: 1, control: 35, value: 50 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::ProgramChange { channel: 1, program: 35 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::ChannelAftertouch { channel: 1, value: 35 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::PitchBendChange { channel: 1, value: 35 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::Other;
        assert!(!matcher.matches(&message));

        // Any channel, any key, but velocity at most 30
        let matcher = MidiEventMatcher::NoteOff {
            channel_match: None,
            key_match: None,
            velocity_match: Some(NumberMatcher::Range { min: None, max: Some(30) }),
        };

        let message = MidiMessage::NoteOff { channel: 5, key: 3, velocity: 20 };
        assert!(matcher.matches(&message));

        let message = MidiMessage::NoteOff { channel: 10, key: 124, velocity: 15 };
        assert!(matcher.matches(&message));

        let message = MidiMessage::NoteOff { channel: 4, key: 80, velocity: 70 };
        assert!(!matcher.matches(&message));
    }

    #[test]
    fn midi_event_match_poly_aftertouch() {
        // Only channel 1, key 32-38 inclusive, don't care about value
        let matcher = MidiEventMatcher::PolyAftertouch {
            channel_match: Some(NumberMatcher::Val(1)),
            key_match: Some(NumberMatcher::Range { min: Some(32), max: Some(38) }),
            value_match: None,
        };

        let message = MidiMessage::PolyAftertouch { channel: 1, key: 34, value: 23 };
        assert!(matcher.matches(&message));

        let message = MidiMessage::PolyAftertouch { channel: 2, key: 34, value: 23 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::PolyAftertouch { channel: 1, key: 23, value: 92 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::NoteOn { channel: 1, key: 35, velocity: 8 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::NoteOff { channel: 1, key: 35, velocity: 50 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::ControlChange { channel: 1, control: 35, value: 50 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::ProgramChange { channel: 1, program: 35 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::ChannelAftertouch { channel: 1, value: 35 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::PitchBendChange { channel: 1, value: 35 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::Other;
        assert!(!matcher.matches(&message));

        // Any channel, any key, but value at most 30
        let matcher = MidiEventMatcher::PolyAftertouch{
            channel_match: None,
            key_match: None,
            value_match: Some(NumberMatcher::Range { min: None, max: Some(30) }),
        };

        let message = MidiMessage::PolyAftertouch { channel: 5, key: 3, value: 20 };
        assert!(matcher.matches(&message));

        let message = MidiMessage::PolyAftertouch { channel: 10, key: 124, value: 15 };
        assert!(matcher.matches(&message));

        let message = MidiMessage::PolyAftertouch { channel: 4, key: 80, value: 70 };
        assert!(!matcher.matches(&message));
    }

    #[test]
    fn midi_event_match_control_change() {
        // Only channel 1, key 32-38 inclusive, don't care about value
        let matcher = MidiEventMatcher::ControlChange {
            channel_match: Some(NumberMatcher::Val(1)),
            control_match: Some(NumberMatcher::Range { min: Some(32), max: Some(38) }),
            value_match: None,
        };

        let message = MidiMessage::ControlChange { channel: 1, control: 34, value: 23 };
        assert!(matcher.matches(&message));

        let message = MidiMessage::ControlChange { channel: 2, control: 34, value: 23 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::ControlChange { channel: 1, control: 23, value: 92 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::NoteOn { channel: 1, key: 35, velocity: 8 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::NoteOff { channel: 1, key: 35, velocity: 50 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::PolyAftertouch { channel: 1, key: 35, value: 50 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::ProgramChange { channel: 1, program: 35 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::ChannelAftertouch { channel: 1, value: 35 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::PitchBendChange { channel: 1, value: 35 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::Other;
        assert!(!matcher.matches(&message));

        // Any channel, any key, but value at most 30
        let matcher = MidiEventMatcher::ControlChange {
            channel_match: None,
            control_match: None,
            value_match: Some(NumberMatcher::Range { min: None, max: Some(30) }),
        };

        let message = MidiMessage::ControlChange { channel: 5, control: 3, value: 20 };
        assert!(matcher.matches(&message));

        let message = MidiMessage::ControlChange { channel: 10, control: 124, value: 15 };
        assert!(matcher.matches(&message));

        let message = MidiMessage::ControlChange { channel: 4, control: 80, value: 70 };
        assert!(!matcher.matches(&message));
    }

    #[test]
    fn midi_event_match_program_change() {
        let matcher = MidiEventMatcher::ProgramChange {
            channel_match: Some(NumberMatcher::Val(1)),
            program_match: Some(NumberMatcher::Range { min: Some(32), max: Some(38) }),
        };

        let message = MidiMessage::ProgramChange { channel: 1, program: 34 };
        assert!(matcher.matches(&message));

        let message = MidiMessage::ProgramChange { channel: 2, program: 34 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::ProgramChange { channel: 1, program: 23 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::NoteOn { channel: 1, key: 35, velocity: 8 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::NoteOff { channel: 1, key: 35, velocity: 50 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::ControlChange { channel: 1, control: 35, value: 75 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::PolyAftertouch { channel: 1, key: 35, value: 50 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::ChannelAftertouch { channel: 1, value: 35 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::PitchBendChange { channel: 1, value: 35 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::Other;
        assert!(!matcher.matches(&message));
    }

    #[test]
    fn midi_event_channel_aftertouch() {
        let matcher = MidiEventMatcher::ChannelAftertouch {
            channel_match: Some(NumberMatcher::Val(1)),
            value_match: Some(NumberMatcher::Range { min: Some(32), max: Some(38) }),
        };

        let message = MidiMessage::ChannelAftertouch { channel: 1, value: 34 };
        assert!(matcher.matches(&message));

        let message = MidiMessage::ChannelAftertouch { channel: 2, value: 34 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::ChannelAftertouch { channel: 1, value: 23 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::NoteOn { channel: 1, key: 35, velocity: 8 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::NoteOff { channel: 1, key: 35, velocity: 50 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::ControlChange { channel: 1, control: 35, value: 75 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::PolyAftertouch { channel: 1, key: 35, value: 50 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::ProgramChange { channel: 1, program: 35 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::PitchBendChange { channel: 1, value: 35 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::Other;
        assert!(!matcher.matches(&message));
    }

    #[test]
    fn midi_event_match_pitch_bend_range() {
        let matcher = MidiEventMatcher::PitchBendRange {
            channel_match: Some(NumberMatcher::Val(1)),
            value_match: Some(NumberMatcher::Range { min: Some(32), max: Some(38) }),
        };

        let message = MidiMessage::PitchBendChange { channel: 1, value: 34 };
        assert!(matcher.matches(&message));

        let message = MidiMessage::PitchBendChange { channel: 2, value: 34 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::PitchBendChange { channel: 1, value: 23 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::NoteOn { channel: 1, key: 35, velocity: 8 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::NoteOff { channel: 1, key: 35, velocity: 50 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::ControlChange { channel: 1, control: 35, value: 75 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::PolyAftertouch { channel: 1, key: 35, value: 50 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::ProgramChange { channel: 1, program: 35 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::ChannelAftertouch { channel: 1, value: 35 };
        assert!(!matcher.matches(&message));

        let message = MidiMessage::Other;
        assert!(!matcher.matches(&message));
    }
}