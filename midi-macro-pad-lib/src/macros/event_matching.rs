use crate::match_checker::MatchChecker;
use crate::midi::MidiMessage;

pub mod midi;

/// Wrapping type enumerating any supported event matchers.
/// Mainly used as a single access point to match a analogously genericized event
/// against, using the MatchChecker implementation.
pub enum EventMatcher {
    /// Checks against Event::Midi events
    Midi(Box<dyn MatchChecker<MidiMessage>>),

    /// Checks against Event::Other events
    Other
}

impl <'a> MatchChecker<Event<'a>> for EventMatcher {
    fn matches(&self, val: &Event<'a>) -> bool {
        match val {
            Event::Midi(data) => self.matches_midi(data),
            Event::Other => self.matches_other(),
        }
    }
}

impl EventMatcher {
    fn matches_midi(&self, midi_message: &MidiMessage) -> bool {
        if let EventMatcher::Midi(match_checker) = self {
            match_checker.matches(midi_message)
        } else {
            false
        }
    }

    fn matches_other(&self) -> bool {
        if let EventMatcher::Other = self {
            true
        } else {
            false
        }
    }

    // Add other matches_x methods here for other event types/matches
}

/// Wrapping type enumerating all the kinds of events supported by EventMatcher.
pub enum Event<'a> {
    Midi(&'a MidiMessage),
    Other
}

#[cfg(test)]
mod tests {
    use crate::macros::event_matching::{EventMatcher, Event};
    use crate::macros::event_matching::midi::MidiEventMatcher;
    use crate::match_checker::{NumberMatcher, MatchChecker};
    use crate::midi::MidiMessage;

    #[test]
    fn event_wrapped_midi_message_matches() {
        let event_matcher = EventMatcher::Midi(Box::new(MidiEventMatcher::NoteOn {
            channel_match: Some(NumberMatcher::Val(1)),
            key_match: Some(NumberMatcher::Val(20)),
            velocity_match: Some(NumberMatcher::Val(100)),
        }));

        let event = Event::Midi(&MidiMessage::NoteOn {
            channel: 1,
            key: 20,
            velocity: 100
        });

        assert!(event_matcher.matches(&event));
    }

    #[test]
    fn event_wrapped_midi_with_mismatching_internals_doesnt_match() {
        let event_matcher = EventMatcher::Midi(Box::new(MidiEventMatcher::NoteOn {
            channel_match: Some(NumberMatcher::Val(1)),
            key_match: Some(NumberMatcher::Val(20)),
            velocity_match: Some(NumberMatcher::Val(100)),
        }));

        let event = Event::Midi(&MidiMessage::ChannelAftertouch {
            channel: 1,
            value: 30
        });

        assert!(!event_matcher.matches(&event));
    }

    #[test]
    fn event_wrapped_midi_doesnt_match_non_midi_event_matcher() {
        let event_matcher = EventMatcher::Other;

        let event = Event::Midi(&MidiMessage::NoteOn {
            channel: 4,
            key: 43,
            velocity: 100
        });

        assert!(!event_matcher.matches(&event));
    }

    #[test]
    fn event_other_doesnt_match_midi_event() {
        let event_matcher = EventMatcher::Midi(Box::new(MidiEventMatcher::NoteOn {
            channel_match: Some(NumberMatcher::Val(1)),
            key_match: Some(NumberMatcher::Val(20)),
            velocity_match: Some(NumberMatcher::Val(100)),
        }));

        let event = Event::Other;

        assert!(!event_matcher.matches(&event));
    }
}