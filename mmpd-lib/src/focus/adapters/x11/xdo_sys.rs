extern crate libxdo_sys as sys;
use x11::xlib::Window;

pub struct XdoSys {
    handle: *mut sys::xdo_t,
}

macro_rules! xdosys (
    ($fncall: expr) => {
        unsafe {
            match $fncall {
                0 => Ok(()),
                _ => Err(())
            }
        }
    }
);

impl XdoSys {
    pub fn new() -> Option<XdoSys> {
        let display: *const i8 = ::std::ptr::null();
        let handle = unsafe { sys::xdo_new(display) };

        if handle.is_null() {
            return None;
        }

        Some(XdoSys { handle })
    }

    pub fn get_focused_window_id(&self) -> Option<Window> {
        let mut window: Window = 0;

        let result = xdosys!(sys::xdo_get_focused_window_sane(self.handle, &mut window));

        if result.is_ok() {
            Some(window)
        } else {
            None
        }
    }
}


impl Drop for XdoSys {
    fn drop(&mut self) {
        unsafe {
            sys::xdo_free(self.handle);
        }
    }
}
