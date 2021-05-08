use crate::focus::{FocusAdapter, FocusedWindow};
use std::process::Command;
use std::path::Path;

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
        // Expected output data from this applescript (mac_os/get_window_info.scpt)
        // 3 lines of text:
        //   - application class ("displayed name")
        //   - application name ("name")
        //   - window title ("AXTitle" attribute value of window with AXMain is true)
        // The first two populate the window_class vec, while the last one populates window_name
        //
        // Note: ideally, we'd use an actual SDK to retrieve this info, but it appears no rust
        // binding for the APIs needed is available yet, and I didn't want to have the scope of
        // "adding Mac OS support" spiral out into creating bindings for AppKit's NSWorkspace
        // See:
        // https://developer.apple.com/documentation/appkit/nsworkspace/1532097-frontmostapplication
        // Further details of how to actually figure out the window title etc from there to be
        // figured out by whomever ends up implementing it the "right" way.
        let script_output = Command::new("osascript")
            .arg("-e")
            .arg(include_str!("mac_os/get_window_info.scpt"))
            .output()
            .ok()?;

        let output_raw = String::from_utf8(script_output.stdout).ok()?;
        let output_lines: Vec<&str> = output_raw.trim().split("\n").collect();

        // If number of lines doesn't match what expect we can't reliably retrieve correct info
        if output_lines.len() != 4 {
            return None;
        }

        let executable_path = output_lines[3].to_string();
        let exec_path = Path::new(executable_path.as_str());
        let mut executable_basename: Option<String> = None;

        if let Some(file_name) = exec_path.file_name() {
            if let Some(file_name) = file_name.to_str() {
                executable_basename = Some(file_name.to_string());
            }
        }

        Some(FocusedWindow {
            // Index access won't panic since we checked length above
            window_class: vec![
                output_lines[0].to_string(),
                output_lines[1].to_string()
            ],
            window_name: output_lines[2].to_string(),

            executable_path: Some(executable_path),
            executable_basename,
        })
    }
}