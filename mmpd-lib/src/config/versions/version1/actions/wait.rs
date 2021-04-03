use crate::config::raw_config::{RawConfig, AccessHelpers};
use crate::config::ConfigError;
use crate::macros::actions::Action;

/// Constructs an `Action::Wait` from `raw_data` `RawConfig`.
///
/// There are two permissible forms for `raw_data` to construct an `Action::Wait`:
///
/// - `RawConfig::Integer`: specify the wait duration directly, in microseconds
/// - `RawConfig::Hash`: specify it as an object, as follows:
///   ```yaml
///   duration: 3000 # Duration in microseconds
///   duration_ms: 3 # Alternatively, duration in milliseconds
///   ```
///
///   `duration` should be a positive integer; this is how long the action should wait, expressed in
///   microseconds.
///   If both `duration` and `duration_ms` are specified, the value for `duration` is used, unless
///   `duration` contains an invalid (negative) value.
///
///   `duration_ms` should be a positive integer; this is how long the action should wait, expressed
///   in milliseconds.
///   If both `duration` and `duration_ms` are specified, the value for `duration` is used, unless
///   `duration` contains an invalid (negative) value.
///
/// ## Errors
/// The function returns a `ConfigError` in any of the following circumstances:
///
/// - `raw_data` is `None`
/// - `raw_data` is neither `RawConfig::Integer` nor `RawConfig::Hash`
/// - `raw_data` is `RawConfig::Integer` but its value is negative
/// - `raw_data` is `RawConfig::Hash` but neither `duration` or `duration_ms` fields are present,
///   or neither of their values are positive integers
pub fn build_action_wait(raw_data: Option<&RawConfig>) -> Result<Action, ConfigError> {
    const DURATION_FIELD: &str = "duration";
    const DURATION_MS_FIELD: &str = "duration_ms";

    let raw_data = raw_data.ok_or_else(|| {
        ConfigError::InvalidConfig(
            format!("Action wait: missing data field")
        )
    })?;

    Ok(match raw_data {
        RawConfig::Integer(i) => {
            if *i < 0 {
                return Err(ConfigError::InvalidConfig(
                    format!("Action wait: duration should be 0 or more, found {}", *i)
                ));
            } else {
                Action::Wait { duration: *i as u64 }
            }
        }

        RawConfig::Hash(hash) => {
            let duration = hash.get_integer(DURATION_FIELD).map(|d| {
                if d < 0 {
                    None
                } else {
                    Some(d as u64)
                }
            }).flatten()
            .or_else(|| {
                hash.get_integer(DURATION_MS_FIELD).map(|d| {
                    if d < 0 {
                        None
                    } else {
                        Some((d as u64) * 1000)
                    }
                }).flatten()
            });

            if duration.is_none() {
                return Err(ConfigError::InvalidConfig(
                    format!("Action wait: no positive duration or duration_ms specified")
                ));
            }

            Action::Wait { duration: duration.unwrap() }
        }

        _ => return Err(ConfigError::InvalidConfig(
            format!("Action wait: data field should be either int or hash, but was neither")
        ))
    })
}

#[cfg(test)]
mod tests {
    use crate::config::versions::version1::actions::wait::build_action_wait;
    use crate::config::raw_config::{RawConfig, RCHash, k};
    use crate::macros::actions::Action;

    #[test]
    fn returns_an_error_if_no_data_is_provided() {
        let action = build_action_wait(None);
        assert!(action.is_err());
    }

    #[test]
    fn builds_action_from_simplified_form() {
        let action = build_action_wait(Some(&RawConfig::Integer(42)))
            .ok().unwrap();

        assert_eq!(action, Action::Wait { duration: 42 });
    }

    #[test]
    fn returns_an_error_if_simplified_form_uses_negative_integer() {
        let action = build_action_wait(Some(&RawConfig::Integer(-5)));
        assert!(action.is_err());
    }

    #[test]
    fn returns_an_error_if_neither_duration_or_duration_ms_are_specified() {
        let action = build_action_wait(Some(&RawConfig::Hash(RCHash::new())));
        assert!(action.is_err());
    }

    #[test]
    fn builds_action_from_duration_field() {
        let mut data_hash = RCHash::new();
        data_hash.insert(k("duration"), RawConfig::Integer(20));
        let action = build_action_wait(Some(&RawConfig::Hash(data_hash)))
            .ok().unwrap();

        assert_eq!(action, Action::Wait { duration: 20 });
    }

    #[test]
    fn builds_action_from_duration_ms_field() {
        let mut data_hash = RCHash::new();
        data_hash.insert(k("duration_ms"), RawConfig::Integer(20));
        let action = build_action_wait(Some(&RawConfig::Hash(data_hash)))
            .ok().unwrap();

        assert_eq!(action, Action::Wait { duration: 20_000 });
    }

    #[test]
    fn returns_error_if_duration_field_is_negative() {
        let mut data_hash = RCHash::new();
        data_hash.insert(k("duration"), RawConfig::Integer(-20));
        let action = build_action_wait(Some(&RawConfig::Hash(data_hash)));

        assert!(action.is_err())
    }

    #[test]
    fn returns_error_if_duration_ms_field_is_negative() {
        let mut data_hash = RCHash::new();
        data_hash.insert(k("duration_ms"), RawConfig::Integer(-20));
        let action = build_action_wait(Some(&RawConfig::Hash(data_hash)));

        assert!(action.is_err())
    }

    #[test]
    fn duration_field_supersedes_duration_ms_field() {
        let mut data_hash = RCHash::new();
        data_hash.insert(k("duration_ms"), RawConfig::Integer(20));
        data_hash.insert(k("duration"), RawConfig::Integer(40));
        let action = build_action_wait(Some(&RawConfig::Hash(data_hash)))
            .ok().unwrap();

        assert_eq!(action, Action::Wait { duration: 40 });
    }

    #[test]
    fn uses_valid_field_if_alternative_is_negative() {
        let mut data_hash = RCHash::new();
        data_hash.insert(k("duration_ms"), RawConfig::Integer(20));
        // Negative = invalid, so "duration should be ignored" where it would normally be used
        // instead of "duration_ms" when both are specified
        data_hash.insert(k("duration"), RawConfig::Integer(-40));
        let action = build_action_wait(Some(&RawConfig::Hash(data_hash)))
            .ok().unwrap();

        assert_eq!(action, Action::Wait { duration: 20_000 });
    }

    #[test]
    fn returns_error_if_both_duration_and_duration_ms_are_negative() {
        let mut data_hash = RCHash::new();
        data_hash.insert(k("duration_ms"), RawConfig::Integer(-20));
        data_hash.insert(k("duration"), RawConfig::Integer(-40));
        let action = build_action_wait(Some(&RawConfig::Hash(data_hash)));

        assert!(action.is_err());
    }
}