use crate::config::raw_config::{RCHash, AccessHelpers, RawConfig};
use crate::macros::{Scope, Macro, MacroBuilder};
use crate::config::ConfigError;
use crate::macros::event_matching::EventMatcher;
use crate::macros::preconditions::Precondition;
use crate::macros::actions::Action;
use crate::config::versions::version1::event_matchers::build_event_matcher;
use crate::config::versions::version1::precondition::build_precondition;
use crate::config::versions::version1::actions::build_action;

/// From a list of `RawConfig`s (expected to be `RawConfig::Hash`, otherwise skipped over), returns
/// a list of `Macro` instances, attaching a copy of the provided `Scope`, if any.
/// If any of the parsing into a macros goes wrong down the line, returns a `ConfigError` instead.
pub fn build_scope_macros(
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

    let mut matching_events: Vec<EventMatcher> = vec![];
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

#[cfg(test)]
mod tests {
    use crate::config::raw_config::{RCHash, k, RawConfig};
    use crate::config::versions::version1::macros::{build_macro, build_scope_macros};
    use crate::macros::{Macro, Scope};
    use crate::macros::event_matching::{EventMatcher, MatcherType};
    use crate::macros::event_matching::midi::MidiEventMatcher;
    use crate::macros::actions::Action;
    use crate::config::versions::version1::scope::build_scope;
    use crate::match_checker::StringMatcher;

    #[test]
    fn build_macro_returns_an_error_if_no_matching_events_are_specified() {
        let mut hash = RCHash::new();

        let mut enter_text_hash = RCHash::new();
        enter_text_hash.insert(k("type"), k("enter_text"));
        enter_text_hash.insert(k("data"), k("Hello"));

        hash.insert(k("actions"), RawConfig::Array(vec![
            RawConfig::Hash(enter_text_hash)
        ]));

        let broken_macro = build_macro(&hash, None);
        assert!(broken_macro.is_err());

        // Specified but empty array
        let mut hash = RCHash::new();
        hash.insert(k("matching_events"), RawConfig::Array(vec![]));

        let mut enter_text_hash = RCHash::new();
        enter_text_hash.insert(k("type"), k("enter_text"));
        enter_text_hash.insert(k("data"), k("Hello"));

        hash.insert(k("actions"), RawConfig::Array(vec![
            RawConfig::Hash(enter_text_hash)
        ]));

        let broken_macro = build_macro(&hash, None);
        assert!(broken_macro.is_err());
    }

    #[test]
    fn build_macro_returns_an_error_if_no_actions_are_specified() {
        let mut hash = RCHash::new();

        let mut evt_hash = RCHash::new();
        evt_hash.insert(k("type"), k("midi"));

        let mut note_on_hash = RCHash::new();
        note_on_hash.insert(k("message_type"), k("note_on"));

        evt_hash.insert(k("data"), RawConfig::Hash(note_on_hash));

        hash.insert(k("matching_events"), RawConfig::Array(vec![
            RawConfig::Hash(evt_hash)
        ]));

        let broken_macro = build_macro(&hash, None);
        assert!(broken_macro.is_err());

        // Specified but empty array
        let mut hash = RCHash::new();

        let mut evt_hash = RCHash::new();
        evt_hash.insert(k("type"), k("midi"));

        let mut note_on_hash = RCHash::new();
        note_on_hash.insert(k("message_type"), k("note_on"));

        evt_hash.insert(k("data"), RawConfig::Hash(note_on_hash));

        hash.insert(k("matching_events"), RawConfig::Array(vec![
            RawConfig::Hash(evt_hash)
        ]));

        hash.insert(k("actions"), RawConfig::Array(vec![]));

        let broken_macro = build_macro(&hash, None);
        assert!(broken_macro.is_err());
    }

    #[test]
    fn returns_error_if_invalid_event_matcher_data_specified() {
        let mut hash = RCHash::new();

        let mut evt_hash = RCHash::new();
        evt_hash.insert(k("type"), k("invalid-event-type"));

        hash.insert(k("matching_events"), RawConfig::Array(vec![
            RawConfig::Hash(evt_hash)
        ]));

        let mut enter_text_hash = RCHash::new();
        enter_text_hash.insert(k("type"), k("enter_text"));
        enter_text_hash.insert(k("data"), k("Hello"));

        hash.insert(k("actions"), RawConfig::Array(vec![
            RawConfig::Hash(enter_text_hash)
        ]));

        let broken_macro = build_macro(&hash, None);
        assert!(broken_macro.is_err());
    }

    #[test]
    fn returns_error_if_invalid_action_data_specified() {
        let mut hash = RCHash::new();

        let mut evt_hash = RCHash::new();
        evt_hash.insert(k("type"), k("midi"));
        let mut note_on_hash = RCHash::new();
        note_on_hash.insert(k("message_type"), k("note_on"));
        evt_hash.insert(k("data"), RawConfig::Hash(note_on_hash));

        hash.insert(k("matching_events"), RawConfig::Array(vec![
            RawConfig::Hash(evt_hash)
        ]));

        let mut invalid_action_hash = RCHash::new();
        invalid_action_hash.insert(k("type"), k("invalid-action"));

        hash.insert(k("actions"), RawConfig::Array(vec![
            RawConfig::Hash(invalid_action_hash)
        ]));

        let broken_macro = build_macro(&hash, None);
        assert!(broken_macro.is_err());
    }

    #[test]
    fn builds_minimal_macro() {
        let mut hash = RCHash::new();

        let mut evt_hash = RCHash::new();
        evt_hash.insert(k("type"), k("midi"));
        let mut note_on_hash = RCHash::new();
        note_on_hash.insert(k("message_type"), k("note_on"));
        evt_hash.insert(k("data"), RawConfig::Hash(note_on_hash));

        let mut action_hash = RCHash::new();
        action_hash.insert(k("type"), k("enter_text"));
        action_hash.insert(k("data"), k("Hello"));

        hash.insert(k("matching_events"), RawConfig::Array(vec![
            RawConfig::Hash(evt_hash)
        ]));

        hash.insert(k("actions"), RawConfig::Array(vec![
            RawConfig::Hash(action_hash)
        ]));

        let simple_macro = build_macro(&hash, None)
            .ok().unwrap();

        assert_eq!(
            simple_macro,

            Macro {
                name: None,
                match_events: vec![
                    EventMatcher  {
                        matcher: MatcherType::Midi(MidiEventMatcher::NoteOn {
                            channel_match: None,
                            key_match: None,
                            velocity_match: None,
                        }),

                        required_preconditions: None
                    }
                ],
                required_preconditions: None,
                actions: vec![
                    Action::EnterText("Hello".to_string(), 1)
                ],
                scope: None
            }
        );
    }

    #[test]
    fn builds_macro_with_all_optional_fields() {
        let mut hash = RCHash::new();

        let mut evt_hash = RCHash::new();
        evt_hash.insert(k("type"), k("midi"));
        let mut note_on_hash = RCHash::new();
        note_on_hash.insert(k("message_type"), k("note_on"));
        evt_hash.insert(k("data"), RawConfig::Hash(note_on_hash));

        let mut action_hash = RCHash::new();
        action_hash.insert(k("type"), k("enter_text"));
        action_hash.insert(k("data"), k("Hello"));

        hash.insert(k("matching_events"), RawConfig::Array(vec![
            RawConfig::Hash(evt_hash)
        ]));

        hash.insert(k("actions"), RawConfig::Array(vec![
            RawConfig::Hash(action_hash)
        ]));

        hash.insert(k("name"), k("test macro"));

        hash.insert(k("required_preconditions"), RawConfig::Array(vec![
            RawConfig::Null,
            RawConfig::Null
        ]));

        let mut scope_hash = RCHash::new();
        let mut string_matcher_hash = RCHash::new();
        string_matcher_hash.insert(k("is"), k("match"));
        scope_hash.insert(k("window_name"), RawConfig::Hash(string_matcher_hash));

        let scope = build_scope(&scope_hash).ok().unwrap().unwrap();

        let proper_macro = build_macro(&hash, Some(scope))
            .ok().unwrap();

        assert_eq!(
            proper_macro,

            Macro {
                name: Some("test macro".to_string()),
                match_events: vec![
                    EventMatcher  {
                        matcher: MatcherType::Midi(MidiEventMatcher::NoteOn {
                            channel_match: None,
                            key_match: None,
                            velocity_match: None,
                        }),

                        required_preconditions: None
                    }
                ],
                required_preconditions: None,
                actions: vec![
                    Action::EnterText("Hello".to_string(), 1)
                ],

                scope: Some(Scope {
                    window_class: None,
                    window_name: Some(StringMatcher::Is("match".to_string()))
                })
            }
        );
    }

    #[test]
    fn attaches_scopes_to_list_of_macros() {
        let mut hash1 = RCHash::new();

        let mut evt_hash = RCHash::new();
        evt_hash.insert(k("type"), k("midi"));
        let mut note_on_hash = RCHash::new();
        note_on_hash.insert(k("message_type"), k("note_on"));
        evt_hash.insert(k("data"), RawConfig::Hash(note_on_hash));

        let mut action_hash = RCHash::new();
        action_hash.insert(k("type"), k("enter_text"));
        action_hash.insert(k("data"), k("Hello1"));

        hash1.insert(k("matching_events"), RawConfig::Array(vec![
            RawConfig::Hash(evt_hash)
        ]));

        hash1.insert(k("actions"), RawConfig::Array(vec![
            RawConfig::Hash(action_hash)
        ]));


        let mut hash2 = RCHash::new();

        let mut evt_hash = RCHash::new();
        evt_hash.insert(k("type"), k("midi"));
        let mut note_on_hash = RCHash::new();
        note_on_hash.insert(k("message_type"), k("note_off"));
        evt_hash.insert(k("data"), RawConfig::Hash(note_on_hash));

        let mut action_hash = RCHash::new();
        action_hash.insert(k("type"), k("enter_text"));
        action_hash.insert(k("data"), k("Hello2"));

        hash2.insert(k("matching_events"), RawConfig::Array(vec![
            RawConfig::Hash(evt_hash)
        ]));

        hash2.insert(k("actions"), RawConfig::Array(vec![
            RawConfig::Hash(action_hash)
        ]));


        let mut scope_hash = RCHash::new();
        let mut string_matcher_hash = RCHash::new();
        string_matcher_hash.insert(k("is"), k("match"));
        scope_hash.insert(k("window_name"), RawConfig::Hash(string_matcher_hash));

        let scope = build_scope(&scope_hash).ok().unwrap().unwrap();

        let macro_list = build_scope_macros(
            &vec![RawConfig::Hash(hash1), RawConfig::Hash(hash2)],
            Some(scope)
        ).ok().unwrap();

        assert_eq!(

            macro_list,

            vec![
                Macro {
                    name: None,
                    match_events: vec![
                        EventMatcher  {
                            matcher: MatcherType::Midi(MidiEventMatcher::NoteOn {
                                channel_match: None,
                                key_match: None,
                                velocity_match: None,
                            }),

                            required_preconditions: None
                        }
                    ],

                    required_preconditions: None,

                    actions: vec![
                        Action::EnterText("Hello1".to_string(), 1)
                    ],

                    scope: Some(Scope {
                        window_class: None,
                        window_name: Some(StringMatcher::Is("match".to_string()))
                    })
                },


                Macro {
                    name: None,
                    match_events: vec![
                        EventMatcher  {
                            matcher: MatcherType::Midi(MidiEventMatcher::NoteOff {
                                channel_match: None,
                                key_match: None,
                                velocity_match: None,
                            }),

                            required_preconditions: None
                        }
                    ],

                    required_preconditions: None,

                    actions: vec![
                        Action::EnterText("Hello2".to_string(), 1)
                    ],

                    scope: Some(Scope {
                        window_class: None,
                        window_name: Some(StringMatcher::Is("match".to_string()))
                    })
                },
            ]
        );
    }
}