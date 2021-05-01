mod midi_state;

use crate::macros::Scope;
use crate::focus::FocusAdapter;
use crate::match_checker::MatchChecker;
use crate::macros::preconditions::{Precondition, PreconditionType};

#[cfg(test)]
use mockall::automock;
use crate::macros::event_matching::Event;
use crate::state::midi_state::MidiState;

#[cfg_attr(test, automock)]
pub trait State {
    fn process_event(&mut self, event: &Event);

    fn matches_scope(&self, scope: &Option<Scope>) -> bool;

    fn matches_precondition(&self, precondition: &Precondition) -> bool;
}

pub fn new(
    focus_adapter: Box<dyn FocusAdapter>
) -> Box<dyn State> {
    StateImpl::new(focus_adapter)
}

struct StateImpl {
    focus_adapter: Box<dyn FocusAdapter>,
    midi: MidiState
}

impl StateImpl {
    pub fn new(
        focus_adapter: Box<dyn FocusAdapter>
    ) -> Box<dyn State> {
        Box::new(StateImpl {
            focus_adapter,
            midi: MidiState::new()
        })
    }
}

impl State for StateImpl {
    fn process_event(&mut self, event: &Event) {
        match event {
            Event::Midi(midi_msg) => self.midi.process_message(midi_msg),
            Event::Other => {}
        }
    }

    fn matches_scope(&self, scope: &Option<Scope>) -> bool {
        if scope.is_none() {
            return true
        }

        let scope = scope.clone().unwrap();

        let window = self.focus_adapter.get_focused_window();

        // If there is no focused window, but we have scope qualifiers, we cannot match
        if window.is_none() {
            return false
        }

        let window = window.unwrap();

        if let Some(window_name) = &scope.window_name {
            if !window_name.matches(&window.window_name.as_ref()) {
                return false
            }
        }

        if let Some(window_class) = &scope.window_class {
            if !window.window_class.iter().any(|wc| window_class.matches(&wc.as_ref())) {
                return false;
            }
        }

        if let Some(executable_path) = &scope.executable_path {
            if window.executable_path.is_none() {
                return false;
            }

            let w_exec_path = window.executable_path.unwrap();

            if !executable_path.matches(&w_exec_path.as_ref()) {
                return false;
            }
        }

        if let Some(executable_basename) = &scope.executable_basename {
            if window.executable_basename.is_none() {
                return false;
            }

            let w_exec_basename = window.executable_basename.unwrap();

            if !executable_basename.matches(&w_exec_basename.as_ref()) {
                return false;
            }
        }

        true
    }

    fn matches_precondition(&self, precondition: &Precondition) -> bool {
        let normal_match = match &precondition.condition {
            PreconditionType::Midi(condition) => self.midi.matches(condition),
            PreconditionType::Other => true
        };


        if precondition.invert  {
            !normal_match
        } else {
            normal_match
        }
    }
}

// TODO: tests for StateImpl