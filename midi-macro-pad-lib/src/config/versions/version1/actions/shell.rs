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

#[cfg(test)]
mod tests {
    use crate::config::versions::version1::actions::shell::build_action_shell;
    use crate::config::raw_config::{RawConfig, RCHash, k};
    use crate::macros::actions::Action;

    #[test]
    fn returns_error_if_no_data_is_provided() {
        let action = build_action_shell(None);
        assert!(action.is_err());
    }

    #[test]
    fn builds_shell_action_from_simple_form() {
        let action = build_action_shell(Some(&RawConfig::String("cmd".to_string())))
            .ok().unwrap();

        assert_eq!(
            action,
            Action::Shell { command: "cmd".to_string(), args: None, env_vars: None }
        );
    }

    #[test]
    fn returns_error_if_data_is_neither_string_nor_hash() {
        let action = build_action_shell(Some(&RawConfig::Null));
        assert!(action.is_err());
    }

    #[test]
    fn builds_shell_action_from_hash_with_only_command() {
        let mut hash = RCHash::new();
        hash.insert(k("command"), k("cmd"));

        let action = build_action_shell(Some(&RawConfig::Hash(hash)))
            .ok().unwrap();

        assert_eq!(
            action,
            Action::Shell { command: "cmd".to_string(), args: None, env_vars: None}
        );
    }

    #[test]
    fn returns_error_if_data_is_hash_and_command_is_missing() {
        let mut hash = RCHash::new();
        hash.insert(k("args"), RawConfig::Array(vec![
            RawConfig::String("arg1".to_string())
        ]));

        let action = build_action_shell(Some(&RawConfig::Hash(hash)));
        assert!(action.is_err());
    }

    #[test]
    fn builds_shell_action_with_string_and_int_args() {
        let args = RawConfig::Array(vec![
            RawConfig::String("arg1".to_string()),
            RawConfig::Integer(7),
            RawConfig::String("arg3".to_string()),
        ]);

        let mut hash = RCHash::new();
        hash.insert(k("command"), k("cmd"));
        hash.insert(k("args"), args);

        let action = build_action_shell(Some(&RawConfig::Hash(hash)))
            .ok().unwrap();

        assert_eq!(
            action,
            Action::Shell {
                command: "cmd".to_string(),
                args: Some(vec!["arg1".to_string(), 7.to_string(), "arg3".to_string()]),
                env_vars: None,
            }
        );
    }

    #[test]
    fn returns_error_if_args_contains_values_other_than_string_or_int() {
        let args = RawConfig::Array(vec![
            RawConfig::String("arg1".to_string()),
            RawConfig::Null,
        ]);

        let mut hash = RCHash::new();
        hash.insert(k("command"), k("cmd"));
        hash.insert(k("args"), args);

        let action = build_action_shell(Some(&RawConfig::Hash(hash)));
        assert!(action.is_err());
    }

    #[test]
    fn builds_shell_action_with_env_vars() {
        let mut env_vars_hash = RCHash::new();
        env_vars_hash.insert(k("env1"), k("val1"));
        env_vars_hash.insert(k("env2"), k("val2"));

        let mut hash = RCHash::new();
        hash.insert(k("command"), k("cmd"));
        hash.insert(k("env_vars"), RawConfig::Hash(env_vars_hash));

        let action = build_action_shell(Some(&RawConfig::Hash(hash)))
            .ok().unwrap();

        assert_eq!(
            action,
            Action::Shell {
                command: "cmd".to_string(),
                args: None,
                env_vars: Some(vec![
                    ("env1".to_string(), "val1".to_string()),
                    ("env2".to_string(), "val2".to_string())
                ])
            }
        );
    }

    #[test]
    fn returns_error_if_any_env_var_key_or_value_is_not_string_or_int() {
        let mut env_vars_hash_key_wrong = RCHash::new();
        env_vars_hash_key_wrong.insert(RawConfig::Bool(true), k("val1"));

        let mut hash_key_wrong = RCHash::new();
        hash_key_wrong.insert(k("command"), k("cmd"));
        hash_key_wrong.insert(k("env_vars"), RawConfig::Hash(env_vars_hash_key_wrong));

        let action_key_wrong = build_action_shell(Some(&RawConfig::Hash(hash_key_wrong)));

        assert!(action_key_wrong.is_err());

        let mut env_vars_hash_val_wrong = RCHash::new();
        env_vars_hash_val_wrong.insert(k("arg1"),RawConfig::Bool(true));

        let mut hash_val_wrong = RCHash::new();
        hash_val_wrong.insert(k("command"), k("cmd"));
        hash_val_wrong.insert(k("env_vars"), RawConfig::Hash(env_vars_hash_val_wrong));

        let action_val_wrong = build_action_shell(Some(&RawConfig::Hash(hash_val_wrong)));

        assert!(action_val_wrong.is_err());
    }

    #[test]
    fn builds_shell_action_with_args_and_env_vars() {
        let mut hash = RCHash::new();
        hash.insert(k("command"), k("cmd"));

        hash.insert(k("args"), RawConfig::Array(vec![
            RawConfig::String("arg1".to_string()),
            RawConfig::Integer(4)
        ]));

        let mut env_vars_hash = RCHash::new();
        env_vars_hash.insert(k("env1"), k("val1"));
        env_vars_hash.insert(k("env2"), RawConfig::Integer(7));

        hash.insert(k("env_vars"), RawConfig::Hash(env_vars_hash));

        let action = build_action_shell(Some(&RawConfig::Hash(hash)))
            .ok().unwrap();

        assert_eq!(
            action,
            Action::Shell {
                command: "cmd".to_string(),
                args: Some(vec!["arg1".to_string(), 4.to_string()]),
                env_vars: Some(vec![
                    ("env1".to_string(), "val1".to_string()),
                    ("env2".to_string(), 7.to_string())
                ])
            }
        );
    }
}
