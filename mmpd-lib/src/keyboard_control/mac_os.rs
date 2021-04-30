use crate::keyboard_control::{KeyboardControlAdapter, KeyboardResult, KeyboardControlError};
use std::process::Command;

pub fn get_adapter() -> Option<Box<impl KeyboardControlAdapter>> {
    MacOs::new().map(|mac_os| Box::new(mac_os))
}

struct MacOs {}

impl MacOs {
    fn new() -> Option<impl KeyboardControlAdapter> {
        Some(MacOs {})
    }
}

impl KeyboardControlAdapter for MacOs {
    // Note: this implementation leaves much to be desired.
    // 1) It's incredibly slow, especially when running multiple calls of this in a row;
    //    each one is a full new invocation of osascript. This could be much improved if
    //    multiple keysequence actions were combined into one, in some way, so that the Mac OS
    //    implementation could generate and run a single script to run all of them.
    // 2) It would be nicer if we could use proper Apple SDK (like CoreGraphics events) calls
    //    instead of doing it in the hacky AppleScript way. Unfortunately, existing binding
    //    projects don't appear to offer adequate coverage yet, and contributing to that scale
    //    of rust access is a bit beyond the scope of this project for me (MichD); I'm just
    //    implementing support for Mac OS, an OS I have no intention of spending any appreciable
    //    time using, for the sake of having this project be properly multi-platform.
    //
    // Anyway, some ideas, that would take this outside of this module: We could pre-process
    // a list of actions found in a macro upon loading config, and repackage a series of
    // key sequence actions into a MacOS-only "AppleScript" wrapper action to make this a lot
    // more efficient. The AppleScript action does not need to be accessible from the config file
    // directly though; best to keep it internal.
    fn send_keysequence(&self, sequence: &str, _delay_microsecs: u32) -> KeyboardResult {
        // See https://eastmanreference.com/complete-list-of-applescript-key-codes
        // And https://eastmanreference.com/how-to-automate-your-keyboard-in-mac-os-x-with-applescript
        let sequence_script = build_sequence_script(sequence)?;

        Command::new("osascript")
            .arg("-e")
            .arg(format!("tell application \"System Events\" to {}", sequence_script))
            .status()
            .map(|_| ())
            .map_err(|err| KeyboardControlError::Other(err.to_string()))
    }

    fn send_text(&self, text: &str, _delay_microsecs: u32) -> KeyboardResult {
        // Note: the text input is passed as an argument instead of concatenated into the script
        // to allow proper handling of the data, thereby ensuring there's no opportunity for
        // the context of text to become arbitrary apple script that can be executed.
        // To run arbitrary code, Action::Shell should be used instead.

        let script = r#"
            on run argv
                tell application "System Events"
                    keystroke item 1 of argv
                end tell
            end run
        "#;

        Command::new("osascript")
            .arg("-e")
            .arg(script)
            .arg(text)
            .status()
            .map(|_| ())
            .map_err(|err| KeyboardControlError::Other(err.to_string()))
    }
}

/// Builds the line of AppleScript that will execute a key combination
fn build_sequence_script(str_sequence: &str) -> Result<String, KeyboardControlError> {
    let str_sequence = str_sequence.trim();

    if str_sequence.is_empty() {
        return Err(KeyboardControlError::Other("No keys specified".to_string()));
    }

    // Split on `+` and trim each found key
    let keys= str_sequence.split('+')
        .into_iter()
        .map(|key| key.trim().to_string())
        .collect::<Vec<String>>();

    // Different numbers of keys call for different AppleScript syntax
    Ok(match keys.len() {
        // This seems very unlikely to occur given the is_empty check above
        0 => return Err(KeyboardControlError::Other("No keys specified".to_string())),

        // Unwrap is safe since length was checked
        // "keystroke <key>" or "key code <key code>"
        1 => build_main_key_subcommand(keys.first().unwrap())?,

        // Unwrap is again safe since length was checked
        // "keystroke <key> using <other key> down"
        2 => format!(
            "{} using {} down",
            build_main_key_subcommand(keys.last().unwrap())?,
            keys.first().unwrap()
        ),

        // > 2
        // "keystroke <key> using {<other key 1> down, <other key 2> down[...]}"
        _ => {
            let held_keys = &keys[0..keys.len() - 1]
                .iter()
                .rev()
                .map(|s| format!("{} down", s))
                .collect::<Vec<String>>()
                .join(", ")
                .to_string();

            format!(
                "{} using {{{}}}",
                build_main_key_subcommand(keys.last().unwrap())?,
                held_keys
            )
        }
    })
}

