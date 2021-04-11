use crate::macros::preconditions::midi::MidiPrecondition;
use crate::config::raw_config::{RCHash, AccessHelpers, k};
use crate::config::ConfigError;
use crate::config::versions::version1::primitive_matchers::{build_number_matcher, build_musical_key_matcher};

/// Constructs a `MidiPrecondition` from a `data` `RCHash`.
///
/// `data` should be structured as follows:
/// ```yaml
/// condition_type: note_on
/// channel: (number matcher)
/// key: (number matcher | musical note string)
/// ```
///
/// This is just one example; there are different valid properties, depending on the value of
/// `condition_type`. All the expected values for the the non-`condition_type` fields are number
/// matchers. see `build_number_matcher` for details of what a number matcher entails.
///
/// Here's an exhaustive list of additional available condition_types and the fields available for
/// them.
///
/// - `note_on` - A key is currently held
///     - `channel` - Which MIDI channel the key is pressed for (0-15)
///     - `key` - Which key is pressed (0-127)
/// - `control` - A control value is known and matches some value, from "control_change" messages
///     - `channel` - Which MIDI channel the control is on (0-15)
///     - `control` - Control identifier (0-127)
///     - `value` - Known value the control is set to (0-127)
/// - `program` - A program setting, from "program_change" messages
///     - `channel` - Which MIDI channel the program setting is for (0-15)
///     - `program` - The currently set program number (0-127)
/// - `pitch_bend` - Position of the pitch bender, from "pitch_bend_change" messages
///     - `channel` - Which MIDI channel the pitch bend setting is on (0-15)
///     - `value` - What the last known pitch bend value is (0-16383)
///
/// For `note_on`'s `key` field, you can specify a string describing a note, e.g.: "D#2", "A2", Bb1".
/// You can also leave out the octave number, to create a number matcher matching that note on every
/// octave, e.g.: "D#", "A", "Bb".
///
/// ## Errors
/// The function returns `ConfigError` in any of the following conditions:
///
/// - No `data` is specified
/// - No `condition_type` string field is found in `data`
/// - `condition_type` is not one of the support values (see above)
/// - Downstream, there is an issue constructing a number matcher for any reason
pub fn build_midi_precondition(
    data: Option<&RCHash>
) -> Result<MidiPrecondition, ConfigError> {
    const CONDITION_TYPE_FIELD: &str = "condition_type";

    const NOTE_ON_CONDITION: &str = "note_on";
    const CONTROL_CONDITION: &str = "control";
    const PROGRAM_CONDITION: &str = "program";
    const PITCH_BEND_CONDITION: &str = "pitch_bend";

    const CHANNEL_FIELD: &str = "channel";
    const KEY_FIELD: &str = "key";
    const CONTROL_FIELD: &str = "control";
    const VALUE_FIELD: &str = "value";
    const PROGRAM_FIELD: &str = "program";

    let data = data.ok_or_else(|| {
        ConfigError::InvalidConfig("Missing data for midi precondition".to_string())
    })?;

    let condition_type = data.get_string(CONDITION_TYPE_FIELD).ok_or_else(|| {
        ConfigError::InvalidConfig(format!(
            "Missing {} field in midi precondition data",
            CONDITION_TYPE_FIELD
        ))
    })?;

    let raw_channel_matcher = data.get(&k(CHANNEL_FIELD));
    let channel_match = build_number_matcher(raw_channel_matcher)?;

    Ok(match condition_type {
        NOTE_ON_CONDITION => MidiPrecondition::NoteOn {
            channel_match,
            key_match: build_musical_key_matcher(data.get(&k(KEY_FIELD)))?
        },

        CONTROL_CONDITION => MidiPrecondition::Control {
            channel_match,
            control_match: build_number_matcher(data.get(&k(CONTROL_FIELD)))?,
            value_match: build_number_matcher(data.get(&k(VALUE_FIELD)))?
        },

        PROGRAM_CONDITION => MidiPrecondition::Program {
            channel_match,
            program_match: build_number_matcher(data.get(&k(PROGRAM_FIELD)))?
        },

        PITCH_BEND_CONDITION => MidiPrecondition::PitchBend {
            channel_match,
            value_match: build_number_matcher(data.get(&k(VALUE_FIELD)))?
        },

        _ => {
            return Err(ConfigError::InvalidConfig(
               format!(
                   "Invalid or unsupported MIDI condition_type '{}'",
                   condition_type
               )
            ));
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::config::raw_config::{RCHash, k, RawConfig, RCHashBuilder};
    use crate::config::versions::version1::precondition::midi::build_midi_precondition;
    use crate::macros::preconditions::midi::MidiPrecondition;
    use crate::match_checker::NumberMatcher;

    #[test]
    fn builds_note_on_precondition() {
        let mut hash = RCHash::new();
        hash.insert(k("condition_type"), k("note_on"));
        hash.insert(k("channel"), RawConfig::Integer(2));
        hash.insert(k("key"), RawConfig::Integer(42));

        let condition = build_midi_precondition(Some(&hash))
            .ok().unwrap();

        assert_eq!(
            condition,
            MidiPrecondition::NoteOn {
                channel_match: Some(NumberMatcher::Val(2)),
                key_match: Some(NumberMatcher::Val(42))
            }
        );

        // Sparse version
        let mut hash = RCHash::new();
        hash.insert(k("condition_type"), k("note_on"));

        let condition = build_midi_precondition(Some(&hash))
            .ok().unwrap();

        assert_eq!(
            condition,
            MidiPrecondition::NoteOn {
                channel_match: None,
                key_match: None
            }
        );

        // With string key matcher
        let hash = RCHashBuilder::new()
            .insert(k("condition_type"), k("note_on"))
            .insert(k("key"), k("C3"))
            .build();

        let condition = build_midi_precondition(Some(&hash))
            .ok().unwrap();

        assert_eq!(
            condition,
            MidiPrecondition::NoteOn {
                channel_match: None,
                key_match: Some(NumberMatcher::Val(48))
            }
        )
    }

    #[test]
    fn builds_control_precondition() {
        let mut hash = RCHash::new();
        hash.insert(k("condition_type"), k("control"));
        hash.insert(k("channel"), RawConfig::Integer(2));
        hash.insert(k("control"), RawConfig::Integer(1));
        hash.insert(k("value"), RawConfig::Integer(75));

        let condition = build_midi_precondition(Some(&hash))
            .ok().unwrap();

        assert_eq!(
            condition,
            MidiPrecondition::Control {
                channel_match: Some(NumberMatcher::Val(2)),
                control_match: Some(NumberMatcher::Val(1)),
                value_match: Some(NumberMatcher::Val(75))
            }
        );

        // Sparse version
        let mut hash = RCHash::new();
        hash.insert(k("condition_type"), k("control"));

        let condition = build_midi_precondition(Some(&hash))
            .ok().unwrap();

        assert_eq!(
            condition,
            MidiPrecondition::Control {
                channel_match: None,
                control_match: None,
                value_match: None
            }
        );
    }

    #[test]
    fn builds_program_precondition() {
        let mut hash = RCHash::new();
        hash.insert(k("condition_type"), k("program"));
        hash.insert(k("channel"), RawConfig::Integer(2));
        hash.insert(k("program"), RawConfig::Integer(42));

        let condition = build_midi_precondition(Some(&hash))
            .ok().unwrap();

        assert_eq!(
            condition,
            MidiPrecondition::Program {
                channel_match: Some(NumberMatcher::Val(2)),
                program_match: Some(NumberMatcher::Val(42))
            }
        );

        // Sparse version
        let mut hash = RCHash::new();
        hash.insert(k("condition_type"), k("program"));

        let condition = build_midi_precondition(Some(&hash))
            .ok().unwrap();

        assert_eq!(
            condition,
            MidiPrecondition::Program {
                channel_match: None,
                program_match: None
            }
        );
    }

    #[test]
    fn builds_pitch_bend_precondition() {
        let mut hash = RCHash::new();
        hash.insert(k("condition_type"), k("pitch_bend"));
        hash.insert(k("channel"), RawConfig::Integer(2));
        hash.insert(k("value"), RawConfig::Integer(42));

        let condition = build_midi_precondition(Some(&hash))
            .ok().unwrap();

        assert_eq!(
            condition,
            MidiPrecondition::PitchBend {
                channel_match: Some(NumberMatcher::Val(2)),
                value_match: Some(NumberMatcher::Val(42))
            }
        );

        // Sparse version
        let mut hash = RCHash::new();
        hash.insert(k("condition_type"), k("pitch_bend"));

        let condition = build_midi_precondition(Some(&hash))
            .ok().unwrap();

        assert_eq!(
            condition,
            MidiPrecondition::PitchBend {
                channel_match: None,
                value_match: None,
            }
        );
    }

    #[test]
    fn returns_error_if_no_data_provided() {
        let condition = build_midi_precondition(None);
        assert!(condition.is_err());
    }

    #[test]
    fn returns_error_if_condition_type_field_is_missing() {
        let hash = RCHash::new();
        let condition = build_midi_precondition(Some(&hash));
        assert!(condition.is_err());
    }

    #[test]
    fn returns_error_if_condition_type_has_invalid_value() {
        let mut hash = RCHash::new();
        hash.insert(k("condition_type"), k("InvalidValueHere"));

        let condition = build_midi_precondition(Some(&hash));
        assert!(condition.is_err());
    }

    #[test]
    fn returns_error_if_invalid_number_matcher_data_provided() {
        // note_on, bad data for channel
        let mut hash = RCHash::new();
        hash.insert(k("condition_type"), k("note_on"));
        hash.insert(k("channel"), RawConfig::Integer(-1));
        let condition = build_midi_precondition(Some(&hash));
        assert!(condition.is_err());

        // note_on, bad data for key
        let mut hash = RCHash::new();
        hash.insert(k("condition_type"), k("note_on"));
        hash.insert(k("key"), RawConfig::Integer(-1));
        let condition = build_midi_precondition(Some(&hash));
        assert!(condition.is_err());

        // control, bad data for channel
        let mut hash = RCHash::new();
        hash.insert(k("condition_type"), k("control"));
        hash.insert(k("channel"), RawConfig::Integer(-1));
        let condition = build_midi_precondition(Some(&hash));
        assert!(condition.is_err());

        // control, bad data for control
        let mut hash = RCHash::new();
        hash.insert(k("condition_type"), k("control"));
        hash.insert(k("control"), RawConfig::Integer(-1));
        let condition = build_midi_precondition(Some(&hash));
        assert!(condition.is_err());

        // control, bad data for value
        let mut hash = RCHash::new();
        hash.insert(k("condition_type"), k("control"));
        hash.insert(k("value"), RawConfig::Integer(-1));
        let condition = build_midi_precondition(Some(&hash));
        assert!(condition.is_err());

        // program, bad data for channel
        let mut hash = RCHash::new();
        hash.insert(k("condition_type"), k("program"));
        hash.insert(k("channel"), RawConfig::Integer(-1));
        let condition = build_midi_precondition(Some(&hash));
        assert!(condition.is_err());

        // program, bad data for program
        let mut hash = RCHash::new();
        hash.insert(k("condition_type"), k("program"));
        hash.insert(k("program"), RawConfig::Integer(-1));
        let condition = build_midi_precondition(Some(&hash));
        assert!(condition.is_err());

        // pitch_bend, bad data for channel
        let mut hash = RCHash::new();
        hash.insert(k("condition_type"), k("pitch_bend"));
        hash.insert(k("channel"), RawConfig::Integer(-1));
        let condition = build_midi_precondition(Some(&hash));
        assert!(condition.is_err());

        // pitch_bend bad data for value
        let mut hash = RCHash::new();
        hash.insert(k("condition_type"), k("pitch_bend"));
        hash.insert(k("value"), RawConfig::Integer(-1));
        let condition = build_midi_precondition(Some(&hash));
        assert!(condition.is_err());
    }
}