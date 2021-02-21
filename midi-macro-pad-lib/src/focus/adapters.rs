use crate::focus::adapters::xdo::Xdo;
use crate::focus::FocusedWindow;

mod xdo;


pub fn get_adapter() -> Option<Box<dyn FocusAdapter>> {
    // TODO: provide different adapter based on platform as needed
    let adapter = Xdo::new();

    match adapter {
        Some(a) => Some(Box::new(a)),
        None => None
    }
}

pub trait FocusAdapter {
    fn get_focused_window(&self) -> Option<FocusedWindow>;
}