use crate::keyboard_control::{KeyboardControlAdapter, KeyboardResult, KeyboardControlError};

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
    fn send_keysequence(&self, _sequence: &str, _delay_microsecs: u32) -> KeyboardResult {
        // See https://eastmanreference.com/complete-list-of-applescript-key-codes
        // And https://eastmanreference.com/how-to-automate-your-keyboard-in-mac-os-x-with-applescript
        println!("Todo: send_keysequence in MacOs");
        Ok(())
    }

    fn send_text(&self, _text: &str, _delay_microsecs: u32) -> KeyboardResult {
        println!("Todo: send_text in MacOs");
        Ok(())
    }
}

fn build_sequence_script(str_sequence: &str) -> Result<String, KeyboardControlError> {
    let str_sequence = str_sequence.trim();

    if str_sequence.is_empty() {
        return Err(KeyboardControlError::Other("No keys specified".to_string()));
    }

    let keys= str_sequence.split('+')
        .into_iter()
        .map(|key| key.trim().to_string())
        .collect::<Vec<String>>();

    const CMD_PREFIX: &str = "keystroke";

    Ok(match keys.len() {
        0 => return Err(KeyboardControlError::Other("No keys specified".to_string())),

        // Unwrap is safe since length was checked
        1 => format!("{} {}", CMD_PREFIX, keys.first().unwrap()),

        // Unwrap is again safe since length was checked
        2 => format!(
            "{} {} using {} down",
            CMD_PREFIX,
            keys.last().unwrap(),
            keys.first().unwrap()
        ),

        // > 2
        _ => {
            let held_keys = &keys[0..keys.len() - 1]
                .iter()
                .rev()
                .map(|s| format!("{} down", s))
                .collect::<Vec<String>>()
                .join(", ").to_string();

            format!(
                "{} {} using {{{}}}",
                CMD_PREFIX,
                keys.last().unwrap(),
                held_keys
            )
        }
    })
}

#[cfg(test)]
mod build_sequence_scrip_tests {
    use crate::keyboard_control::mac_os::build_sequence_script;

    #[test]
    fn fails_on_blank_string() {
        let input = " ";

        let output = build_sequence_script(input);

        assert!(output.is_err());
    }

    #[test]
    fn forms_single_key_sequence_command() {
        let input = "a";

        let output = build_sequence_script(input).unwrap();

        assert_eq!("keystroke a".to_string(), output);
    }

    #[test]
    fn forms_double_key_sequence_command() {
        let input: &str = "a+b";

        let output = build_sequence_script(input).unwrap();

        assert_eq!("keystroke b using a down".to_string(), output);
    }

    #[test]
    fn forms_triple_key_sequence_command() {
        let input: &str = "a+b+c";

        let output = build_sequence_script(input).unwrap();

        assert_eq!("keystroke c using {b down, a down}".to_string(), output);
    }

    #[test]
    fn forms_quadruple_key_sequence_command() {
        let input: &str = "aa+bb+cc+dd";

        let output = build_sequence_script(input).unwrap();

        assert_eq!("keystroke dd using {cc down, bb down, aa down}".to_string(), output);
    }
}