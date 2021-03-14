use crate::focus::adapters::FocusAdapter;
use std::process::Command;
use crate::focus::FocusedWindow;
use std::str;

/// Blank struct to act as a handle for the trait.
pub struct Xdo {}

impl Xdo {
    /// Creates a new instance of this adapter
    pub fn new() -> Option<impl FocusAdapter> {
        // TODO check here whether xdotool and xprop are installed
        // instead of letting actual commands fail
        Some(Xdo {})
    }
}

impl FocusAdapter for Xdo {
    /// Gathers and returns focused window information, if available
    fn get_focused_window(&self) -> Option<FocusedWindow> {
        let focused_window_id = get_raw_window_id()?;
        let raw_window_info = get_xprop_info(&focused_window_id)?;
        Some(parse_window_info(&raw_window_info))
    }
}

/// Uses the xdotool executable to get the currently focused window's window id as a string.
/// If any part of this process doesn't work, returns None.
fn get_raw_window_id() -> Option<String> {
    let raw_output = Command::new("xdotool")
        .arg("getwindowfocus")
        .output()
        .ok()?;

    Some(String::from(str::from_utf8(raw_output.stdout.as_slice()).ok()?))
}

/// For a given window id, uses the xprop executable to retrieve class and name info
/// If any part of this process fails, returns None.
fn get_xprop_info(window_id: &str) -> Option<String> {
    let raw_output = Command::new("xprop")
        .arg("-root")
        .arg("-id")
        .arg(window_id)
        .arg("WM_CLASS")
        .arg("WM_NAME")
        .output()
        .ok()?;

    Some(String::from(str::from_utf8(raw_output.stdout.as_slice()).ok()?))
}

/// Parses xprop output from get_xprop_info into a FocusedWindow instance
/// If parsing fails, that property of FocusedWindow is left blank.
fn parse_window_info(raw_window_info: &str) -> FocusedWindow {
    let mut fw = FocusedWindow::blank();

    for line in raw_window_info.lines() {
        if line.starts_with("WM_CLASS(STRING) = \"") {
            let len = line.len();
            fw.window_class = parse_quoted_list(&line[20..len - 1]);
        }

        if line.starts_with("WM_NAME(STRING) = \"") {
            let len = line.len();
            fw.window_name = String::from(&line[19..len - 1]);
        }

        if line.starts_with("WM_NAME(COMPOUND_TEXT) = \"") {
            let len = line.len();
            fw.window_name = String::from(&line[26..len - 1]);
        }
    }

    return fw;
}

/// Parses a quoted list of strings into a Vector.
/// Not very versatile, assumes the strings are all separated by ", ", doesn't take into account
/// any strings that may _contain_ quotes.
fn parse_quoted_list(list: &str) -> Vec<String> {
    let split = list.split("\", \"");

    let result: Vec<&str> = split.collect();
    let mut converted_result: Vec<String> = vec![];

    for item in result.iter() {
        converted_result.push(String::from(item.to_owned()))
    }

    return converted_result
}
