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

/// Access point for config file format version 1 parsing
pub (crate) struct Version1Processor {
    // Ideas:
    // - Ability to specify how strict to be: If there's any error in trying to parse a
    //   data structure, continue by discarding it as null, perhaps adding a warning into
    //   a list kept in this struct. Otherwise (if strict mode) fail entire config load for
    //   any incorrect/missing data encountered
}

impl Version1Processor {
    /// Provides an instance of Version1Processor presented as "an implementation of
    /// `ConfigProcessor`"
    pub (crate) fn new() -> Box<dyn ConfigProcessor> {
        Box::new(Version1Processor {})
    }
}

impl ConfigProcessor for Version1Processor {
    /// Processes a top level RCHash into a fully formed Config instances, or returns a ConfigError
    /// if something doesn't work out correctly.
    ///
    /// ## Notes on the version 1 format
    ///
    /// At the top level, there are 3 possible expected fields:
    /// - `scopes`:
    ///     Contains window class/name matching, as well as a list of macros that apply to that
    ///     scope. Note that in the parsed Config struct, this is organised differently; there is
    ///     one list of macros, each of which may or may not come with a scope. In the program it
    ///     is more practical that way, but in the context of authoring a configuration file, it
    ///     makes sense to specify a series of macros that apply to a given scope.
    /// - `global_macros`:
    ///     Contains all macros that apply regardless of focused window: macros without a scope.
    ///
    /// Further documentation and examples on the format can be found in /docs/config.md
    ///
    /// ## Arguments
    /// raw_config: Top level hash parsed from the config input file
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
                    let scope = build_scope(raw_scope)?;
                    if let None = scope { continue; }

                    let raw_macros = raw_scope.get_array(MACROS_FIELD);
                    if let None = raw_macros { continue; }

                    config.macros.extend(
                        build_scope_macros(
                            raw_macros.unwrap(),
                            Some(scope.unwrap())
                        )?
                    );
                }
            }
        }

        if let Some(raw_macros) = raw_config.get_array(GLOBAL_MACROS_FIELD) {
            config.macros.extend(build_scope_macros(raw_macros, None)?);
        }

        Ok(config)
    }
}

/// From a list of `RawConfig`s (expected to be `RawConfig::Hash`, otherwise skipped over), returns
/// a list of `Macro` instances, attaching a copy of the provided `Scope`, if any.
/// If any of the parsing into a macros goes wrong down the line, returns a `ConfigError` instead.
fn build_scope_macros(
    raw_macros: &Vec<RawConfig>,
    scope: Option<Scope>
) -> Result<Vec<Macro>, ConfigError> {
    // Note: a bit of magic happens here.
    // in the `map(...)` bit an iterator of Results is built.
    // The calling collect will either collect a Ok(Vec(all the Ok values)),
    // or return Err(first error encountered), thereby satisfying this function's signature
    // If I understand correctly, it will also not iterate beyond the first encountered error.
    raw_macros
        .iter()
        .filter_map(|raw_macro| {
            // Filter out any values that aren't hashes
            if let RawConfig::Hash(hash) = raw_macro {
                Some(hash)
            } else {
                None
            }
        })
        .map(|raw_macro| {
            // Build an Ok(macro) or Err(ConfigError) for each item
            Ok(build_macro(raw_macro, scope.clone())?)
        })
        .collect()
}

/// From a given `raw_scope` `RCHash`, parses its fields to construct a `Scope`; a set of
/// `StringMatcher`s to match against a window class and/or field. If the relevant fields aren't
/// found or don't contain relevant string matching fields, returns `None`.
///
/// Expects the `raw_scope` hash to be structured as follows:
///
/// ```yml
/// window_class:
///     # (string matcher)
/// window_name:
///     # (string matcher)
/// macros:
///     # list of macros
/// ```
/// Both `window_class` and `window_name` are optional. If both are specified, then for the scope
/// to match, the string matchers for both must matched the focused window's fields.
///
/// The expected structure of string matcher is described by `build_string_matcher`.
///
/// The `macros` field is not actually used in the `build_scope` function, but is shown for
/// completeness.
///
/// ## Errors
/// This function will return `ConfigError` if constructing a `StringMatcher` fails for any reason.
fn build_scope(raw_scope: &RCHash) -> Result<Option<Scope>, ConfigError> {
    const WINDOW_CLASS_FIELD: &str = "window_class";
    const WINDOW_NAME_FIELD: &str = "window_name";

    let window_class_matcher = build_string_matcher(
        raw_scope.get_hash(WINDOW_CLASS_FIELD)
    )?;

    let window_name_matcher = build_string_matcher(
        raw_scope.get_hash(WINDOW_NAME_FIELD)
    )?;

    let has_any_matchers = vec![&window_class_matcher, &window_name_matcher]
        .iter()
        .any(|matcher| matcher.is_some());

    Ok(if has_any_matchers {
        Some(
            Scope::new(window_class_matcher, window_name_matcher)
        )
    } else {
        None
    })
}

