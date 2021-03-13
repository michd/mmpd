use crate::config::raw_config::{RCHash, RawConfig, AccessHelpers};
use crate::match_checker::{StringMatcher, NumberMatcher};
use crate::config::ConfigError;
use regex::Regex;

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
