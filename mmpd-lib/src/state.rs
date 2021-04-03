mod midi_state;

use crate::macros::Scope;
use crate::focus::adapters::FocusAdapter;
use crate::match_checker::MatchChecker;
use crate::macros::preconditions::Precondition;

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

        if scope.window_name.is_none() && scope.window_class.is_none() {
            return true
        }

        let window = self.focus_adapter.get_focused_window();

        return if let Some(window) = window {
            if let Some(window_name) = &scope.window_name {
                if !window_name.matches(&window.window_name.as_ref()) {
                    return false
                }
            }

            if let Some(window_class) = &scope.window_class {
                for wcls in window.window_class {
                    if window_class.matches(&wcls.as_ref()) {
                        return true
                    }
                }

                return false
            }

            true
        } else {
            true
        }
    }

    fn matches_precondition(&self, precondition: &Precondition) -> bool {
        match precondition {
            Precondition::Midi(condition) => self.midi.matches(condition),
            Precondition::Other => true
        }
    }
}