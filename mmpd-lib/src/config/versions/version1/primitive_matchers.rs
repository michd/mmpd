use crate::config::raw_config::{RCHash, RawConfig, AccessHelpers};
use crate::match_checker::{StringMatcher, NumberMatcher};
use crate::config::ConfigError;
use regex::Regex;
use crate::midi;

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
pub fn build_string_matcher(
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
pub fn build_number_matcher(matcher: Option<&RawConfig>) -> Result<Option<NumberMatcher>, ConfigError> {
    const MIN_FIELD: &str = "min";
    const MAX_FIELD: &str = "max";

    if let Some(matcher) = matcher {
        Ok(match matcher {
            RawConfig::Integer(i) => {
                if *i >= 0 {
                    Some(NumberMatcher::Val(*i as u32))
                } else {
                    return Err(ConfigError::InvalidConfig(
                       format!("Number matcher only supports positive integers, {} given", *i)
                    ));
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

pub fn build_number_matcher_from_musical_note(key_str: &str) -> Result<NumberMatcher, ConfigError> {
    let midi_notes = midi::parse_keys_from_str(key_str);

    if midi_notes.is_empty() {
        return Err(ConfigError::InvalidConfig(format!(
            "The key string '{}' matches no valid MIDI note number(s)",
            key_str
        )));
    }

    Ok(if midi_notes.len() == 1 {
        NumberMatcher::Val(
            *midi_notes.first().unwrap() as u32
        )
    } else {
        NumberMatcher::List(
            midi_notes
                .iter()
                .map(|n| {
                    NumberMatcher::Val(*n as u32)
                })
                .collect()
        )
    })
}

#[cfg(test)]
mod string_matcher_tests {
    use crate::config::raw_config::{RCHash, k};
    use crate::config::versions::version1::primitive_matchers::build_string_matcher;
    use crate::match_checker::StringMatcher;
    use regex::Regex;

    #[test]
    fn builds_is_string_matcher() {
        let mut input = RCHash::new();
        input.insert(k("is"), k("match"));

        let matcher = build_string_matcher(Some(&input))
            .ok().unwrap().unwrap();

        assert_eq!(matcher, StringMatcher::Is("match".to_string()))
    }

    #[test]
    fn builds_contains_string_matcher() {
        let mut input = RCHash::new();
        input.insert(k("contains"), k("match"));

        let matcher = build_string_matcher(Some(&input))
            .ok().unwrap().unwrap();

        assert_eq!(matcher, StringMatcher::Contains("match".to_string()));
    }

    #[test]
    fn builds_starts_with_string_matcher() {
        let mut input = RCHash::new();
        input.insert(k("starts_with"), k("match"));

        let matcher = build_string_matcher(Some(&input))
            .ok().unwrap().unwrap();

        assert_eq!(matcher, StringMatcher::StartsWith("match".to_string()));
    }

    #[test]
    fn builds_ends_with_string_matcher() {
        let mut input = RCHash::new();
        input.insert(k("ends_with"), k("match"));

        let matcher = build_string_matcher(Some(&input))
            .ok().unwrap().unwrap();

        assert_eq!(matcher, StringMatcher::EndsWith("match".to_string()));
    }

    #[test]
    fn builds_regex_string_matcher() {
        let mut input = RCHash::new();
        input.insert(k("regex"), k("foo|bar"));

        let matcher = build_string_matcher(Some(&input))
            .ok().unwrap().unwrap();

        assert_eq!(matcher, StringMatcher::Regex(Regex::new("foo|bar").ok().unwrap()));
    }

    #[test]
    fn regex_string_matcher_returns_error_on_invalid_regex() {
        let mut input = RCHash::new();
        input.insert(k("regex"), k("foo(bar"));

        let matcher = build_string_matcher(Some(&input));

        assert!(matcher.is_err());
    }

    #[test]
    fn builds_none_string_matcher_if_key_not_found() {
        let mut input = RCHash::new();
        input.insert(k("gobbledygook"), k("hello"));

        let matcher = build_string_matcher(Some(&input));

        assert!(if let Ok(None) = matcher { true } else { false });
    }

    #[test]
    fn ignores_capitalisation_in_string_matcher_keys() {
        let mut input_is = RCHash::new();
        input_is.insert(k("Is"), k("match"));
        let matcher_is = build_string_matcher(Some(&input_is))
            .ok().unwrap().unwrap();

        let mut input_contains = RCHash::new();
        input_contains.insert(k("conTaInS"), k("match"));
        let matcher_contains = build_string_matcher(Some(&input_contains))
            .ok().unwrap().unwrap();

        let mut input_starts_with = RCHash::new();
        input_starts_with.insert(k("StArTs_WITH"), k("match"));
        let matcher_starts_with = build_string_matcher(Some(&input_starts_with))
            .ok().unwrap().unwrap();

        let mut input_ends_with = RCHash::new();
        input_ends_with.insert(k("Ends_WITH"), k("match"));
        let matcher_ends_with = build_string_matcher(Some(&input_ends_with))
            .ok().unwrap().unwrap();

        let mut input_regex = RCHash::new();
        input_regex.insert(k("REGEx"), k("match|other_match"));
        let matcher_regex = build_string_matcher(Some(&input_regex))
            .ok().unwrap().unwrap();


        assert_eq!(matcher_is, StringMatcher::Is("match".to_string()));
        assert_eq!(matcher_contains, StringMatcher::Contains("match".to_string()));
        assert_eq!(matcher_starts_with, StringMatcher::StartsWith("match".to_string()));
        assert_eq!(matcher_ends_with, StringMatcher::EndsWith("match".to_string()));
        assert_eq!(matcher_regex, StringMatcher::Regex(Regex::new("match|other_match").unwrap()));
    }
}

#[cfg(test)]
mod number_matcher_tests {
    use crate::config::raw_config::{RawConfig, RCHash, k};
    use crate::config::versions::version1::primitive_matchers::build_number_matcher;
    use crate::match_checker::NumberMatcher;

    #[test]
    fn build_val_number_matcher() {
        let input_pos = RawConfig::Integer(7);
        let matcher_pos = build_number_matcher(Some(&input_pos))
            .ok().unwrap().unwrap();

        let input_zero = RawConfig::Integer(0);
        let matcher_zero = build_number_matcher(Some(&input_zero))
            .ok().unwrap().unwrap();

        assert_eq!(matcher_pos, NumberMatcher::Val(7));
        assert_eq!(matcher_zero, NumberMatcher::Val(0));
    }

    #[test]
    fn returns_an_error_on_negative_val() {
        let input = RawConfig::Integer(-7);

        let matcher = build_number_matcher(Some(&input));

        assert!(matcher.is_err());
    }

    #[test]
    fn builds_range_number_matcher() {
        let mut input = RCHash::new();
        input.insert(k("min"), RawConfig::Integer(7));
        input.insert(k("max"), RawConfig::Integer(10));

        let matcher = build_number_matcher(Some(&RawConfig::Hash(input)))
            .ok().unwrap().unwrap();

        assert_eq!(matcher, NumberMatcher::Range { min: Some(7), max: Some(10) });
    }

    #[test]
    fn builds_open_ended_range_number_matcher() {
        let mut input_min = RCHash::new();
        input_min.insert(k("min"), RawConfig::Integer(7));
        let matcher_min = build_number_matcher(Some(&RawConfig::Hash(input_min)))
            .ok().unwrap().unwrap();

        let mut input_max = RCHash::new();
        input_max.insert(k("max"), RawConfig::Integer(10));
        let matcher_max = build_number_matcher(Some(&RawConfig::Hash(input_max)))
            .ok().unwrap().unwrap();

        assert_eq!(matcher_min, NumberMatcher::Range { min: Some(7), max: None });
        assert_eq!(matcher_max, NumberMatcher::Range { min: None, max: Some(10) });
    }

    #[test]
    fn returns_error_when_range_min_or_max_are_negative() {
        let mut input_min_neg = RCHash::new();
        input_min_neg.insert(k("min"), RawConfig::Integer(-3));
        input_min_neg.insert(k("max"), RawConfig::Integer(9));
        let matcher_min_neg = build_number_matcher(Some(&RawConfig::Hash(input_min_neg)));

        let mut input_max_neg = RCHash::new();
        input_max_neg.insert(k("min"), RawConfig::Integer(2));
        input_max_neg.insert(k("max"), RawConfig::Integer(-9));
        let matcher_max_neg = build_number_matcher(Some(&RawConfig::Hash(input_max_neg)));

        let mut input_both_neg = RCHash::new();
        input_both_neg.insert(k("min"), RawConfig::Integer(-3));
        input_both_neg.insert(k("max"), RawConfig::Integer(-1));
        let matcher_both_neg = build_number_matcher(Some(&RawConfig::Hash(input_both_neg)));

        assert!(matcher_min_neg.is_err());
        assert!(matcher_max_neg.is_err());
        assert!(matcher_both_neg.is_err());
    }

    #[test]
    fn returns_error_when_range_min_is_greater_than_max() {
        let mut input = RCHash::new();
        input.insert(k("min"), RawConfig::Integer(10));
        input.insert(k("max"), RawConfig::Integer(7));

        let matcher = build_number_matcher(Some(&RawConfig::Hash(input)));

        assert!(matcher.is_err());
    }

    #[test]
    fn builds_list_of_number_matchers() {
        let mut range_input_hash = RCHash::new();
        range_input_hash.insert(k("min"), RawConfig::Integer(12));
        range_input_hash.insert(k("max"), RawConfig::Integer(18));

        let input = RawConfig::Array(vec![
            RawConfig::Integer(3),
            RawConfig::Integer(7),
            RawConfig::Hash(range_input_hash),
            RawConfig::Array(vec![
                RawConfig::Integer(23),
                RawConfig::Integer(42)
            ])
        ]);

        let matcher = build_number_matcher(Some(&input))
            .ok().unwrap().unwrap();

        assert_eq!(
            matcher,
            NumberMatcher::List(vec![
                NumberMatcher::Val(3),
                NumberMatcher::Val(7),
                NumberMatcher::Range { min: Some(12), max: Some(18) },
                NumberMatcher::List(vec![
                    NumberMatcher::Val(23),
                    NumberMatcher::Val(42)
                ])
            ])
        )
    }
}

#[cfg(test)]
mod musical_note_number_matcher_tests {
    use crate::config::versions::version1::primitive_matchers::build_number_matcher_from_musical_note;
    use crate::match_checker::NumberMatcher;

    #[test]
    fn returns_error_on_invalid_key() {
        let key_str = "NYERGH";

        let matcher = build_number_matcher_from_musical_note(key_str);

        assert!(matcher.is_err());
    }

    #[test]
    fn builds_val_number_matcher_if_one_note_is_returned() {
        let key_str = "C3";

        let matcher = build_number_matcher_from_musical_note(key_str)
            .ok().unwrap();

        assert_eq!(matcher, NumberMatcher::Val(48));
    }

    #[test]
    fn builds_list_number_matcher_if_multiple_notes_are_returned() {
        let key_str = "Eb";

        let matcher = build_number_matcher_from_musical_note(key_str)
            .ok().unwrap();

        assert_eq!(
            matcher,
            NumberMatcher::List(vec![
                NumberMatcher::Val(3),
                NumberMatcher::Val(15),
                NumberMatcher::Val(27),
                NumberMatcher::Val(39),
                NumberMatcher::Val(51),
                NumberMatcher::Val(63),
                NumberMatcher::Val(75),
                NumberMatcher::Val(87),
                NumberMatcher::Val(99),
                NumberMatcher::Val(111),
                NumberMatcher::Val(123)
            ])
        );
    }
}