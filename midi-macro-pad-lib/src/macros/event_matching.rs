use crate::match_checker::MatchChecker;
use crate::midi::MidiMessage;

pub mod midi;

pub enum EventMatcher {
    Midi(Box<dyn MatchChecker<MidiMessage>>),
    Other
}

// Temporary dummy type as a placeholder
type Precondition = bool;

pub enum Event<'a> {
    Midi(&'a MidiMessage),
    Other
}