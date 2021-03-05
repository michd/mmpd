use crate::macros::Scope;
use crate::focus::adapters::FocusAdapter;
use crate::match_checker::MatchChecker;

pub trait State {
    fn matches_scope(&self, scope: &Option<&Scope>) -> bool;

    // TODO: precondition checking
}

pub fn new(
    focus_adapter: Box<dyn FocusAdapter>
) -> Box<dyn State> {
    StateImpl::new(focus_adapter)
}

struct StateImpl {
    focus_adapter: Box<dyn FocusAdapter>
}

impl StateImpl {
    pub fn new(
        focus_adapter: Box<dyn FocusAdapter>
    ) -> Box<dyn State> {
        Box::new(StateImpl {
            focus_adapter
        })
    }
}

impl State for StateImpl {
    fn matches_scope<'a>(&self, scope: &Option<&Scope<'a>>) -> bool {
        if scope.is_none() {
            return true
        }

        let scope = scope.unwrap();

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
}