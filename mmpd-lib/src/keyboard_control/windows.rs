use crate::keyboard_control::{KeyboardControlAdapter, KeyboardResult, KeyboardControlError};
use windows_bindings::Windows::Win32::KeyboardAndMouseInput::{
    SendInput,
    INPUT,
    INPUT_typeFlags,
    KEYBDINPUT,
    INPUT_0,
    keybd_eventFlags
};

use windows_bindings::Windows::Win32::WindowsAndMessaging::*;
use std::os::windows::ffi::OsStrExt;
use std::{mem, thread};
use core::time;
use std::ffi::{OsStr};

pub fn get_adapter() -> Option<Box<impl KeyboardControlAdapter>> {
   Windows::new().map(|windows| Box::new(windows))
}

pub struct Windows {}

impl Windows {
    pub fn new() -> Option<impl KeyboardControlAdapter> {
        Some(Windows {})
    }
}

impl KeyboardControlAdapter for Windows {
    fn send_keysequence(&self, sequence: &str, delay_microsecs: u32) -> KeyboardResult {
        let input_sequence: Vec<INPUT> = build_keysequence_inputs(sequence)?;
        send_inputs(input_sequence, delay_microsecs);
        Ok(())
    }

    fn send_text(&self, text: &str, delay_microsecs: u32) -> KeyboardResult {
        let input_sequence = build_enter_text_inputs(text);
        send_inputs(input_sequence, delay_microsecs);
        Ok(())
    }
}

/// Sends each input struct one by one, sleeping the thread for delay_microsecs microseconds between
/// each one.
fn send_inputs(inputs: Vec<INPUT>, delay_microsecs: u32) {
    let mut input_arr = [key_to_win_input(0, false)]; // Fill array with a dummy INPUT

    for input in inputs {
        input_arr[0] = input;

        unsafe {
            // see
            // https://docs.microsoft.com/en-gb/windows/win32/api/winuser/nf-winuser-sendinput?redirectedfrom=MSDN
            SendInput(
                1, // length of input_arr to iterate over
                input_arr.as_mut_ptr(), // Pointer to input array
                mem::size_of::<INPUT>() as i32 // Size of a single INPUT
            );
        }

        thread::sleep(time::Duration::from_micros(delay_microsecs as u64));
    }
}

/// Splits a keysequence (like "ctrl+shift+t" up into its components like "ctrl", "shift", "t",
/// converts them to virtual key codes that windows understands, and forms a list of INPUT
/// structs that presses them all down in sequence, then releases them all in opposite order
fn build_keysequence_inputs(str_sequence: &str) -> Result<Vec<INPUT>, KeyboardControlError> {
    let split = str_sequence.split('+');

    let virtual_keys: Vec<u32> = split.into_iter().map(|sym| {
        get_virtual_keycode(sym).ok_or_else(|| {
            KeyboardControlError::InvalidKey(sym.to_string())
        })
    }).collect::<Result<Vec<u32>, KeyboardControlError>>()?;

    let mut inputs: Vec<INPUT> = vec![];

    let mut forward_inputs: Vec<INPUT> =
        virtual_keys
            .iter()
            .map(|key_code| key_to_win_input(*key_code, false))
            .collect();

    inputs.append(&mut forward_inputs);

    let mut reverse_inputs : Vec<INPUT> =
        virtual_keys
            .iter()
            .rev()
            .map(|key_code| key_to_win_input(*key_code, true))
            .collect();

    inputs.append(&mut reverse_inputs);

    Ok(inputs)
}

// Builds a keyboard INPUT struct from a given keycode
fn key_to_win_input(key_code: u32, is_release: bool) -> INPUT {
    let kb_int = KEYBDINPUT {
        wVk: key_code as u16,
        wScan: 0,
        dwFlags: if is_release { keybd_eventFlags::KEYEVENTF_KEYUP } else { keybd_eventFlags(0) },
        time: 0,
        dwExtraInfo: 0
    };

    INPUT {
        r#type: INPUT_typeFlags::INPUT_KEYBOARD,
        Anonymous: INPUT_0 { ki: kb_int }
    }
}

/// Builds a list of INPUT structs to form key presses for each character in text, converted
/// to Windows' wchar (u16).
fn build_enter_text_inputs(text: &str) -> Vec<INPUT> {
    let mut inputs: Vec<INPUT> = vec![];

    OsStr::new(text)
        .encode_wide()
        .collect::<Vec<u16>>()
        .iter()
        .for_each(|wc| {
            inputs.append(&mut char_to_win_inputs(*wc))
        });

    inputs
}

