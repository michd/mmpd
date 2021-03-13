use crate::config::raw_config::{RawConfig, AccessHelpers};
use crate::macros::actions::Action;
use crate::config::ConfigError;

/// Constructs an `Action::KeySequence` from `raw_data` `RawConfig`.
///
/// There are two permissible forms for `raw_data` to construct an `Action::KeySequence`:
///
/// - `RawConfig::String`: specify the key sequence directly
/// - `RawConfig::Hash`: specify more info, as follows:
///   ```yaml
///   sequence: "sequence goes here"
///   count: 2
///   ```
///
///   `sequence` is required and should be a String, like "ctrl+shift+t"
///
///   `count` is optional and should be a positive integer; this is how many times the key sequence
///   is to be repeated. It also defaults to 1 if anything that isn't an integer is given
///
/// When specified as a `RawConfig::String`, or omitted in a `RawConfig::Hash`, `count` will default
/// to 1.
///
/// ## Errors
/// The function return `ConfigError` in any of the following circumstances:
///
/// - `raw_data` is None
/// - `raw_data` is neither `RawConfig::String` nor `RawConfig::Hash`
/// - `raw_data` is a `RawConfig::Hash` but is missing a `sequence` field that is a
///   `RawConfig::String`
/// - `raw_data` is a `RawConfig::Hash` but `count` is a negative integer
pub fn build_action_key_sequence(raw_data: Option<&RawConfig>) -> Result<Action, ConfigError> {
    const SEQUENCE_FIELD: &str = "sequence";
    const COUNT_FIELD: &str = "count";

    let raw_data = raw_data.ok_or_else(|| {
        ConfigError::InvalidConfig(
            format!("Action key_sequence: missing data field")
        )
    })?;

    match raw_data {
        RawConfig::String(sequence) => Ok(Action::KeySequence(sequence.to_string(), 1)),

        RawConfig::Hash(hash) => {
            let sequence = hash.get_string(SEQUENCE_FIELD).ok_or_else(|| {
                ConfigError::InvalidConfig(format!(
                    "Action key_sequence: data field doesn't contain a '{}' field",
                    SEQUENCE_FIELD
                ))
            })?;

            let count = hash.get_integer(COUNT_FIELD).unwrap_or(1);

            if count < 0 {
                Err(ConfigError::InvalidConfig(
                    format!("Action key_sequence: count should be 0 or more, found {}", count)
                ))
            } else {
                Ok(Action::KeySequence(sequence.to_string(), count as usize))
            }
        }

        _ => Err(ConfigError::InvalidConfig(format!(
            "Action key_sequence: data field should be either string or hash, but was neither"
        )))
    }
}

#[cfg(test)]
mod tests {
    use crate::config::raw_config::{RawConfig, RCHash, k};
    use crate::macros::actions::Action;
    use crate::config::versions::version1::actions::key_sequence::build_action_key_sequence;

    #[test]
    fn returns_an_error_if_no_data_is_provided() {
        let action = build_action_key_sequence(None);
        assert!(action.is_err());
    }

    #[test]
    fn builds_key_sequence_action_from_simplified_form() {
        let action = build_action_key_sequence(
            Some(&RawConfig::String("ctrl+shift+t".to_string()))
        ).ok().unwrap();

        assert_eq!(action, Action::KeySequence("ctrl+shift+t".to_string(), 1));
    }

    #[test]
    fn builds_key_sequence_action_from_hash_with_count() {
        let mut data_hash = RCHash::new();
        data_hash.insert(k("sequence"), k("ctrl+shift+t"));
        data_hash.insert(k("count"), RawConfig::Integer(3));

        let action = build_action_key_sequence(Some(&RawConfig::Hash(data_hash)))
            .ok().unwrap();

        assert_eq!(action, Action::KeySequence("ctrl+shift+t".to_string(), 3));
    }

    #[test]
    fn builds_key_sequence_action_from_hash_without_count() {
        let mut data_hash = RCHash::new();
        data_hash.insert(k("sequence"), k("ctrl+shift+t"));

        let action = build_action_key_sequence(Some(&RawConfig::Hash(data_hash)))
            .ok().unwrap();

        assert_eq!(action, Action::KeySequence("ctrl+shift+t".to_string(), 1));
    }

    #[test]
    fn returns_an_error_if_count_is_negative() {
        let mut data_hash = RCHash::new();
        data_hash.insert(k("text"), k("Hello world"));
        data_hash.insert(k("count"), RawConfig::Integer(-5));

        let action = build_action_key_sequence(Some(&RawConfig::Hash(data_hash)));

        assert!(action.is_err());
    }

    #[test]
    fn returns_an_error_if_data_is_neither_hash_or_string() {
        let action = build_action_key_sequence(Some(&RawConfig::Null));
        assert!(action.is_err());
    }
}
