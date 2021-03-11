use crate::config::raw_config::{ConfigProcessor, RawConfig, RCHash, AccessHelpers, k};
use crate::config::{ConfigError, Config};
use crate::macros::{Scope, Macro, MacroBuilder};
use crate::match_checker::{StringMatcher, MatchChecker, NumberMatcher};
use regex::Regex;
use crate::macros::event_matching::{EventMatcher, MatcherType};
use crate::macros::preconditions::Precondition;
use crate::macros::actions::Action;
use crate::midi::MidiMessage;
use crate::macros::event_matching::midi::MidiEventMatcher;

pub (crate) struct Version1Processor {
    // Ideas:
    // - Ability to specify how strict to be: If there's any error in trying to parse a
    //   data structure, continue by discarding it as null, perhaps adding a warning into
    //   a list kept in this struct. Otherwise (if strict mode) fail entire config load for
    //   any incorrect/missing data encountered
}

impl Version1Processor {
    pub (crate) fn new() -> Box<dyn ConfigProcessor> {
        Box::new(Version1Processor {})
    }
}

impl ConfigProcessor for Version1Processor {
    fn process<'a>(&self, raw_config: RCHash) -> Result<Config, ConfigError> {
        const SCOPES_FIELD: &str = "scopes";
        const MACROS_FIELD: &str = "macros";
        const GLOBAL_MACROS_FIELD: &str = "global_macros";

        let mut config = Config {
            macros: vec![]
        };

        if let Some(raw_scopes) = raw_config.get_array(SCOPES_FIELD) {
            for raw_scope in raw_scopes {
                if let RawConfig::Hash(raw_scope) = raw_scope {
                    if let Some(scope) = build_scope(raw_scope)? {

                        let scope_copy = scope.clone();

                        if let Some(raw_macros) = raw_scope.get_array(MACROS_FIELD) {
                            config.macros.extend(
                                build_scope_macros(raw_macros, Some(scope_copy))?
                            );
                        }
                    }
                } else {
                    continue;
                }
            }
        }

        if let Some(raw_macros) = raw_config.get_array(GLOBAL_MACROS_FIELD) {
            config.macros.extend(build_scope_macros(raw_macros, None)?);
        }

        Ok(config)
    }
}

fn build_scope_macros<'a>(
    raw_macros: &Vec<RawConfig>,
    scope: Option<Scope>
) -> Result<Vec<Macro>, ConfigError> {
    let mut macros: Vec<Macro> = vec![];

    for raw_macro in raw_macros {
        let scope_copy = scope.clone();

        if let RawConfig::Hash(raw_macro) = raw_macro {
            macros.push(build_macro(raw_macro, scope_copy)?);
        } else {
            continue
        }
    }

    Ok(macros)
}

fn build_scope<'a>(raw_scope: &RCHash) -> Result<Option<Scope>, ConfigError> {
    const WINDOW_CLASS_FIELD: &str = "window_class";
    const WINDOW_NAME_FIELD: &str = "window_name";

    let window_class_matcher = build_string_matcher(
        raw_scope.get_hash(WINDOW_CLASS_FIELD)
    )?;

    let window_name_matcher = build_string_matcher(
        raw_scope.get_hash(WINDOW_NAME_FIELD)
    )?;

    Ok(Some(Scope::new(window_class_matcher, window_name_matcher)))
}

fn build_string_matcher<'a>(raw_matcher: Option<&RCHash>) -> Result<Option<StringMatcher>, ConfigError> {
    if let None = raw_matcher {
        return Ok(None);
    }

    let raw_matcher = raw_matcher.unwrap();

    let last_field = raw_matcher.iter().last();

    if let None = last_field {
        return Ok(None);
    }

    if let Some((RawConfig::String(key), RawConfig::String(value))) = last_field {
        const TYPE_IS: &str = "is";
        const TYPE_CONTAINS: &str = "contains";
        const TYPE_STARTS_WITH: &str = "starts_with";
        const TYPE_ENDS_WITH: &str = "ends_with";
        const TYPE_REGEX: &str = "regex";

        Ok(match key.to_lowercase().as_ref() {
            TYPE_IS => Some(StringMatcher::Is(String::from(value))),
            TYPE_CONTAINS => Some(StringMatcher::Contains(String::from(value))),
            TYPE_STARTS_WITH => Some(StringMatcher::StartsWith(String::from(value))),
            TYPE_ENDS_WITH => Some(StringMatcher::EndsWith(String::from(value))),
            TYPE_REGEX => {
                let regex = Regex::new(value);

                if let Err(regex_err) = regex {
                    return Err(ConfigError::InvalidConfig(
                        format!(
                            "String matcher: invalid regex. {}", regex_err.to_string()
                        )
                    ))
                }

                Some(StringMatcher::Regex(regex.unwrap()))
            },
            _ => None
        })
    } else {
        Ok(None)
    }
}

