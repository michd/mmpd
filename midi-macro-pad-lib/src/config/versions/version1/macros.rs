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

