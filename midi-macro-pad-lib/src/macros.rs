use crate::macros::actions::Action;
use crate::macros::event_matching::{Event, EventMatcher};

pub mod actions;
pub mod event_matching;

pub struct Macro<'a> {
    match_events: Vec<Box<EventMatcher<'a>>>,
    // TODO: required_preconditions: Vec<Precondition>
    actions: Vec<Action<'a>>
}

impl<'a> Macro<'a> {
//    pub fn new() -> Macro {
//    }

    /// Evaluates an incoming event, and it it matches against this macro's matching events,
    /// returns a list of actions to execute.
    /// TODO: also pass in an object that provides access to relevant state for preconditions.
    pub fn evaluate(&self, event: &'a Event) -> Option<&Vec<Action>>{
        let event_matches = self.matches_event(event);

        if event_matches {
            Some(&self.actions)
        } else {
            None
        }
    }

    fn matches_event(&self, event: &'a Event) -> bool {
        self.match_events.iter().any(|event_matcher| {
            match event_matcher.as_ref() {
                EventMatcher::Midi(match_checker) => {
                    match event {
                        Event::Midi { data, .. } => match_checker.matches(data),
                        _ => false
                    }
                }

                _ => false
            }
        })
    }
}


