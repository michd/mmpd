use crate::config::raw_config::{RawConfig, AccessHelpers};
use crate::macros::actions::{ControlAction, Action};
use crate::config::ConfigError;

pub fn build_action_control(raw_data: Option<&RawConfig>) -> Result<Action, ConfigError> {
    const FIELD_ACTION: &str = "action";

    const ACTION_RELOAD_MACROS: &str = "reload_macros";
    const ACTION_RESTART: &str = "restart";
    const ACTION_EXIT: &str = "exit";

    let raw_data = raw_data.ok_or_else(|| {
        ConfigError::InvalidConfig(
            format!("Action control: missing data field")
        )
    })?;

    let action_str = match raw_data {
        RawConfig::String(raw_action) => Ok(raw_action.as_str()),

        RawConfig::Hash(hash) => Ok(hash.get_string(FIELD_ACTION).ok_or_else(|| {
                ConfigError::InvalidConfig(
                    format!(
                        "Action control: data field doesn't contain a string '{}' field",
                        FIELD_ACTION
                    )
                )
            })?),

        _ => Err(ConfigError::InvalidConfig(
            "Action control: data field should be either string or hash, but was neither"
                .to_string()
            ))
    }?;

    Ok(Action::Control(match action_str {
        ACTION_RELOAD_MACROS => ControlAction::ReloadMacros,
        ACTION_RESTART => ControlAction::Restart,
        ACTION_EXIT => ControlAction::Exit,

        _ => {
            return Err(ConfigError::InvalidConfig(
                format!(
                    "Action control: Unknown control action '{}'",
                    action_str
                )
            ));
        }
    }))
}

#[cfg(test)]
mod tests {
    use crate::config::raw_config::{RawConfig, k, RCHash};
    use crate::config::versions::version1::actions::control::build_action_control;
    use crate::macros::actions::{ControlAction, Action};

    #[test]
    fn builds_reload_macros_action_from_string() {
        let data = k("reload_macros");

        let action = build_action_control(Some(&data))
            .ok().unwrap();

        assert_eq!(action, Action::Control(ControlAction::ReloadMacros));
    }

    #[test]
    fn builds_reload_macros_action_from_hash() {
        let mut data = RCHash::new();
        data.insert(k("action"), k("reload_macros"));

        let action = build_action_control(Some(&RawConfig::Hash(data)))
            .ok().unwrap();

        assert_eq!(action, Action::Control(ControlAction::ReloadMacros));
    }

    #[test]
    fn builds_restart_action_from_string() {
        let data = k("restart");

        let action = build_action_control(Some(&data))
            .ok().unwrap();

        assert_eq!(action, Action::Control(ControlAction::Restart));
    }

    #[test]
    fn builds_restart_action_from_hash() {
        let mut data = RCHash::new();
        data.insert(k("action"), k("restart"));

        let action = build_action_control(Some(&RawConfig::Hash(data)))
            .ok().unwrap();

        assert_eq!(action, Action::Control(ControlAction::Restart));
    }

    #[test]
    fn builds_exit_action_from_string() {
        let data = k("exit");

        let action = build_action_control(Some(&data))
            .ok().unwrap();

        assert_eq!(action, Action::Control(ControlAction::Exit));
    }

    #[test]
    fn builds_exit_action_from_hash() {
        let mut data = RCHash::new();
        data.insert(k("action"), k("exit"));

        let action = build_action_control(Some(&RawConfig::Hash(data)))
            .ok().unwrap();

        assert_eq!(action, Action::Control(ControlAction::Exit));
    }

    #[test]
    fn returns_error_if_data_is_none() {
        let action = build_action_control(None);
        assert!(action.is_err());
    }

    #[test]
    fn returns_error_if_data_is_neither_string_nor_hash() {
        let action = build_action_control(Some(&RawConfig::Null));
        assert!(action.is_err());

        let action = build_action_control(Some(&RawConfig::Bool(true)));
        assert!(action.is_err());

        let action = build_action_control(Some(&RawConfig::Integer(0)));
        assert!(action.is_err());

        let action = build_action_control(Some(&RawConfig::Array(vec![])));
        assert!(action.is_err());
    }

    #[test]
    fn returns_error_if_string_data_is_invalid_action() {
        let action = build_action_control(Some(&RawConfig::String("InvalidAction".to_string())));
        assert!(action.is_err());
    }

    #[test]
    fn returns_error_if_data_is_hash_but_lacks_action_string_field() {
        let data = RCHash::new();
        let action = build_action_control(Some(&RawConfig::Hash(data)));
        assert!(action.is_err());

        let mut data = RCHash::new();
        data.insert(k("action"), RawConfig::Null);
        let action = build_action_control(Some(&RawConfig::Hash(data)));
        assert!(action.is_err());
    }
}