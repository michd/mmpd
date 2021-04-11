use crate::config::raw_config::{RCHash, AccessHelpers, k};
use crate::config::ConfigError;
use crate::config::versions::version1::primitive_matchers::{build_number_matcher, build_musical_key_matcher};
use crate::macros::event_matching::midi::MidiEventMatcher;

/// Constructs a `MidiEventMatcher` from a `data` `RCHash`.
///
/// `data` should be structured as follows:
/// ```yaml
/// message_type: note_on
/// channel: (number matcher)
/// key: (number matcher | musical note string)
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
/// For `note_on`, `note_off`, and `poly_aftertouch`'s `key` field, you can specify a string
/// describing a note, e.g.: "D#2", "A2", Bb1".
/// You can also leave out the octave number, to create a number matcher matching that note on every
/// octave, e.g.: "D#", "A", "Bb".
///
/// ## Errors
/// The function returns `ConfigError` in any of the following conditions:
///
/// - No `data` is specified
/// - No `message_type` string field is part of `data`
/// - `message_type` value is not one of the supported values
/// - Downstream there is an issue constructing a number matcher for any reason
pub fn build_midi_event_matcher(
    data: Option<&RCHash>
) -> Result<MidiEventMatcher, ConfigError> {

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

    Ok(match message_type {
        NOTE_ON_EVENT => {
            let raw_key_matcher = data.get(&k(KEY_FIELD));
            let raw_velocity_matcher = data.get(&k(VELOCITY_FIELD));

            MidiEventMatcher::NoteOn {
                channel_match,
                key_match: build_musical_key_matcher(raw_key_matcher)?,
                velocity_match: build_number_matcher(raw_velocity_matcher)?
            }
        }

        NOTE_OFF_EVENT => {
            let raw_key_matcher = data.get(&k(KEY_FIELD));
            let raw_velocity_matcher = data.get(&k(VELOCITY_FIELD));

            MidiEventMatcher::NoteOff {
                channel_match,
                key_match: build_musical_key_matcher(raw_key_matcher)?,
                velocity_match: build_number_matcher(raw_velocity_matcher)?
            }
        }

        POLY_AFTERTOUCH_EVENT => {
            let raw_key_matcher = data.get(&k(KEY_FIELD));
            let raw_value_matcher = data.get(&k(VALUE_FIELD));

            MidiEventMatcher::PolyAftertouch {
                channel_match,
                key_match: build_musical_key_matcher(raw_key_matcher)?,
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
    })
}

#[cfg(test)]
mod tests {
    use crate::config::versions::version1::event_matchers::midi::build_midi_event_matcher;
    use crate::config::raw_config::{RCHash, k, RawConfig, RCHashBuilder};
    use crate::macros::event_matching::midi::MidiEventMatcher;
    use crate::match_checker::NumberMatcher;

    #[test]
    fn returns_an_error_if_no_data_is_given() {
        let matcher = build_midi_event_matcher(None);
        assert!(matcher.is_err());
    }

    #[test]
    fn returns_an_error_if_message_type_field_is_missing() {
        let mut hash = RCHash::new();
        hash.insert(k("channel"), RawConfig::Integer(0));

        let matcher = build_midi_event_matcher(Some(&hash));
        assert!(matcher.is_err());
    }

    #[test]
    fn returns_an_error_if_message_type_has_an_unknown_value() {
        let mut hash = RCHash::new();
        hash.insert(k("message_type"), k("unsupported-message-type"));

        let matcher = build_midi_event_matcher(Some(&hash));
        assert!(matcher.is_err());
    }

    #[test]
    fn returns_an_error_if_an_invalid_number_matcher_is_specified() {
        let mut hash = RCHash::new();
        hash.insert(k("message_type"), k("note_on"));
        hash.insert(k("channel"), RawConfig::Integer(-3)); // negative ints not allowed

        let matcher = build_midi_event_matcher(Some(&hash));
        assert!(matcher.is_err());
    }

    #[test]
    fn builds_note_on_matcher() {
        let mut hash = RCHash::new();
        hash.insert(k("message_type"), k("note_on"));
        hash.insert(k("channel"), RawConfig::Integer(0));
        hash.insert(k("key"), RawConfig::Integer(20));
        hash.insert(k("velocity"), RawConfig::Integer(40));

        let matcher = build_midi_event_matcher(Some(&hash)).ok().unwrap();

        assert_eq!(
            matcher,
            MidiEventMatcher::NoteOn {
                channel_match: Some(NumberMatcher::Val(0)),
                key_match: Some(NumberMatcher::Val(20)),
                velocity_match: Some(NumberMatcher::Val(40))
            }
        );

        // Without any optional matchers
        let mut hash = RCHash::new();
        hash.insert(k("message_type"), k("note_on"));

        let matcher = build_midi_event_matcher(Some(&hash)).ok().unwrap();

        assert_eq!(
            matcher,
            MidiEventMatcher::NoteOn {
                channel_match: None,
                key_match: None,
                velocity_match: None
            }
        );

        // With string key matcher
        let hash = RCHashBuilder::new()
            .insert(k("message_type"), k("note_on"))
            .insert(k("key"), k("C3"))
            .build();

        let matcher = build_midi_event_matcher(Some(&hash))
            .ok().unwrap();

        assert_eq!(
            matcher,
            MidiEventMatcher::NoteOn {
                channel_match: None,
                key_match: Some(NumberMatcher::Val(48)),
                velocity_match: None
            }
        );
    }

    #[test]
    fn builds_note_off_matcher() {
        let mut hash = RCHash::new();
        hash.insert(k("message_type"), k("note_off"));
        hash.insert(k("channel"), RawConfig::Integer(0));
        hash.insert(k("key"), RawConfig::Integer(20));
        hash.insert(k("velocity"), RawConfig::Integer(40));

        let matcher = build_midi_event_matcher(Some(&hash)).ok().unwrap();

        assert_eq!(
            matcher,
            MidiEventMatcher::NoteOff {
                channel_match: Some(NumberMatcher::Val(0)),
                key_match: Some(NumberMatcher::Val(20)),
                velocity_match: Some(NumberMatcher::Val(40))
            }
        );

        // Without any optional matchers
        let mut hash = RCHash::new();
        hash.insert(k("message_type"), k("note_off"));

        let matcher = build_midi_event_matcher(Some(&hash)).ok().unwrap();

        assert_eq!(
            matcher,
            MidiEventMatcher::NoteOff {
                channel_match: None,
                key_match: None,
                velocity_match: None
            }
        );

        // With string key matcher
        let hash = RCHashBuilder::new()
            .insert(k("message_type"), k("note_off"))
            .insert(k("key"), k("C3"))
            .build();

        let matcher = build_midi_event_matcher(Some(&hash))
            .ok().unwrap();

        assert_eq!(
            matcher,
            MidiEventMatcher::NoteOff {
                channel_match: None,
                key_match: Some(NumberMatcher::Val(48)),
                velocity_match: None
            }
        );

    }

    #[test]
    fn build_poly_aftertouch_matcher() {
        let mut hash = RCHash::new();
        hash.insert(k("message_type"), k("poly_aftertouch"));
        hash.insert(k("channel"), RawConfig::Integer(0));
        hash.insert(k("key"), RawConfig::Integer(20));
        hash.insert(k("value"), RawConfig::Integer(40));

        let matcher = build_midi_event_matcher(Some(&hash)).ok().unwrap();

        assert_eq!(
            matcher,
            MidiEventMatcher::PolyAftertouch {
                channel_match: Some(NumberMatcher::Val(0)),
                key_match: Some(NumberMatcher::Val(20)),
                value_match: Some(NumberMatcher::Val(40))
            }
        );

        // Without any optional matchers
        let mut hash = RCHash::new();
        hash.insert(k("message_type"), k("poly_aftertouch"));

        let matcher = build_midi_event_matcher(Some(&hash)).ok().unwrap();

        assert_eq!(
            matcher,
            MidiEventMatcher::PolyAftertouch {
                channel_match: None,
                key_match: None,
                value_match: None
            }
        );

        // With string key matcher
        let hash = RCHashBuilder::new()
            .insert(k("message_type"), k("poly_aftertouch"))
            .insert(k("key"), k("C3"))
            .build();

        let matcher = build_midi_event_matcher(Some(&hash))
            .ok().unwrap();

        assert_eq!(
            matcher,
            MidiEventMatcher::PolyAftertouch {
                channel_match: None,
                key_match: Some(NumberMatcher::Val(48)),
                value_match: None
            }
        );
    }

    #[test]
    fn build_control_change_matcher() {
        let mut hash = RCHash::new();
        hash.insert(k("message_type"), k("control_change"));
        hash.insert(k("channel"), RawConfig::Integer(0));
        hash.insert(k("control"), RawConfig::Integer(20));
        hash.insert(k("value"), RawConfig::Integer(40));

        let matcher = build_midi_event_matcher(Some(&hash)).ok().unwrap();

        assert_eq!(
            matcher,
            MidiEventMatcher::ControlChange {
                channel_match: Some(NumberMatcher::Val(0)),
                control_match: Some(NumberMatcher::Val(20)),
                value_match: Some(NumberMatcher::Val(40))
            }
        );

        // Without any optional matchers
        let mut hash = RCHash::new();
        hash.insert(k("message_type"), k("control_change"));

        let matcher = build_midi_event_matcher(Some(&hash)).ok().unwrap();

        assert_eq!(
            matcher,
            MidiEventMatcher::ControlChange {
                channel_match: None,
                control_match: None,
                value_match: None
            }
        );
    }

    #[test]
    fn build_program_change_matcher() {
        let mut hash = RCHash::new();
        hash.insert(k("message_type"), k("program_change"));
        hash.insert(k("channel"), RawConfig::Integer(0));
        hash.insert(k("program"), RawConfig::Integer(20));

        let matcher = build_midi_event_matcher(Some(&hash)).ok().unwrap();

        assert_eq!(
            matcher,
            MidiEventMatcher::ProgramChange {
                channel_match: Some(NumberMatcher::Val(0)),
                program_match: Some(NumberMatcher::Val(20)),
            }
        );

        // Without any optional matchers
        let mut hash = RCHash::new();
        hash.insert(k("message_type"), k("program_change"));

        let matcher = build_midi_event_matcher(Some(&hash)).ok().unwrap();

        assert_eq!(
            matcher,
            MidiEventMatcher::ProgramChange {
                channel_match: None,
                program_match: None,
            }
        );
    }

    #[test]
    fn build_channel_aftertouch_matcher() {
        let mut hash = RCHash::new();
        hash.insert(k("message_type"), k("channel_aftertouch"));
        hash.insert(k("channel"), RawConfig::Integer(0));
        hash.insert(k("value"), RawConfig::Integer(20));

        let matcher = build_midi_event_matcher(Some(&hash)).ok().unwrap();

        assert_eq!(
            matcher,
            MidiEventMatcher::ChannelAftertouch {
                channel_match: Some(NumberMatcher::Val(0)),
                value_match: Some(NumberMatcher::Val(20)),
            }
        );

        // Without any optional matchers
        let mut hash = RCHash::new();
        hash.insert(k("message_type"), k("channel_aftertouch"));

        let matcher = build_midi_event_matcher(Some(&hash)).ok().unwrap();

        assert_eq!(
            matcher,
            MidiEventMatcher::ChannelAftertouch {
                channel_match: None,
                value_match: None,
            }
        );
    }

    #[test]
    fn build_pitch_bend_change_matcher() {
        let mut hash = RCHash::new();
        hash.insert(k("message_type"), k("pitch_bend_change"));
        hash.insert(k("channel"), RawConfig::Integer(0));
        hash.insert(k("value"), RawConfig::Integer(20));

        let matcher = build_midi_event_matcher(Some(&hash)).ok().unwrap();

        assert_eq!(
            matcher,
            MidiEventMatcher::PitchBendChange {
                channel_match: Some(NumberMatcher::Val(0)),
                value_match: Some(NumberMatcher::Val(20)),
            }
        );

        // Without any optional matchers
        let mut hash = RCHash::new();
        hash.insert(k("message_type"), k("pitch_bend_change"));

        let matcher = build_midi_event_matcher(Some(&hash)).ok().unwrap();

        assert_eq!(
            matcher,
            MidiEventMatcher::PitchBendChange {
                channel_match: None,
                value_match: None,
            }
        );
    }
}
