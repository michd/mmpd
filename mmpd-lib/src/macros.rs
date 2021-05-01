use crate::macros::actions::Action;
use crate::macros::event_matching::{Event, EventMatcher};
use crate::match_checker::StringMatcher;
use crate::state::State;
use crate::macros::preconditions::Precondition;

pub mod actions;
pub mod event_matching;
pub mod preconditions;

#[derive(Clone, PartialEq, Debug)]
pub struct Scope {
    pub window_class: Option<StringMatcher>,
    pub window_name: Option<StringMatcher>,
    pub executable_path: Option<StringMatcher>,
    pub executable_basename: Option<StringMatcher>
}

impl Scope {
    pub fn new<'a>(
        window_class: Option<StringMatcher>,
        window_name: Option<StringMatcher>,
        executable_path: Option<StringMatcher>,
        executable_basename: Option<StringMatcher>
    ) -> Scope {
        Scope {
            window_class,
            window_name,
            executable_path,
            executable_basename
        }
    }

    /// Turns the instance into an option, as a convenience for checking whether _any_ of its
    /// matchers are `Some`. Returns `None` if all contained matchers are `None`.
    pub fn into_option(self) -> Option<Scope> {
        if self.window_class.is_some() ||
            self.window_name.is_some() ||
            self.executable_path.is_some() ||
            self.executable_basename.is_some() {
            Some(self)
        } else {
            None
        }
    }
}

pub struct MacroBuilder {
    name: Option<String>,
    match_events: Vec<EventMatcher>,
    required_preconditions: Option<Vec<Precondition>>,
    actions: Vec<Action>,
    scope: Option<Scope>
}

impl <'a> MacroBuilder {
    pub fn from_event_matcher(
        event_matcher: EventMatcher
    ) -> MacroBuilder {
        MacroBuilder {
            name: None,
            match_events: vec![event_matcher],
            required_preconditions: None,
            actions: vec![],
            scope: None
        }
    }

    pub fn from_event_matchers(
        event_matchers: Vec<EventMatcher>
    ) -> MacroBuilder {
        MacroBuilder {
            name: None,
            match_events: event_matchers,
            required_preconditions: None,
            actions: vec![],
            scope: None
        }
    }

    pub fn set_event_matchers(mut self, event_matchers: Vec<EventMatcher>) -> Self {
        self.match_events = event_matchers;
        self
    }

    pub fn add_event_matcher(mut self, event_matcher: EventMatcher) -> Self {
        self.match_events.push(event_matcher);
        self
    }

    pub fn set_actions(mut self, actions: Vec<Action>) -> Self {
        self.actions = actions;
        self
    }

    pub fn add_action(mut self, action: Action) -> Self {
        self.actions.push(action);
        self
    }

    pub fn set_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn set_preconditions(mut self, preconditions: Vec<Precondition>) -> Self {
        self.required_preconditions = Some(preconditions);
        self
    }

    pub fn add_precondition(mut self, precondition: Precondition) -> Self {
        let mut new_preconditions : Vec<Precondition> = vec![];

        if let Some(_) = self.required_preconditions {
            let mut preconditions = self.required_preconditions.take().unwrap();
            new_preconditions.append(&mut preconditions);
            new_preconditions.push(precondition);
            self.required_preconditions = Some(new_preconditions);
        } else {
            self.required_preconditions = Some(vec![precondition]);
        }

        self
    }

    pub fn set_scope(mut self, scope: Scope) -> Self {
        self.scope = Some(scope);
        self
    }

    pub fn build(self) -> Macro {
        Macro {
            name: self.name,
            match_events: self.match_events,
            required_preconditions: self.required_preconditions,
            actions: self.actions,
            scope: self.scope
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct Macro {
    pub(crate) name: Option<String>,
    pub(crate) match_events: Vec<EventMatcher>,
    pub(crate) required_preconditions: Option<Vec<Precondition>>,
    pub(crate) actions: Vec<Action>,
    pub(crate) scope: Option<Scope>
}

impl Macro {
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
        &self, event: &'b Event,
        state: &'b Box<dyn State>
    ) -> Option<&Vec<Action>> {

        // TODO: rejigger the order of these checks so the most expensive check is done last
        if !state.matches_scope(&self.scope) {
            return None
        }

        if let Some(conditions) = &self.required_preconditions {
            if conditions.iter().any(|condition| !state.matches_precondition(condition)) {
                return None;
            }
        }

        if self.matches_event(event, state) {
            Some(&self.actions)
        } else {
            None
        }
    }

    fn matches_event<'b>(&self, event: &Event, state: &'b Box<dyn State>) -> bool {
        self.match_events.iter().any(|event_matcher| {
            event_matcher.matches(event, state)
        })
    }
}