/// For a given `raw_matcher` `RCHash`, constructs a `StringMatcher`, a type that can match against
/// a string in several different ways.
///
/// For practicality, returns `None` if `raw_matcher` is None.
/// If anything goes wrong in parsing the contents of `raw_matcher`, returns a ConfigError.
///
/// Expects the structure of `raw_matcher` to be one of the following:
///
/// - String should equal - maps to `StringMatcher::Is`
///   ```yaml
///   is: "string-to-match"
///   ```
/// - String should contain - maps to `StringMatcher::Contains`
///   ```yaml
///   contains: "string-to-be-contained"
///   ```
/// - String should start with - maps to `StringMatcher::StartsWith`
///   ```yaml
///   starts_with: "string-it-should-start-with"
///   ```
/// - String should end with - maps to `StringMatcher::EndsWith`
///    ```yaml
///    ends_with: "string-it-should-end-with"
///    ```
/// - String should match regular expression - maps to `String::Regex`
///   ```yaml
///   regex: "string-should-match-this-regular-expression"
///   ```
///
/// The function will return `None` in the following cases:
///
///   - `raw_matcher` is None
///   - `raw_matcher` is empty
///   - The last found key in `raw_matcher` is none of the listed ones
///   - The last found value mapped to a relevant key is not a `RawConfig::String`
///
/// The function will return `ConfigError` if the field found is "regex", but the value specified
/// fails to parse as a regular expression pattern.
fn build_string_matcher(
    raw_matcher: Option<&RCHash>
) -> Result<Option<StringMatcher>, ConfigError> {
    if let None = raw_matcher { return Ok(None); }
    let raw_matcher = raw_matcher.unwrap();
    let last_field = raw_matcher.iter().last();

    Ok(if let Some((RawConfig::String(key), RawConfig::String(value))) = last_field {
        const TYPE_IS: &str = "is";
        const TYPE_CONTAINS: &str = "contains";
        const TYPE_STARTS_WITH: &str = "starts_with";
        const TYPE_ENDS_WITH: &str = "ends_with";
        const TYPE_REGEX: &str = "regex";

        match key.to_lowercase().as_ref() {
            TYPE_IS => Some(StringMatcher::Is(String::from(value))),
            TYPE_CONTAINS => Some(StringMatcher::Contains(String::from(value))),
            TYPE_STARTS_WITH => Some(StringMatcher::StartsWith(String::from(value))),
            TYPE_ENDS_WITH => Some(StringMatcher::EndsWith(String::from(value))),
            TYPE_REGEX => Some(StringMatcher::Regex(
                Regex::new(value).map_err(|err| {
                    ConfigError::InvalidConfig(
                        format!("String matcher: invalid regex. {}", err.to_string())
                    )
                })?
            )),

            _ => None
        }
    } else {
        None
    })
}

