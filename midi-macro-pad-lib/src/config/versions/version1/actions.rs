mod key_sequence;
mod enter_text;
mod shell;

use crate::config::raw_config::{RCHash, AccessHelpers, k};
use crate::macros::actions::Action;
use crate::config::ConfigError;
use key_sequence::build_action_key_sequence;
use enter_text::build_action_enter_text;
use shell::build_action_shell;

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

        _ => {
            return Err(ConfigError::InvalidConfig(
                format!("Unknown action type '{}'", action_type)
            ));
        }
    })
}
