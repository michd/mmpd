use crate::macros::actions::Action;
use crate::macros::event_matching::{Event, EventMatcher};
use crate::match_checker::{StringMatcher, MatchChecker};
use crate::state::State;

pub mod actions;
pub mod event_matching;

pub struct Scope<'a> {
    pub window_class: Option<StringMatcher<'a>>,
    pub window_name: Option<StringMatcher<'a>>,
}

impl Scope<'_> {
    pub fn new<'a>(
        window_class: Option<StringMatcher<'a>>,
        window_name: Option<StringMatcher<'a>>
    ) -> Scope<'a> {
        Scope { window_class, window_name }
    }
}

pub struct Macro<'a> {
    match_events: Vec<Box<EventMatcher>>,
    // TODO: required_preconditions: Option<Vec<Precondition>>
    actions: Vec<Action<'a>>,
    scope: Option<&'a Scope<'a>>
}

impl Macro<'_> {
    pub fn new<'a>(match_events: Vec<Box<EventMatcher>>, actions: Vec<Action<'a>>, scope: Option<&'a Scope>) -> Macro<'a> {
        Macro {
            match_events,
            actions,
            scope
        }
    }

    /// Evaluates an incoming event, and it it matches against this macro's matching events,
    /// returns a list of actions to execute.
    /// TODO: also pass in an object that provides access to relevant state for preconditions.
    pub fn evaluate<'b>(
        &self, event: &'b Event<'b>,
        state: &'b Box<dyn State>
    ) -> Option<&Vec<Action>> {
        let event_matches = self.matches_event(event, state);

        // TODO: check macro-level preconditions against state as well
        if event_matches && state.matches_scope(&self.scope) {
            Some(&self.actions)
        } else {
            None
        }
    }

    fn matches_event<'b>(&self, event: &Event<'b>, _state: &'b Box<dyn State>) -> bool {
        // TODO: check preconditions associated with this event matcher against state
        self.match_events.iter().any(|event_matcher| {
            event_matcher.matches(event)
        })
    }
}