fn build_macro<'a>(raw_macro: &RCHash, scope: Option<Scope>) -> Result<Macro, ConfigError> {
    const NAME_FIELD: &str = "name";
    const MATCHING_EVENTS_FIELD: &str = "matching_events";
    const REQUIRED_PRECONDITIONS_FIELD: &str = "required_preconditions";
    const ACTIONS_FIELD: &str = "actions";

    let raw_matching_events = raw_macro.get_array(MATCHING_EVENTS_FIELD);

    if let None = raw_matching_events {
        return Err(ConfigError::InvalidConfig(
            format!("Macro definition missing {} list", MATCHING_EVENTS_FIELD)
        ));
    }

    let raw_matching_events = raw_matching_events.unwrap();

    let raw_actions = raw_macro.get_array(ACTIONS_FIELD);

    if let None = raw_actions {
        return Err(ConfigError::InvalidConfig(
               format!("Macro definition missing {} list", ACTIONS_FIELD)
        ));
    }

    let raw_actions = raw_actions.unwrap();

    let raw_preconditions = raw_macro.get_array(REQUIRED_PRECONDITIONS_FIELD);

    let mut matching_events: Vec<Box<EventMatcher>> = vec![];
    let mut required_preconditions: Vec<Precondition> = vec![];
    let mut actions: Vec<Action> = vec![];

    for raw_event_matcher in raw_matching_events {

        if let RawConfig::Hash(raw_event_matcher) = raw_event_matcher {
            matching_events.push(build_event_matcher(raw_event_matcher)?);
        }
    }

    for raw_action in raw_actions {
        if let RawConfig::Hash(raw_action) = raw_action {
            actions.push(build_action(raw_action)?);
        }
    }

    if let Some(raw_preconditions) = raw_preconditions {
        for raw_preconditions in raw_preconditions {
            if let RawConfig::Hash(raw_preconditions) = raw_preconditions {
                required_preconditions.push(build_precondition(raw_preconditions)?);
            }
        }
    }

    let name = raw_macro.get_string(NAME_FIELD);

    let mut macro_builder = MacroBuilder::from_event_matchers(matching_events);

    if let Some(name) = name {
        macro_builder = macro_builder.set_name(name.to_string());
    }

    if !required_preconditions.is_empty() {
        macro_builder = macro_builder.set_preconditions(required_preconditions);
    }

    if !actions.is_empty() {
        macro_builder = macro_builder.set_actions(actions);
    }

    if let Some(scope) = scope {
        macro_builder = macro_builder.set_scope(scope);
    }

    Ok(macro_builder.build())
}

fn build_event_matcher(raw_event_matcher: &RCHash) -> Result<Box<EventMatcher>, ConfigError> {
    const TYPE_FIELD: &str = "type";
    const DATA_FIELD: &str = "data";
    const REQUIRED_PRECONDITIONS_FIELD: &str = "required_preconditions";
    const TYPE_MIDI: &str = "midi";

    let event_type = raw_event_matcher.get_string(TYPE_FIELD);

    if let None = event_type {
        return Err(ConfigError::InvalidConfig(
            format!("event missing valid (string) '{}' field", TYPE_FIELD)
        ))
    }

    let event_type = event_type.unwrap();

    let raw_preconditions = raw_event_matcher.get_array(REQUIRED_PRECONDITIONS_FIELD);
    let mut preconditions: Vec<Precondition> = vec![];

    if let Some(raw_preconditions) = raw_preconditions {
        for precondition_hash in raw_preconditions {
            if let RawConfig::Hash(precondition_hash) = precondition_hash {
                preconditions.push(build_precondition(precondition_hash)?);
            }
        }
    }

    let data = raw_event_matcher.get_hash(DATA_FIELD);

    let matcher_type: MatcherType = match event_type {
        TYPE_MIDI => MatcherType::Midi(build_midi_event_matcher(data)?),
        _ => {
            return Err(ConfigError::InvalidConfig(
                format!("Unknown event matcher type '{}'", event_type)
            ));
        }
    };

    Ok(Box::new(
        EventMatcher::new(
            matcher_type,
            if preconditions.is_empty() { None } else { Some(preconditions) }
        )
    ))
}