/// From a wide char (u16), builds a keypress made of a key down and key up event for that
/// character
fn char_to_win_inputs(wide_char: u16) -> Vec<INPUT> {
    // Key down event
    let kb_int_down = KEYBDINPUT {
        wVk: 0,
        wScan: wide_char,
        dwFlags: keybd_eventFlags::KEYEVENTF_UNICODE,
        time: 0,
        dwExtraInfo: 0
    };

    // Key up event, constructed by cloning the down event and adding on the keyup flag
    let mut kb_int_up = kb_int_down.clone();
    kb_int_up.dwFlags = kb_int_up.dwFlags | keybd_eventFlags::KEYEVENTF_KEYUP;

    vec![
        INPUT {
            r#type: INPUT_typeFlags::INPUT_KEYBOARD,
            Anonymous: INPUT_0 { ki: kb_int_down }
        },
        INPUT {
            r#type: INPUT_typeFlags::INPUT_KEYBOARD,
            Anonymous: INPUT_0 { ki: kb_int_up }
        }
    ]
}

/// Translates a string key representation to the virtual key code as Windows uses.
///
/// Not every possible one is implemented. If support is needed for something not covered, add it
/// in here.
///
/// ## References:
///
/// - [Virtual-Key Codes (Winuser.h)](https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes)
/// - [keysymdef.h from X11](https://code.woboq.org/kde/include/X11/keysymdef.h.html)
fn get_virtual_keycode(str_sym: &str) -> Option<u32> {
    Some(match str_sym.to_lowercase().as_str() {
        "backspace" => VK_BACK,
        "tab" => VK_TAB,
        "clear" => VK_CLEAR,
        "return" => VK_RETURN,
        "pause" => VK_PAUSE,
        "scroll" => VK_SCROLL,
        "escape" => VK_ESCAPE,
        "delete" => VK_DELETE,
        "ctrl" => VK_CONTROL,
        "ctrl_l" => VK_LCONTROL,
        "ctrl_r" => VK_RCONTROL,
        "alt" => VK_MENU,
        "alt_l" => VK_LMENU,
        "alt_r" => VK_RMENU,
        "shift" => VK_SHIFT,
        "shift_l" => VK_LSHIFT,
        "shift_r" => VK_RSHIFT,
        "home" => VK_HOME,
        "left" => VK_LEFT,
        "up" => VK_UP,
        "right" => VK_RIGHT,
        "down" => VK_DOWN,
        "prior" | "page_up" => VK_PRIOR,
        "next" | "page_down" => VK_NEXT,
        "end" => VK_END,
        "begin" => VK_HOME,
        "space" => VK_SPACE,
        "f1" => VK_F1,
        "f2" => VK_F2,
        "f3" => VK_F3,
        "f4" => VK_F4,
        "f5" => VK_F5,
        "f6" => VK_F6,
        "f7" => VK_F7,
        "f8" => VK_F8,
        "f9" => VK_F9,
        "f10" => VK_F10,
        "f11" => VK_F11,
        "f12" => VK_F12,
        "f13" => VK_F13,
        "f14" => VK_F14,
        "f15" => VK_F15,
        "f16" => VK_F16,
        "f17" => VK_F17,
        "f18" => VK_F18,
        "f19" => VK_F19,
        "f20" => VK_F20,
        "f21" => VK_F21,
        "f22" => VK_F22,
        "f23" => VK_F23,
        "f24" => VK_F24,
        "caps_lock" => VK_CAPITAL,
        "super" | "super_l" => VK_LWIN,
        "super_r" => VK_RWIN,
        "kp_0" => VK_NUMPAD0,
        "kp_1" => VK_NUMPAD1,
        "kp_2" => VK_NUMPAD2,
        "kp_3" => VK_NUMPAD3,
        "kp_4" => VK_NUMPAD4,
        "kp_5" => VK_NUMPAD5,
        "kp_6" => VK_NUMPAD6,
        "kp_7" => VK_NUMPAD7,
        "kp_8" => VK_NUMPAD8,
        "kp_9" => VK_NUMPAD9,
        "0" => 0x30,
        "1" => 0x31,
        "2" => 0x32,
        "3" => 0x33,
        "4" => 0x34,
        "5" => 0x35,
        "6" => 0x36,
        "7" => 0x37,
        "8" => 0x38,
        "9" => 0x39,
        "a" => 0x41,
        "b" => 0x42,
        "c" => 0x43,
        "d" => 0x44,
        "e" => 0x45,
        "f" => 0x46,
        "g" => 0x47,
        "h" => 0x48,
        "i" => 0x49,
        "j" => 0x4A,
        "k" => 0x4B,
        "l" => 0x4C,
        "m" => 0x4D,
        "n" => 0x4E,
        "o" => 0x4F,
        "p" => 0x50,
        "q" => 0x51,
        "r" => 0x52,
        "s" => 0x53,
        "t" => 0x54,
        "u" => 0x55,
        "v" => 0x56,
        "w" => 0x57,
        "x" => 0x58,
        "y" => 0x59,
        "z" => 0x5A,
        "multiply" => VK_MULTIPLY,
        "kp_multiply" => VK_MULTIPLY,
        "add" | "kp_add" => VK_ADD,
        "minus" => VK_OEM_MINUS,
        "kp_subtract" => VK_SUBTRACT,
        "kp_divide" => VK_DIVIDE,
        "comma" => VK_OEM_COMMA,
        "period" => VK_OEM_PERIOD,
        "kanji" => VK_KANJI,
        _ => return None
    })
}