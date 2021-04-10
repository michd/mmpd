mod midi;

use crate::config::raw_config::{RCHash, AccessHelpers};
use crate::macros::preconditions::{Precondition, PreconditionType};
use crate::config::ConfigError;
use crate::config::versions::version1::precondition::midi::build_midi_precondition;

/// Constructs a `Precondition` instance from a raw `raw_precondition` `RCHash`'s fields.
///
/// Preconditions are expected to follow this structure:
///
/// ```yaml
/// type: "precondition type goes here"
/// invert: true|false
/// data:
///     # (Any fields relevant to the precondition type)
/// ```
///
/// `type` is required. Its value must be one of the implemented precondition types. Currently,
/// there are:
///     - midi
///
/// `invert` is optional, and specifies whether the condition should be inverted; it essentially
/// applies a logic "NOT" to the question "does this precondition match?" Defaults to `false`.
///
/// `data` is meant to be a hash, but is not strictly required. Depending on the event type it may
/// be required, but this function does not enforce it.
///
/// ## Errors
/// This function will return `ConfigError` under any of these conditions:
///
/// - `type` field is missing or is not a `RawConfig::String`
/// - The value for the `type` field does not match any known precondition types; see above
/// - Down the stream, a precondition type such as `MidiPrecondition`  fails to be constructed for
///   any reason
pub (crate) fn build_precondition(raw_precondition: &RCHash) -> Result<Precondition, ConfigError> {
    const TYPE_FIELD: &str = "type";
    const INVERT_FIELD: &str = "invert";
    const DATA_FIELD: &str = "data";

    const TYPE_MIDI: &str = "midi";

    // Allows a building a simple do-nothing precondition in tests
    #[cfg(test)]
    const TYPE_OTHER: &str = "other";

    let condition_type = raw_precondition.get_string(TYPE_FIELD).ok_or_else(|| {
        ConfigError::InvalidConfig(
            format!("precondition missing valid (string) '{}' field", TYPE_FIELD)
        )
    })?;

    let invert = raw_precondition.get_bool(INVERT_FIELD).unwrap_or(false);

    let data = raw_precondition.get_hash(DATA_FIELD);

    Ok(Precondition {
        invert,
        condition: match condition_type {
            TYPE_MIDI => PreconditionType::Midi(build_midi_precondition(data)?),

            // Allows building a simple do-nothing precondition in tests
            #[cfg(test)]
            TYPE_OTHER => PreconditionType::Other,

            _ => {
                return Err(ConfigError::InvalidConfig(
                    format!("Unknown precondition type '{}'", condition_type)
                ));
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::config::raw_config::{RCHash, k};
    use crate::config::raw_config::RawConfig;
    use crate::config::versions::version1::precondition::build_precondition;
    use crate::macros::preconditions::{Precondition, PreconditionType};
    use crate::macros::preconditions::midi::MidiPrecondition;

    #[test]
    fn builds_a_valid_precondition() {
        let mut hash = RCHash::new();
        hash.insert(k("type"), k("midi"));
        hash.insert(k("invert"), RawConfig::Bool(true));

        let mut midi_hash = RCHash::new();
        midi_hash.insert(k("condition_type"), k("program"));

        hash.insert(k("data"), RawConfig::Hash(midi_hash));

        let condition = build_precondition(&hash)
            .ok().unwrap();

        assert_eq!(
            condition,
            Precondition {
                invert: true,
                condition: PreconditionType::Midi(
                    MidiPrecondition::Program {
                        channel_match: None,
                        program_match: None
                    }
                )
            }
        );
    }

    #[test]
    fn returns_an_error_if_type_field_is_missing() {
        let hash = RCHash::new();
        let condition = build_precondition(&hash);
        assert!(condition.is_err());
    }

    #[test]
    fn returns_an_error_if_type_field_has_invalid_value() {
        let mut hash = RCHash::new();
        hash.insert(k("type"), k("InvalidType"));
        let condition = build_precondition(&hash);
        assert!(condition.is_err());
    }

    #[test]
    fn returns_an_error_if_downstream_build_errors() {
        let mut hash = RCHash::new();
        hash.insert(k("type"), k("midi"));
        hash.insert(k("data"), RawConfig::Null);
        let condition = build_precondition(&hash);
        assert!(condition.is_err());
    }
}