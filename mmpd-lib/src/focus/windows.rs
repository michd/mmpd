use crate::focus::{FocusAdapter, FocusedWindow};

pub fn get_adapter() -> Option<Box<impl FocusAdapter>> {
    Windows::new().map(|windows| Box::new(windows))
}

pub struct Windows {
}

impl Windows {
    pub fn new() -> Option<impl FocusAdapter> {
        Some(Windows {

        })
    }
}

impl FocusAdapter for Windows {
    fn get_focused_window(&self) -> Option<FocusedWindow> {
        todo!()
    }
}