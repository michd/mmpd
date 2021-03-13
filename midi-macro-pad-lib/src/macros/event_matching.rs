use crate::match_checker::MatchChecker;
use crate::midi::MidiMessage;
use crate::macros::preconditions::Precondition;
use crate::state::State;
use crate::macros::event_matching::midi::MidiEventMatcher;

pub mod midi;

/// An eventMatcher includes a matcher to validate whether a given event
/// matches that what is defined, as well as an optional list of preconditions that
/// must be satisfied for the entire event to match.
#[derive(PartialEq, Debug)]
pub struct EventMatcher {
    pub (crate) matcher: MatcherType,
    pub (crate) required_preconditions: Option<Vec<Precondition>>
}

impl EventMatcher {
    pub fn new(
        matcher: MatcherType,
        required_preconditions: Option<Vec<Precondition>>
    ) -> EventMatcher {
        EventMatcher { matcher, required_preconditions }
    }

    pub fn matches<'a>(&self, event: &Event<'a>, state: &'a Box<dyn State>) -> bool {
        // If there are any preconditions to satisfy, first check those against state.
        // If any one precondition is not satisfies, no further precondition is evaluated,
        // nor is the event object matched against MatcherType.
        if let Some(conditions) = &self.required_preconditions {
            if conditions.iter().any(|condition| !state.matches(condition)) {
                return false;
            }
        }

        // If we're here, all preconditions, if any, are satisfied
        self.matcher.matches(event)
    }

    pub fn get_preconditions(&self) -> Option<Vec<&Precondition>> {
        if let Some(conditions) = &self.required_preconditions {
            Some(conditions.iter().map(|c| c).collect())
        } else {
            None
        }
    }
}

/// Wrapping type enumerating any supported event matchers.
/// Mainly used as a single access point to match a analogously genericized event
/// against, using the MatchChecker implementation.
#[derive(PartialEq, Debug)]
pub enum MatcherType {
    /// Checks against Event::Midi events
    Midi(MidiEventMatcher),

    /// Checks against Event::Other events
    Other
}

impl <'a> MatchChecker<Event<'a>> for MatcherType {
    fn matches(&self, val: &Event<'a>) -> bool {
        match val {
            Event::Midi(data) => self.matches_midi(data),
            Event::Other => self.matches_other(),
        }
    }
}

impl MatcherType {
    fn matches_midi(&self, midi_message: &MidiMessage) -> bool {
        if let MatcherType::Midi(match_checker) = self {
            match_checker.matches(midi_message)
        } else {
            false
        }
    }

    fn matches_other(&self) -> bool {
        if let MatcherType::Other = self {
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
    use crate::macros::event_matching::{MatcherType, Event, EventMatcher};
    use crate::macros::event_matching::midi::MidiEventMatcher;
    use crate::match_checker::NumberMatcher;
    use crate::midi::MidiMessage;
    use crate::state::{MockState, State};

    #[test]
    fn event_wrapped_midi_message_matches() {
        let state = MockState::new();
        let state_box: Box<dyn State> = Box::new(state);

        let event_matcher = EventMatcher::new(
            MatcherType::Midi(MidiEventMatcher::NoteOn {
                    channel_match: Some(NumberMatcher::Val(1)),
                    key_match: Some(NumberMatcher::Val(20)),
                    velocity_match: Some(NumberMatcher::Val(100)),
                }
            ),

            None
        );

        let event = Event::Midi(&MidiMessage::NoteOn {
            channel: 1,
            key: 20,
            velocity: 100
        });

        assert!(event_matcher.matches(&event, &state_box));
    }

    #[test]
    fn event_wrapped_midi_with_mismatching_internals_doesnt_match() {
        let state = MockState::new();
        let state_box: Box<dyn State> = Box::new(state);


        let event_matcher = EventMatcher::new(
            MatcherType::Midi(
                MidiEventMatcher::NoteOn {
                    channel_match: Some(NumberMatcher::Val(1)),
                    key_match: Some(NumberMatcher::Val(20)),
                    velocity_match: Some(NumberMatcher::Val(100)),
                }
            ),

            None
        );

        let event = Event::Midi(&MidiMessage::ChannelAftertouch {
            channel: 1,
            value: 30
        });

        assert!(!event_matcher.matches(&event, &state_box));
    }

    #[test]
    fn event_wrapped_midi_doesnt_match_non_midi_event_matcher() {
        let state = MockState::new();
        let state_box: Box<dyn State> = Box::new(state);

        let event_matcher = EventMatcher::new(
            MatcherType::Other,
            None
        );

        let event = Event::Midi(&MidiMessage::NoteOn {
            channel: 4,
            key: 43,
            velocity: 100
        });

        assert!(!event_matcher.matches(&event, &state_box));
    }

    #[test]
    fn event_other_doesnt_match_midi_event() {
        let state = MockState::new();
        let state_box: Box<dyn State> = Box::new(state);

        let event_matcher = EventMatcher::new(
            MatcherType::Midi(
                MidiEventMatcher::NoteOn {
                    channel_match: Some(NumberMatcher::Val(1)),
                    key_match: Some(NumberMatcher::Val(20)),
                    velocity_match: Some(NumberMatcher::Val(100)),
                }
            ),

            None
        );

        let event = Event::Other;

        assert!(!event_matcher.matches(&event, &state_box));
    }

    // TODO: tests where preconditions are evaluated
}