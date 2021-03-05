use crate::macros::actions::Action;
use crate::macros::event_matching::{Event, EventMatcher};
use crate::match_checker::StringMatcher;
use crate::state::State;
use crate::macros::preconditions::Precondition;

pub mod actions;
pub mod event_matching;
pub mod preconditions;

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
    name: Option<String>,
    match_events: Vec<Box<EventMatcher>>,
    required_preconditions: Option<Vec<Precondition>>,
    actions: Vec<Action<'a>>,
    scope: Option<&'a Scope<'a>>
}

// TODO: given that there are 4 different arguments to new, 2 of which can be `None`,
// it would be good to switch to a builder pattern instead, making it clearer what parameter
// the argument being passed is for. Sadly rust does not support named arguments.
impl Macro<'_> {
    pub fn new<'a>(
        name: Option<String>,
        match_events: Vec<Box<EventMatcher>>,
        required_preconditions: Option<Vec<Precondition>>,
        actions: Vec<Action<'a>>,
        scope: Option<&'a Scope>
    ) -> Macro<'a> {
        Macro {
            name,
            match_events,
            required_preconditions,
            actions,
            scope
        }
    }

    pub fn name(&self) -> Option<&str> {
        if let Some(n) = &self.name {
            Some(n)
        } else {
            None
        }
    }

    /// Evaluates an incoming event, and it it matches against this macro's matching events,
    /// returns a list of actions to execute.
    pub fn evaluate<'b>(
        &self, event: &'b Event<'b>,
        state: &'b Box<dyn State>
    ) -> Option<&Vec<Action>> {

        if !state.matches_scope(&self.scope) {
            return None
        }

        if let Some(conditions) = &self.required_preconditions {
            if conditions.iter().any(|condition| !state.matches(condition)) {
                return None;
            }
        }

        if self.matches_event(event, state) {
            Some(&self.actions)
        } else {
            None
        }
    }

    fn matches_event<'b>(&self, event: &Event<'b>, state: &'b Box<dyn State>) -> bool {
        self.match_events.iter().any(|event_matcher| {
            event_matcher.matches(event, state)
        })
    }
}
