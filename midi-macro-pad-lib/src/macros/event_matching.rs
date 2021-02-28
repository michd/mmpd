use crate::match_checker::MatchChecker;
use crate::midi::MidiMessage;

pub mod midi;

pub enum EventMatcher<'a> {
    Midi(Box<dyn MatchChecker<&'a MidiMessage>>),
    Other
}

// Temporary dummy type as a placeholder
type Precondition = bool;

pub enum Event {
    Midi {
        data: MidiMessage,
        required_preconditions: Option<Vec<Precondition>>
    },

    Other {
        required_preconditions: Option<Vec<Precondition>>
    }
}