mod midi;

use crate::config::raw_config::{RCHash, AccessHelpers, RawConfig};
use crate::macros::event_matching::{EventMatcher, MatcherType};
use crate::config::ConfigError;
use crate::macros::preconditions::Precondition;
use crate::config::versions::version1::precondition::build_precondition;
use midi::build_midi_event_matcher;

/// Constructs an `EventMatcher` instance from a Raw `raw_event_matcher`
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
/// `data` is meant to be a hash, but is not strictly required. Depending on the event type, it
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
/// - `type` field is missing or is not a `RawConfig::String`
/// - The value for the `type` field does not match any known event matcher types; see above
/// - Down the stream, a more specific event matcher (such as `MidiEventMatcher`) fails to be
///   constructed for any reason
/// - Down the stream, a `Precondition` fails to be constructed for any reason
pub (crate) fn build_event_matcher(raw_event_matcher: &RCHash) -> Result<EventMatcher, ConfigError> {
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

    Ok(EventMatcher::new(
            matcher_type,
            if preconditions.is_empty() { None } else { Some(preconditions) }
        )
    )
}

#[cfg(test)]
mod tests {
    use crate::config::raw_config::{RCHash, k, RawConfig};
    use crate::config::versions::version1::event_matchers::build_event_matcher;
    use crate::macros::event_matching::{EventMatcher, MatcherType};
    use crate::macros::event_matching::midi::MidiEventMatcher;
    use crate::macros::preconditions::{Precondition, PreconditionType};

    #[test]
    fn returns_error_if_missing_type_field() {
        let data_hash = RCHash::new();

        let mut hash = RCHash::new();
        hash.insert(k("data"), RawConfig::Hash(data_hash));

        let matcher = build_event_matcher(&hash);
        assert!(matcher.is_err());
    }

    #[test]
    fn return_error_if_type_is_an_unsupported_value() {
        let data_hash = RCHash::new();

        let mut hash = RCHash::new();
        hash.insert(k("type"), k("unsupported-type-value"));
        hash.insert(k("data"), RawConfig::Hash(data_hash));

        let matcher = build_event_matcher(&hash);
        assert!(matcher.is_err());
    }

    #[test]
    fn builds_midi_event_matcher() {
        let mut data_hash = RCHash::new();
        data_hash.insert(k("message_type"), k("note_on"));

        let mut hash = RCHash::new();
        hash.insert(k("type"), k("midi"));
        hash.insert(k("data"), RawConfig::Hash(data_hash));

        let matcher = build_event_matcher(&hash)
            .ok().unwrap();

        assert_eq!(
            matcher,
            EventMatcher {
                matcher: MatcherType::Midi(MidiEventMatcher::NoteOn {
                    channel_match: None,
                    key_match: None,
                    velocity_match: None
                }),

                required_preconditions: None
            }
        );
    }

    #[test]
    fn returns_an_error_if_data_for_underlying_matcher_is_invalid() {
        let mut data_hash = RCHash::new();
        data_hash.insert(k("message_type"), k("invalid_message_type"));

        let mut hash = RCHash::new();
        hash.insert(k("type"), k("midi"));
        hash.insert(k("data"), RawConfig::Hash(data_hash));

        let matcher = build_event_matcher(&hash);
        assert!(matcher.is_err());
    }

    #[test]
    fn builds_an_event_matcher_with_preconditions() {
        let mut data_hash = RCHash::new();
        data_hash.insert(k("message_type"), k("note_on"));

        let mut hash = RCHash::new();
        hash.insert(k("type"), k("midi"));
        hash.insert(k("data"), RawConfig::Hash(data_hash));
        hash.insert(k("required_preconditions"), RawConfig::Array(vec![
            RawConfig::Hash(RCHash::new()),
            RawConfig::Hash(RCHash::new()),
            RawConfig::Hash(RCHash::new()),
        ]));

        let matcher = build_event_matcher(&hash)
            .ok().unwrap();

        assert_eq!(
            matcher,
            EventMatcher {
                matcher: MatcherType::Midi(MidiEventMatcher::NoteOn {
                    channel_match: None,
                    key_match: None,
                    velocity_match: None
                }),

                required_preconditions: Some(vec![
                    Precondition { invert: false, condition: PreconditionType::Other },
                    Precondition { invert: false, condition: PreconditionType::Other },
                    Precondition { invert: false, condition: PreconditionType::Other },
                ])
            }
        );
    }
}
