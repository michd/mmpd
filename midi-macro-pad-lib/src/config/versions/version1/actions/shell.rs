use crate::config::raw_config::{RawConfig, AccessHelpers};
use crate::macros::actions::Action;
use crate::config::ConfigError;

/// Constructs an `Action:Shell` from `raw_data` `RawConfig`.
///
/// There are two permissible form for `raw_data` to construct an `Action::Shell`:
///
/// - `RawConfig::String`: Specify only a command without arguments or env vars, as a string
/// - `RawConfig::Hash`: specify more info, as follows:
///    ```yaml
///    command: "full path to a program to run"
///
///    args:
///      - # (one or more string or int arguments)
///
///    env_vars:
///      key1: value1
///      key2: value2
///      # (Any set of key/value combo of strings or ints)
///    ```
///
///    `command` is required,the full path to a program to run, as a string
///
///    `args` is optional, a list of arguments to specify, must all be strings or ints
///
///    `env_vars` is optional, a hash of string/int key / string/int value pairs
///
/// ## Errors
/// The function returns ConfigError under any of the following conditions:
///
/// - `raw_data` is `None`
/// - `raw_data` is neither `RawConfig::String` nor `RawConfig::Hash`
/// - `raw_data` is `RawConfig::Hash` but is missing a `RawConfig::String` `command` field
/// - Any of the items in `args` is neither `RawConfig::String` or `RawConfig::Int`
/// - Any of the keys or values in `env_vars` is neither `RawConfig::String` or `RawConfig::Int`
pub fn build_action_shell(raw_data: Option<&RawConfig>) -> Result<Action, ConfigError> {
    const COMMAND_FIELD: &str = "command";
    const ARGS_FIELD: &str = "args";
    const ENV_VARS_FIELD: &str = "env_vars";

    let raw_data = raw_data.ok_or_else(|| {
        ConfigError::InvalidConfig(
            format!("Action: shell: missing data field")
        )
    })?;

    match raw_data {
        RawConfig::String(command) => Ok(
            Action::Shell { command: command.to_string(), args: None, env_vars: None }
        ),

        RawConfig::Hash(hash) => {
            let command = hash.get_string(COMMAND_FIELD).ok_or_else(|| {
                ConfigError::InvalidConfig(format!(
                    "Action: shell: data field doesn't contain a command field"
                ))
            })?;

            let args= hash.get_array(ARGS_FIELD).map_or(
                Ok(None),
                |raw_args| {
                    if raw_args.iter().any(|a| {
                        match a {
                            RawConfig::String(_) => false,
                            RawConfig::Integer(_) => false,
                            _ => true
                        }
                    }) {
                        Err(ConfigError::InvalidConfig(
                            format!("Action: shell: invalid argument type")
                        ))
                    } else {
                        let out_args: Vec<String> = raw_args
                            .iter()
                            .filter_map(|a| {
                                match a {
                                    RawConfig::Integer(i) => Some(i.to_string()),
                                    RawConfig::String(s) => Some(s.to_string()),
                                    _ => None
                                }
                            })
                            .collect();

                        Ok(if out_args.is_empty() {
                            None
                        }   else {
                            Some(out_args)
                        })
                    }
                }
            )?;

            let env_vars = hash.get_hash(ENV_VARS_FIELD).map_or(
                Ok(None),
                |raw_env_vars| {
                    if raw_env_vars.iter().any(|(k, v)| {
                        let k_issue = match k {
                            RawConfig::Integer(_) => false,
                            RawConfig::String(_) => false,
                            _ => true
                        };

                        let v_issue = match v {
                            RawConfig::Integer(_) => false,
                            RawConfig::String(_) => false,
                            _ => true
                        };

                        k_issue || v_issue
                    }) {
                        Err(ConfigError::InvalidConfig(
                            format!("Action: shell: invalid env var key and/or value type")
                        ))
                    } else {
                        let out_env_vars: Vec<(String, String)> = raw_env_vars
                            .iter()
                            .filter_map(|(k, v)| {
                                let k = match k {
                                    RawConfig::Integer(i) => Some(i.to_string()),
                                    RawConfig::String(s) => Some(s.to_string()),
                                    _ => None
                                };

                                let v = match v {
                                    RawConfig::Integer(i) => Some(i.to_string()),
                                    RawConfig::String(s) => Some(s.to_string()),
                                    _ => None
                                };

                                if k.is_none() || v.is_none() {
                                    None
                                } else {
                                    Some((k.unwrap(), v.unwrap()))
                                }
                            })
                            .collect();

                        Ok(if out_env_vars.is_empty() {
                            None
                        } else {
                            Some(out_env_vars)
                        })
                    }
                }
            )?;

            Ok(Action::Shell { command: command.to_string(), args, env_vars })
        }

        _ => Err(ConfigError::InvalidConfig(format!(
            "Action: shell: data field should be either string or hash, but was neither"
        )))
    }
}
