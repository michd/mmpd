use crate::config::raw_config::{RCHash, AccessHelpers, k};
use crate::match_checker::MatchChecker;
use crate::midi::MidiMessage;
use crate::config::ConfigError;
use crate::config::versions::version1::primitive_matchers::build_number_matcher;
use crate::macros::event_matching::midi::MidiEventMatcher;

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
pub fn build_midi_event_matcher(
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
