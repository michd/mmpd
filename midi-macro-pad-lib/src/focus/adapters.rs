use crate::focus::adapters::xdo::Xdo;
use crate::focus::FocusedWindow;

mod xdo;

/// Provides an adapter implementing FocusAdapter based on platform
/// At the moment it just provides the xdo implementation.
pub fn get_adapter() -> Option<Box<dyn FocusAdapter>> {
    // TODO: provide different adapter based on platform as needed
    let adapter = Xdo::new();

    match adapter {
        Some(a) => Some(Box::new(a)),
        None => None
    }
}

/// Adapters implementing this trait can be asked to provided data on the currently focused window.
pub trait FocusAdapter {
    /// Returns an instance of FocusedWindow with relevant focused window info (class, name) if
    /// available, None otherwise.
    fn get_focused_window(&self) -> Option<FocusedWindow>;
}