fn build_precondition(_raw_precondition: &RCHash) -> Result<Precondition, ConfigError> {
    Ok(Precondition::new())
}

fn build_action(raw_action: &RCHash) -> Result<Action, ConfigError> {
    const TYPE_FIELD: &str = "type";
    const DATA_FIELD: &str = "data";

    const KEY_SEQUENCE_TYPE: &str = "key_sequence";
    const ENTER_TEXT_TYPE: &str = "enter_text";
    const SHELL_TYPE: &str = "shell";

    let data_hash = raw_action.get(&k(DATA_FIELD));

    let action_type = raw_action.get_string(TYPE_FIELD);

    if let None = action_type {
        return Err(ConfigError::InvalidConfig(
            format!("Missing '{}' field for action", TYPE_FIELD)
        ));
    }

    let action_type = action_type.unwrap();

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

fn build_action_key_sequence(raw_data: Option<&RawConfig>) -> Result<Action, ConfigError> {
    const SEQUENCE_FIELD: &str = "sequence";
    const COUNT_FIELD: &str = "count";

    if let None = raw_data {
        return Err(ConfigError::InvalidConfig(
           format!("Action key_sequence: missing data field")
        ));
    }

    let raw_data = raw_data.unwrap();

    Ok(match raw_data {
        RawConfig::String(sequence) => {
            Action::KeySequence(sequence.to_string(), 1)
        }

        RawConfig::Hash(hash) => {
            let sequence = hash.get_string(SEQUENCE_FIELD);

            if let None = sequence {
                return Err(ConfigError::InvalidConfig(
                    format!(
                        "Action key_sequence: data field doesn't contain a sequence field"
                    )
                ));
            }

            let sequence = sequence.unwrap();

            let count = hash.get_integer(COUNT_FIELD);

            if let Some(count) = count {
                if count < 0 {
                    return Err(ConfigError::InvalidConfig(
                        format!("Action key_sequence: count should be 0 or more, found {}", count)
                    ));
                }

                Action::KeySequence(sequence.to_string(), count as usize)
            } else {
                Action::KeySequence(sequence.to_string(), 1)
            }
        }

        _ => {
            return Err(ConfigError::InvalidConfig(
                format!(
                    "Action key_sequence: data field should be either string or hash, \
                     but was neither")
            ));
        }
    })
}

fn build_action_enter_text(raw_data: Option<&RawConfig>) -> Result<Action, ConfigError> {
    const TEXT_FIELD: &str = "text";
    const COUNT_FIELD: &str = "count";

    if let None = raw_data {
        return Err(ConfigError::InvalidConfig(
            format!("Action enter_text: missing data field")
        ));
    }

    let raw_data = raw_data.unwrap();

    Ok(match raw_data {
        RawConfig::String(text) => {
            Action::EnterText(text.to_string(), 1)
        }

        RawConfig::Hash(hash) => {
            let text = hash.get_string(TEXT_FIELD);

            if let None = text {
                return Err(ConfigError::InvalidConfig(
                    format!(
                        "Action enter_text: data field doesn't contain a text field"
                    )
                ));
            }

            let text = text.unwrap();

            let count = hash.get_integer(COUNT_FIELD);

            if let Some(count) = count {
                if count < 0 {
                    return Err(ConfigError::InvalidConfig(
                        format!("Action enter_text: count should be 0 or more, found {}", count)
                    ));
                }

                Action::EnterText(text.to_string(), count as usize)
            } else {
                Action::EnterText(text.to_string(), 1)
            }
        }

        _ => {
            return Err(ConfigError::InvalidConfig(
                format!(
                    "Action enter_text: data field should be either string or hash, \
                     but was neither"
                )
            ));
        }
    })
}

fn build_action_shell(raw_data: Option<&RawConfig>) -> Result<Action, ConfigError> {
    const COMMAND_FIELD: &str = "command";
    const ARGS_FIELD: &str = "args";
    const ENV_VARS_FIELD: &str = "env_vars";

    if let None = raw_data {
        return Err(ConfigError::InvalidConfig(
            format!("Action: shell: missing data field")
        ));
    }

    let raw_data = raw_data.unwrap();

    Ok(match raw_data {
        RawConfig::String(command) => {
            Action::Shell { command: command.to_string(), args: None, env_vars: None }
        }

        RawConfig::Hash(hash) => {
            let command = hash.get_string(COMMAND_FIELD);

            if let None = command {
                return Err(ConfigError::InvalidConfig(
                    format!(
                        "Action: shell: data field doesn't contain a command field"
                    )
                ));
            }

            let command = command.unwrap();

            let args = hash.get_array(ARGS_FIELD);

            let args = args.map_or(Ok(None), |raw_args| {
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
                    let mut out_args: Vec<String> = vec![];

                    for a in raw_args {
                        match a {
                            RawConfig::Integer(i) => out_args.push(i.to_string()),
                            RawConfig::String(s) => out_args.push(s.to_string()),
                            _ => {}
                        }
                    }

                    Ok(if out_args.is_empty() {
                        None
                    }   else {
                        Some(out_args)
                    })
                }
            })?;

            let env_vars = hash.get_hash(ENV_VARS_FIELD);

            let env_vars = env_vars.map_or(Ok(None), |raw_env_vars| {
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
                    let mut out_env_vars: Vec<(String, String)> = vec![];

                    for (k, v) in raw_env_vars {
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

                        if let (Some(k), Some(v)) = (k, v) {
                            out_env_vars.push((k, v))
                        }
                    }

                    Ok(if out_env_vars.is_empty() {
                        None
                    } else {
                        Some(out_env_vars)
                    })
                }
            })?;

            Action::Shell { command: command.to_string(), args, env_vars }
        }

        _ => {
            return Err(ConfigError::InvalidConfig(
               format!(
                   "Action: shell: data field should be either string or hash, \
                    but was neither"
               )
            ));
        }
    })
}

