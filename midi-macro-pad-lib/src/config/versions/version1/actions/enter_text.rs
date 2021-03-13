use crate::config::raw_config::{RawConfig, AccessHelpers};
use crate::macros::actions::Action;
use crate::config::ConfigError;

/// Constructs an `Action::EnterText` from `raw_data` `RawConfig`.
///
/// There are two permissible forms for `raw_data` to construct an `Action::EnterText`:
///
/// - `RawConfig::String`: specify the text directly
/// - `RawConfig::Hash`: specify more info, as follows:
///   ```yaml
///   text: "text to be typed"
///   count: 2
///   ```
///
///   `text` is required and should be a String, like "Hello world!"
///
///   `count` is optional and should be a positive integer; this is how many times the text
///   is to be repeated. It also defaults to 1 if anything that isn't an integer is given.
///
/// When specified as a `RawConfig::String`, or omitted in a `RawConfig::Hash`, `count` will default
/// to 1.
///
/// ## Errors
/// The function return `ConfigError` in any of the following circumstances:
///
/// - `raw_data` is None
/// - `raw_data` is neither `RawConfig::String` nor `RawConfig::Hash`
/// - `raw_data` is a `RawConfig::Hash` but is missing a `text` field that is a
///   `RawConfig::String`
/// - `raw_data` is a `RawConfig::Hash` but `count` is a negative integer
pub fn build_action_enter_text(raw_data: Option<&RawConfig>) -> Result<Action, ConfigError> {
    const TEXT_FIELD: &str = "text";
    const COUNT_FIELD: &str = "count";

    let raw_data = raw_data.ok_or_else(|| {
        ConfigError::InvalidConfig(
            format!("Action enter_text: missing data field")
        )
    })?;

    match raw_data {
        RawConfig::String(text) => Ok(Action::EnterText(text.to_string(), 1)),

        RawConfig::Hash(hash) => {
            let text = hash.get_string(TEXT_FIELD).ok_or_else(|| {
                ConfigError::InvalidConfig(format!(
                    "Action enter_text: data field doesn't contain a '{}' field",
                    TEXT_FIELD
                ))
            })?;

            let count = hash.get_integer(COUNT_FIELD).unwrap_or(1);

            if count < 0 {
                Err(ConfigError::InvalidConfig(
                    format!("Action enter_text: count should be 0 or more, found {}", count)
                ))
            } else {
                Ok(Action::EnterText(text.to_string(), count as usize))
            }
        }

        _ => Err(ConfigError::InvalidConfig(
            format!(
                "Action enter_text: data field should be either string or hash, but was neither"
            )
        ))
    }
}