/// Constructs a `Macro` with an optional Scope from a raw `RCHash`.
///
/// Macros are expected to have the following structure:
///
/// ```yaml
/// name: "An optional name identifying this macro"
///
/// matching_events:
///     - # (one or more matching events)
///
/// required_preconditions:
///     - # (zero or more preconditions)
///
/// actions:
///     - # (one or more actions to be executed)
/// ```
///
/// `name` is optional. This name is only used for showing what is happening, but holds
/// not operational significance otherwise.
///
/// `matching_events` is required, and must be a list of "event matchers". The structure
/// of an event matcher is detailed in `build_event_matcher`. There must be at least one specified.
/// To run a macro, at least one of its event matchers must match the incoming event.
///
/// `required_preconditions` is optional. If specified, must be a list of "preconditions". The
/// structure of a preconditions is detailed in `build_precondition`. A precondition is a separate
/// condition that must be satisfied in order to run a macro. If more than one precondition is
/// specified, all of them must be satisfied to run the macro.
///
/// `actions` is required. It must be a list of actions to be run when the macro is run. If more
/// than one action is specified then they will all run in the order they are specified when the
/// macro is run. The structure of an action is specified in `build_action`.
///
/// ## Errors
/// This function will return `ConfigError` in any of these conditions:
///
/// - The field `matching_events` is missing, is not a `RawConfig::Array`, or contains no items
/// - The field `actions` is missing, is not a `RawConfig::Array`, or contains no items
/// - Down the stream, an error occurs while trying to build one of the event matchers, actions, or
///   preconditions
fn build_macro(raw_macro: &RCHash, scope: Option<Scope>) -> Result<Macro, ConfigError> {
    const NAME_FIELD: &str = "name";
    const MATCHING_EVENTS_FIELD: &str = "matching_events";
    const REQUIRED_PRECONDITIONS_FIELD: &str = "required_preconditions";
    const ACTIONS_FIELD: &str = "actions";

    let raw_matching_events = raw_macro.get_array(MATCHING_EVENTS_FIELD).map_or_else(|| {
        Err(ConfigError::InvalidConfig(
            format!("Macro definition missing {} list", MATCHING_EVENTS_FIELD)
        ))
    },|raw_events| {
        if raw_events.is_empty() {
            Err(ConfigError::InvalidConfig(format!(
                "Macro definition contains '{}' field, but not a single matching event is specified",
                MATCHING_EVENTS_FIELD
            )))
        } else {
            Ok(raw_events)
        }
    })?;

    let raw_actions = raw_macro.get_array(ACTIONS_FIELD).map_or_else(|| {
        Err(ConfigError::InvalidConfig(
            format!("Macro definition missing {} list", ACTIONS_FIELD)
        ))
    }, |r_actions| {
        if r_actions.is_empty() {
            Err(ConfigError::InvalidConfig(format!(
                "Macro definition contains '{}' field, but not single action is specified",
                ACTIONS_FIELD
            )))
        } else {
            Ok(r_actions)
        }
    })?;

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

/// Constructs an `EventMatcher` instance (in a `Box`) from a Raw `raw_event_matcher`
/// `RCHash`'s fields.
///
/// Event matchers are expected to follow this structure:
///
/// ```yaml
/// type: "event type goes here"
///
/// data:
///     # (any fields relevant to the event type to be matched)
///
/// required_preconditions:
///     - # (Optional: any preconditions that only have to apply for this event)
/// ```
///
/// `type` is required. Its value must be one of the implemented event types. Currently, these are:
///     - midi
///
/// `data` is not meant to be a hash, but is not strictly required. Depending on the event type, it
/// may be required, but this function does not enforce it.
///
/// `required_preconditions` is optional. If specified, must be a list of "preconditions". The
/// structure of a preconditions is detailed in `build_precondition`. A precondition is a separate
/// condition that must be satisfied in order for an event to match with this event matcher.
/// If more than one precondition is specified, all of them must be satisfied for the event matcher
/// to match.
///
/// ## Errors
/// This function will return `ConfigError` under any of these conditions:
///
/// - `type` field is missing or is not a `RawCondition::String`
/// - The value for the `type` field does not match any known event matcher types; see above
/// - Down the stream, a more specific event matcher (such as `MidiEventMatcher`) fails to be
///   constructed for any reason
/// - Down the stream, a `Precondition` fails to be constructed for any reason
fn build_event_matcher(raw_event_matcher: &RCHash) -> Result<Box<EventMatcher>, ConfigError> {
    const TYPE_FIELD: &str = "type";
    const DATA_FIELD: &str = "data";
    const REQUIRED_PRECONDITIONS_FIELD: &str = "required_preconditions";
    const TYPE_MIDI: &str = "midi";

    let event_type = raw_event_matcher.get_string(TYPE_FIELD).ok_or_else(|| {
        ConfigError::InvalidConfig(
            format!("event missing valid (string) '{}' field", TYPE_FIELD)
        )
    })?;

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

/// Constructs a `Precondition` from a `_raw_precondition` `RCHash`.
///
/// Since preconditions aren't implemented yet (beyond a stub), this always returns a blank
/// `Precondition` instance regardless of the contents of `_raw_precondition`.
fn build_precondition(_raw_precondition: &RCHash) -> Result<Precondition, ConfigError> {
    Ok(Precondition::new())
}

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
fn build_action(raw_action: &RCHash) -> Result<Action, ConfigError> {
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
fn build_action_key_sequence(raw_data: Option<&RawConfig>) -> Result<Action, ConfigError> {
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
fn build_action_enter_text(raw_data: Option<&RawConfig>) -> Result<Action, ConfigError> {
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
fn build_action_shell(raw_data: Option<&RawConfig>) -> Result<Action, ConfigError> {
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

/// Constructs a `MidiEventMatcher` (returned as a `Box<dyn MatchChecker<MidiMessage>>`) from a
/// `data` `RCHash`.
///
/// `data` should be structured as follows:
/// ```yaml
/// message_type: note_on
/// channel: (number matcher)
/// key: (number matcher)
/// velocity: (number matcher)
/// ```
///
/// This is just one example, there are different valid properties, depending on the value of
/// `message_type`. All the expected values for the non-`message_type` fields is a number matcher.
/// see `build_number_matcher` for details of what a number matcher entails.
///
/// Here's an exhaustive list of additional available message_types and the additional fields that
/// are available for them
///
/// - `note_on` - A key is pressed down
///   - `channel` - Which MIDI channel (0-15)
///   - `key` - Which key (0-127)
///   - `velocity` - How fast the key was pressed down (0-127)
/// - `note_off` - A key is released
///   - `channel`
///   - `key`
///   - `velocity` - How fast the key was released (0-127)
/// - `poly_aftertouch` - Pressure on an already held key changes (not widely available)
///     - `channel`
///     - `key`
///     - `value` - Level of pressure on the key
/// - `control_change` - A parameter was changed (like a knob, slider, ...)
///     - `channel`
///     - `control` - Control number (0-127)
///     - `value` - New value of the control (0-127)
/// - `program_change` - The currently selected program/patch changed
///     - `channel`
///     - `program` - New program value (0-127)
/// - `channel_aftertouch` - Pressure on already held key changes (but not key-specific)
///     - `channel`
///     - `value` - New level of pressure on whatever is held down
/// - `pitch_bend_change` - Position of the pitch bender changes
///     - `channel`
///     - `value` - New pitch bend position (0-16383)
///
/// ## Errors
/// The function returns `ConfigError` in any of the following conditions:
///
/// - No `data` is specified
/// - No `message_type` string field is part of `data`
/// - `message_type` value is not one of the supported values
/// - Downstream there is an issue constructing a number matcher for any reason
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

    let data = data.ok_or_else(|| {
       ConfigError::InvalidConfig(format!(
           "Missing data for midi event matcher"
       ))
    })?;

    let message_type = data.get_string(MESSAGE_TYPE_FIELD).ok_or_else(|| {
        ConfigError::InvalidConfig(format!(
            "Missing {} field in midi event data",
            MESSAGE_TYPE_FIELD
        ))
    })?;

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

/// Constructs a number matcher from a `matcher` `RawConfig`.
///
/// There are several valid forms of a number matcher, and they can be nested.
///
/// - `Val`
///   Constructed from a plain number:
///   ```yaml
///   5
///   ```
///   Will match when the number checked against it is, as you guessed, 5
///
/// - `Range`
///   Constructed from a hash containing one or both of `min` and `max` fields:
///   ```yaml
///   min: 3
///   max: 7
///   ```
///   This will match all numbers in 3,4,5,6,7.
///   Omitting `min` but having `max` will match all numbers up to and including
///   the `max` value; omitting `max` but having `min` will match all number above and
///   including the `min` value.
///
/// - `List`
///   Constructed from a list containing plain numbers and/or range hashes.
///   ```yaml
///   - 3
///   - 5
///   - min: 7
///     max: 10
///   ```
///   The above example will match the numbers 3, 5, 7, 8, 9, and 10.
///
/// If `None` is given, returns `None` (which means it will match any number).
///
/// ## Errors
/// The function will return `ConfigError` in any of the following conditions:
///
/// - The value for `min` is less than 0
/// - The value for `max` is less than 0
/// - The value for `max` is less than the value for `min`
///
/// All other cases of invalid data types and what have you return None rather
/// than an error.
fn build_number_matcher(matcher: Option<&RawConfig>) -> Result<Option<NumberMatcher>, ConfigError> {
    const MIN_FIELD: &str = "min";
    const MAX_FIELD: &str = "max";

    if let Some(matcher) = matcher {
        Ok(match matcher {
            RawConfig::Integer(i) => {
                if *i >= 0 {
                    Some(NumberMatcher::Val(*i as u32))
                } else {
                    None
                }
            },

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
                                "Number range matcher supports only positive integers, got \
                                {} for {}",
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
            },

            _ => None
        })
    } else {
        Ok(None)
    }
}