fn build_midi_event_matcher(
    data: Option<&RCHash>
) -> Result<Box<dyn MatchChecker<MidiMessage>>, ConfigError> {

    const MESSAGE_TYPE_FIELD: &str = "message_type";
    const CHANNEL_FIELD: &str = "channel";
    const KEY_FIELD: &str = "key";
    const VELOCITY_FIELD: &str = "velocity";
    const VALUE_FIELD: &str = "value";
    const CONTROL_FIELD: &str = "control";
    const PROGRAM_FIELD: &str = "program";

    const NOTE_ON_EVENT: &str = "note_on";
    const NOTE_OFF_EVENT: &str = "note_off";
    const POLY_AFTERTOUCH_EVENT: &str = "poly_aftertouch";
    const CONTROL_CHANGE_EVENT: &str = "control_change";
    const PROGRAM_CHANGE_EVENT: &str = "program_change";
    const CHANNEL_AFTERTOUCH_EVENT: &str = "channel_aftertouch";
    const PITCH_BEND_CHANGE_EVENT: &str = "pitch_bend_change";

    if let None = data {
        return Err(ConfigError::InvalidConfig(
            format!("Missing data for midi event matcher")
        ));
    }

    let data = data.unwrap();

    let message_type = data.get_string(MESSAGE_TYPE_FIELD);

    if let None = message_type {
        return Err(ConfigError::InvalidConfig(
            format!("Missing {} field in midi event data", MESSAGE_TYPE_FIELD)
        ));
    }

    let message_type = message_type.unwrap();

    let raw_channel_matcher = data.get(&k(CHANNEL_FIELD));
    let channel_match = build_number_matcher(raw_channel_matcher)?;

    Ok(Box::new(match message_type {
        NOTE_ON_EVENT => {
            let raw_key_matcher = data.get(&k(KEY_FIELD));
            let raw_velocity_matcher = data.get(&k(VELOCITY_FIELD));

            MidiEventMatcher::NoteOn {
                channel_match,
                key_match: build_number_matcher(raw_key_matcher)?,
                velocity_match: build_number_matcher(raw_velocity_matcher)?
            }
        }

        NOTE_OFF_EVENT => {
            let raw_key_matcher = data.get(&k(KEY_FIELD));
            let raw_velocity_matcher = data.get(&k(VELOCITY_FIELD));

            MidiEventMatcher::NoteOff {
                channel_match,
                key_match: build_number_matcher(raw_key_matcher)?,
                velocity_match: build_number_matcher(raw_velocity_matcher)?
            }
        }

        POLY_AFTERTOUCH_EVENT => {
            let raw_key_matcher = data.get(&k(KEY_FIELD));
            let raw_value_matcher = data.get(&k(VALUE_FIELD));

            MidiEventMatcher::PolyAftertouch {
                channel_match,
                key_match: build_number_matcher(raw_key_matcher)?,
                value_match: build_number_matcher(raw_value_matcher)?
            }
        }

        CONTROL_CHANGE_EVENT => {
            let raw_control_matcher = data.get(&k(CONTROL_FIELD));
            let raw_value_matcher = data.get(&k(VALUE_FIELD));

            MidiEventMatcher::ControlChange {
                channel_match,
                control_match: build_number_matcher(raw_control_matcher)?,
                value_match: build_number_matcher(raw_value_matcher)?
            }
        }

        PROGRAM_CHANGE_EVENT => {
            let raw_program_matcher = data.get(&k(PROGRAM_FIELD));

            MidiEventMatcher::ProgramChange {
                channel_match,
                program_match: build_number_matcher(raw_program_matcher)?
            }
        }

        CHANNEL_AFTERTOUCH_EVENT => {
            let raw_value_matcher = data.get(&k(VALUE_FIELD));

            MidiEventMatcher::ChannelAftertouch {
                channel_match,
                value_match: build_number_matcher(raw_value_matcher)?
            }
        }

        PITCH_BEND_CHANGE_EVENT => {
            let raw_value_matcher = data.get(&k(VALUE_FIELD));

            MidiEventMatcher::PitchBendChange {
                channel_match,
                value_match: build_number_matcher(raw_value_matcher)?
            }
        }

        _ => {
            return Err(ConfigError::InvalidConfig(
                format!(
                    "Invalid or unsupported MIDI message type '{}'",
                    message_type
                )
            ))
        }
    }))
}