/// Builds the first part of a keystroke AppleScript command from a given key string
/// This function determines whether the command should use `key code` or `keystroke`, and
/// translates accordingly. Can return KeyBoardControlError if an invalid key is specified.
fn build_main_key_subcommand(key: &str) -> Result<String, KeyboardControlError> {
    // Key values that need to be provided to `keystroke` as a quoted string
    const QUOTED_KEYSTROKE_INPUTS: &'static [&str] = &[
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "l", "m", "n", "o", "p", "q", "r", "s",
        "t", "u", "v", "w", "x", "y", "z", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "¬",
        "`", "!", r#"""#, "£", "$", "%", "^", "&", "*", "(", ")", "-", "_", "+", "=", "[", "]", "{",
        "}", ":", ";", "@", "'", "#", "~", r#"\"#, "|", ",", ".", "<", ">", "/", "?",
    ];

    // Key values that may be provided via `keystroke` without quotes
    const VALID_OTHER_KEYS: &'static [&str] = &[
        "control", "option", "shift", "command", "tab", "space"
    ];

    let key = key.to_lowercase();

    // Check if the input key is one in the list that needs quoting for `keystroke`
    if QUOTED_KEYSTROKE_INPUTS.contains(&key.as_str()) {
        let key = match key.as_str() {
            // Escape madness, woo
            r#"""# => r#"\""#, // If the character is ", escape it.
            r#"\"# => r#"\\"#, // If the character is \, escape it.
            _ => key.as_str() // If none of those special cases, use it as-is
        };

        return Ok(format!("keystroke \"{}\"", key))
    }

    // If the key didn't match one of the values that need quoting, check if it matches a value
    // that needs to be represented as a `key code`, and grab the corresponding key code.
    let key_code = match key.as_str() {
        "enter" => Some(76),
        "return" => Some(36),
        "esc" => Some(53),
        "left" => Some(123),
        "right" => Some(124),
        "up" => Some(126),
        "down" => Some(125),
        "home" => Some(115),
        "end" => Some(119),
        "page_up" => Some(116),
        "page_down" => Some(121),
        "f1" => Some(122),
        "f2" => Some(120),
        "f3" => Some(99),
        "f4" => Some(118),
        "f5" => Some(96),
        "f6" => Some(97),
        "f7" => Some(98),
        "f8" => Some(100),
        "f9" => Some(101),
        "f10" => Some(109),
        "f11" => Some(103),
        "f12" => Some(111),
        "f13" => Some(105),
        "f14" => Some(107),
        "f15" => Some(113),
        "f16" => Some(106),
        "f17" => Some(64),
        "f18" => Some(79),
        "f19" => Some(80),
        "f20" => Some(90),
        _ => None
    };

    // If the key matched one of the key codes in the above list, build the command with the result
    if let Some(code) = key_code {
        return Ok(format!("key code {}", code));
    }

    // Lastly, if none of that matched yet, see if the key is a valid key that can be used
    // unquoted, otherwise, return an error
    if VALID_OTHER_KEYS.contains(&key.as_str()) {
        Ok(format!("keystroke {}", key))
    } else {
        Err(KeyboardControlError::InvalidKey(key))
    }
}

#[cfg(test)]
mod build_main_key_subcommand_tests {
    use crate::keyboard_control::mac_os::build_main_key_subcommand;

    #[test]
    fn builds_keystrokes_that_need_quoting() {
        assert_eq!(
            r#"keystroke "a""#,
            build_main_key_subcommand("a").unwrap()
        );
    }

    #[test]
    fn ignores_case_for_keystrokes_that_need_quoting() {
        assert_eq!(
            r#"keystroke "a""#,
            build_main_key_subcommand("A").unwrap()
        )
    }

    #[test]
    fn builds_keystrokes_that_need_keycode() {
        assert_eq!(
            "key code 116",
            build_main_key_subcommand("page_up").unwrap()
        );
    }

    #[test]
    fn ignores_case_for_keystrokes_that_need_keycode() {
        assert_eq!(
            "key code 116",
            build_main_key_subcommand("Page_Up").unwrap()
        );
    }

    #[test]
    fn correctly_escapes_double_quote_and_backslash() {
        assert_eq!(
            r#"keystroke "\"""#,
            build_main_key_subcommand(r#"""#).unwrap()
        );

        assert_eq!(
            r#"keystroke "\\""#,
            build_main_key_subcommand(r#"\"#).unwrap()
        );
    }

    #[test]
    fn rejects_invalid_multi_character_keys() {
        assert!(build_main_key_subcommand("notakey").is_err());
    }

    #[test]
    fn accepts_valid_multi_character_keys() {
        assert_eq!(
            "keystroke command",
            build_main_key_subcommand("command").unwrap()
        );
    }

    #[test]
    fn ignores_case_in_valid_multi_character_keys() {
        assert_eq!(
            "keystroke command",
            build_main_key_subcommand("Command").unwrap()
        );
    }
}

#[cfg(test)]
mod build_sequence_scrip_tests {
    use crate::keyboard_control::mac_os::build_sequence_script;

    #[test]
    fn fails_on_blank_string() {
        assert!(build_sequence_script("").is_err());
        assert!(build_sequence_script(" ").is_err());
    }

    #[test]
    fn forms_single_key_sequence_command() {
        assert_eq!(
            r#"keystroke "a""#.to_string(),
            build_sequence_script("a").unwrap()
        );
    }

    #[test]
    fn forms_double_key_sequence_command() {
        assert_eq!(
            r#"keystroke "b" using a down"#.to_string(),
            build_sequence_script("a+b").unwrap()
        );
    }

    #[test]
    fn forms_triple_key_sequence_command() {
        assert_eq!(
            r#"keystroke "c" using {b down, a down}"#.to_string(),
            build_sequence_script("a+b+c").unwrap()
        );
    }

    #[test]
    fn forms_quadruple_key_sequence_command() {
        assert_eq!(
            r#"keystroke "d" using {c down, b down, a down}"#.to_string(),
            build_sequence_script("a+b+c+d").unwrap()
        );
    }
}