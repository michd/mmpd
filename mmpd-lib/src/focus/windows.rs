use crate::focus::{FocusAdapter, FocusedWindow};

use windows_bindings::Windows::Win32::WindowsAndMessaging::{
    GetForegroundWindow,
    GetWindowTextW,
    GetClassNameW,
    GetWindowThreadProcessId,
    HWND,
};

use windows_bindings::Windows::Win32::SystemServices::{
    BOOL,
    PWSTR,
    PROCESS_ACCESS_RIGHTS,
    OpenProcess,
    QueryFullProcessImageNameW,
    QueryFullProcessImageName_dwFlags,
};

use std::os::windows::ffi::OsStringExt;
use std::ffi::OsString;
use std::path::Path;

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
        let window_class = get_window_class(foreground_handle)?;
        let window_name = get_window_name(foreground_handle)?;

        let executable_path = get_process_executable(
            get_window_process_id(foreground_handle)
        );

        let mut executable_basename: Option<String> = None;

        if let Some(exec_path) = executable_path.clone() {
            let exec_path = Path::new(exec_path.as_str());

            if let Some(file_name) = exec_path.file_name() {
                if let Some(file_name) = file_name.to_str() {
                    executable_basename = Some(file_name.to_string());
                }
            }
        }

        Some(
            FocusedWindow {
                window_class,
                window_name,
                executable_path,
                executable_basename
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

fn get_window_process_id(window_handle: HWND) -> u32 {
    let mut pid: u32 = 0;
    let _ = unsafe { GetWindowThreadProcessId(window_handle, &mut pid) };
    pid
}

fn get_process_executable(pid: u32) -> Option<String> {
    let process_handle = unsafe {
        OpenProcess(
            PROCESS_ACCESS_RIGHTS::PROCESS_QUERY_LIMITED_INFORMATION,
            false, // bInheritHandle: Not having sub-processes inherit this handle
            pid    // dwProcessId: process we're getting info for
        )
    };

    if process_handle.is_null() {
        return None;
    }

    // Storage for the path
    let mut chars: [u16; MAX_STR_LEN] = [0; MAX_STR_LEN];
    let pwstr = PWSTR(chars.as_mut_ptr());
    let mut len: u32 = MAX_STR_LEN as u32;

    let result = unsafe {
        QueryFullProcessImageNameW(
            process_handle,
            QueryFullProcessImageName_dwFlags(0),
            pwstr,
            &mut len,
        )
    };

    // If we failed to get the info for any reason, return None.
    // If this is hit at some point, use GetLastError to figure out why.
    if let BOOL(0) = result {
        return None;
    }

    let os_string: OsString = OsStringExt::from_wide(&chars[0..len as usize]);
    os_string.to_str().map(|s| s.to_string())
}