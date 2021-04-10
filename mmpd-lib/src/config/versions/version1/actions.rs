mod key_sequence;
mod enter_text;
mod shell;
mod wait;
mod control;

use crate::config::raw_config::{RCHash, AccessHelpers, k};
use crate::macros::actions::Action;
use crate::config::ConfigError;
use key_sequence::build_action_key_sequence;
use enter_text::build_action_enter_text;
use shell::build_action_shell;
use wait::build_action_wait;
use crate::config::versions::version1::actions::control::build_action_control;

/// Constructs an `Action` from a `raw_action` `RCHash`.
///
/// Actions are expected to follow this structure:
///
/// ```yaml
/// type: "action type goes here"
///
/// data:
///     # (any fields relevant to the action type here)
/// ```
///
/// `type` is required, and must be one of the implemented action types. Currently, these are:
///     - `key_sequence` (see `build_action_key_sequence`)
///     - `enter_text` (see `build_action_enter_text`)
///     - `shell` (see `build_action_shell`)
///
/// `data` is not strictly required, nor are their restrictions on what type of data it should
/// represent. Most often it will be a hash to specify one or more fields, but depending on the
/// action, it's possibly to simplify to it the single value that action needs. Specifics are up
/// to the build_action_* method relevant to the action type.
///
/// ## Errors
/// This function will return `ConfigError in any of these conditions:
///
/// - The `type` field is missing
/// - The `type` value doesn't match any of the implemented values
/// - Down the stream, we fail to build the specific Action value from given data for any reason
pub fn build_action(raw_action: &RCHash) -> Result<Action, ConfigError> {
    const TYPE_FIELD: &str = "type";
    const DATA_FIELD: &str = "data";

    const KEY_SEQUENCE_TYPE: &str = "key_sequence";
    const ENTER_TEXT_TYPE: &str = "enter_text";
    const SHELL_TYPE: &str = "shell";
    const WAIT_TYPE: &str = "wait";
    const CONTROL_TYPE: &str = "control";

    let data_hash = raw_action.get(&k(DATA_FIELD));

    let action_type = raw_action.get_string(TYPE_FIELD).ok_or_else(|| {
        ConfigError::InvalidConfig(
            format!("Missing '{}' field for action", TYPE_FIELD)
        )
    })?;

    Ok(match action_type {
        KEY_SEQUENCE_TYPE => build_action_key_sequence(data_hash)?,
        ENTER_TEXT_TYPE => build_action_enter_text(data_hash)?,
        SHELL_TYPE => build_action_shell(data_hash)?,
        WAIT_TYPE => build_action_wait(data_hash)?,
        CONTROL_TYPE => build_action_control(data_hash)?,

        _ => {
            return Err(ConfigError::InvalidConfig(
                format!("Unknown action type '{}'", action_type)
            ));
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::config::raw_config::{RCHash, k, RawConfig};
    use crate::config::versions::version1::actions::build_action;
    use crate::macros::actions::{Action, ControlAction};

    #[test]
    fn returns_an_error_if_type_field_is_missing() {
        let mut hash = RCHash::new();
        hash.insert(k("data"), k("Hello world"));

        let action = build_action(&hash);
        assert!(action.is_err());
    }

    #[test]
    fn returns_an_error_if_type_field_has_unknown_value() {
        let mut hash = RCHash::new();
        hash.insert(k("type"), k("not-a-real-action"));
        hash.insert(k("data"), k("Hello world"));

        let action = build_action(&hash);
        assert!(action.is_err());
    }

    #[test]
    fn builds_an_enter_text_action() {
        let mut hash = RCHash::new();
        hash.insert(k("type"), k("enter_text"));
        // For more complicated versions of making enter_text actions, see the tests for
        // build_enter_text_action
        hash.insert(k("data"), k("Hello world"));

        let action = build_action(&hash).ok().unwrap();

        assert_eq!(
            action,
            Action::enter_text("Hello world")
        );
    }

    #[test]
    fn builds_a_key_sequence_action() {
        let mut hash = RCHash::new();
        hash.insert(k("type"), k("key_sequence"));
        // For more complicated versions of making key_sequence actions, see the tests for
        // build_key_sequence_action
        hash.insert(k("data"), k("ctrl+shift+t"));

        let action = build_action(&hash).ok().unwrap();

        assert_eq!(
            action,
            Action::key_sequence("ctrl+shift+t")
        );
    }

    #[test]
    fn builds_a_shell_action() {
        let mut hash = RCHash::new();
        hash.insert(k("type"), k("shell"));
        // For more complicated versions of making shell actions, see the tests for
        // build_shell_action
        hash.insert(k("data"), k("cmd"));

        let action = build_action(&hash).ok().unwrap();

        assert_eq!(
            action,
            Action::Shell {
                command: "cmd".to_string(),
                args: None,
                env_vars: None,
            }
        );
    }

    #[test]
    fn builds_a_wait_action() {
        let mut hash = RCHash::new();
        hash.insert(k("type"), k("wait"));
        hash.insert(k("data"), RawConfig::Integer(20));

        let action = build_action(&hash).ok().unwrap();

        assert_eq!(
            action,
            Action::Wait { duration: 20 }
        )
    }

    #[test]
    fn builds_a_control_action() {
        let mut hash = RCHash::new();
        hash.insert(k("type"), k("control"));
        hash.insert(k("data"), k("exit"));

        let action = build_action(&hash).ok().unwrap();

        assert_eq!(
            action,
            Action::Control(ControlAction::Exit)
        );
    }
}
