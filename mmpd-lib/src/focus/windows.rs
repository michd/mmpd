use crate::focus::{FocusAdapter, FocusedWindow};

use windows_bindings::Windows::Win32::WindowsAndMessaging::{
    GetForegroundWindow,
    GetWindowTextW,
    GetClassNameW,
    HWND,
};

use windows_bindings::Windows::Win32::SystemServices::{
    PWSTR,
};

use std::os::windows::ffi::OsStringExt;
use std::ffi::OsString;

pub fn get_adapter() -> Option<Box<impl FocusAdapter>> {
    Windows::new().map(|windows| Box::new(windows))
}

struct Windows {}

impl Windows {
    fn new() -> Option<impl FocusAdapter> {
        Some(Windows { })
    }
}

impl FocusAdapter for Windows {
    fn get_focused_window(&self) -> Option<FocusedWindow> {
        let foreground_handle = get_focused_window_handle()?;

        Some(
            FocusedWindow {
                window_class: get_window_class(foreground_handle)?,
                window_name: get_window_name(foreground_handle)?
            }
        )
    }
}

const MAX_STR_LEN: usize = 1000;

fn get_focused_window_handle() -> Option<HWND> {
    let foreground_handle = unsafe { GetForegroundWindow() };

    if foreground_handle.is_null() {
        None
    } else {
        Some(foreground_handle)
    }
}

fn get_window_name(window_handle: HWND) -> Option<String> {
    let mut chars: [u16; MAX_STR_LEN] = [0; MAX_STR_LEN];
    let pwstr = PWSTR(chars.as_mut_ptr());

    let len = unsafe { GetWindowTextW(window_handle, pwstr, MAX_STR_LEN as i32) } as usize;

    if len == 0 {
        None
    } else {
        let os_string: OsString = OsStringExt::from_wide(&chars[0..len]);
        os_string.to_str().map(|s| s.to_string())
    }
}

fn get_window_class(window_handle: HWND) -> Option<Vec<String>> {
    let mut chars: [u16; MAX_STR_LEN] = [0; MAX_STR_LEN];
    let pwstr = PWSTR(chars.as_mut_ptr());

    let len = unsafe { GetClassNameW(window_handle, pwstr, MAX_STR_LEN as i32) } as usize;

    if len == 0 {
        None
    } else {
        let os_string: OsString = OsStringExt::from_wide(&chars[0..len]);
        os_string.to_str().map(|s| vec![s.to_string()])
    }
}