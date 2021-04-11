#[macro_use]
mod xdo_sys;
mod x11_sys;

use crate::focus::FocusAdapter;
use crate::focus::FocusedWindow;
use crate::focus::x11::xdo_sys::XdoSys;
use crate::focus::x11::x11_sys::X11Sys;

pub fn get_adapter() -> Option<Box<impl FocusAdapter>> {
    X11::new().map(|x11| Box::new(x11))
}

/// Blank struct to act as a handle for the trait.
pub struct X11 {
    xdosys: XdoSys,
    x11sys: X11Sys
}

impl X11 {
    /// Creates a new instance of this adapter
    pub fn new() -> Option<impl FocusAdapter> {
        Some(X11 {
            xdosys: XdoSys::new()?,
            x11sys: X11Sys::new()
        })
    }
}

impl FocusAdapter for X11 {
    /// Gathers and returns focused window information, if available
    fn get_focused_window(&self) -> Option<FocusedWindow> {
        self.x11sys.get_window_info(self.xdosys.get_focused_window_id()?)
    }
}