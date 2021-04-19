use crate::focus::{FocusAdapter, FocusedWindow};

pub fn get_adapter() -> Option<Box<impl FocusAdapter>> {
    MacOs::new().map(|mac_os| Box::new(mac_os))
}

struct MacOs {}

impl MacOs {
    fn new() -> Option<impl FocusAdapter> {
        Some(MacOs {})
    }
}

impl FocusAdapter for MacOs {
    fn get_focused_window(&self) -> Option<FocusedWindow> {
        println!("Todo: get_focused_window on MacOs");
        None
    }
}