fn build_number_matcher(matcher: Option<&RawConfig>) -> Result<Option<NumberMatcher>, ConfigError> {
    const MIN_FIELD: &str = "min";
    const MAX_FIELD: &str = "max";

    if let Some(matcher) = matcher {
        Ok(match matcher {
            RawConfig::Null => None,

            RawConfig::Integer(i) => {
                if *i >= 0 {
                    Some(NumberMatcher::Val(*i as u32))
                } else {
                    None
                }
            },

            RawConfig::String(_) => None,

            RawConfig::Bool(_) => None,

            RawConfig::Array(arr) => {
                let mut matcher_list: Vec<NumberMatcher> = vec![];

                for raw_matcher in arr {
                    if let Some(parsed_matcher) = build_number_matcher(Some(raw_matcher))? {
                        matcher_list.push(parsed_matcher);
                    }
                }

                if matcher_list.is_empty() {
                    None
                } else {
                    Some(NumberMatcher::List(matcher_list))
                }
            }

            RawConfig::Hash(range) => {
                let raw_min_val = range.get_integer(MIN_FIELD);
                let raw_max_val = range.get_integer(MAX_FIELD);

                let mut min_val: Option<u32> = None;
                let mut max_val: Option<u32> = None;

                if let Some(raw_min_val) = raw_min_val {
                    if raw_min_val >= 0 {
                        min_val = Some(raw_min_val as u32);
                    } else {
                        return Err(ConfigError::InvalidConfig(
                            format!(
                                "Number range matcher supports only positive integers, \
                                 got {} for {}",
                                raw_min_val,
                                MIN_FIELD
                            )
                        ))
                    }
                }

                if let Some(raw_max_val) = raw_max_val {
                    if raw_max_val >= 0 {
                        max_val = Some(raw_max_val as u32);
                    } else {
                        return Err(ConfigError::InvalidConfig(
                            format!(
                                "Number range matcher supports only positive integers, got {} for {}",
                                raw_max_val,
                                MAX_FIELD
                            )
                        ))
                    }
                }

                if let (Some(min_val), Some(max_val)) = (min_val, max_val) {
                    if min_val > max_val {
                        return Err(ConfigError::InvalidConfig(
                            format!(
                                "Number range matcher will never match, since min ({}) > max ({})",
                                min_val,
                                max_val
                            )
                        ))
                    }
                }

                Some(NumberMatcher::Range { min: min_val, max: max_val })
            }
        })
    } else {
        Ok(None)
    }
}
