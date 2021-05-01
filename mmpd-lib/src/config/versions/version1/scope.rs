use crate::config::raw_config::{RCHash, AccessHelpers};
use crate::macros::Scope;
use crate::config::ConfigError;
use crate::config::versions::version1::primitive_matchers::build_string_matcher;

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
/// executable_path:
///     # (string matcher)
/// executable_basename:
///     # (string matcher)
/// macros:
///     # list of macros
/// ```
/// `window_class`, `window_name`, `executable_path`, and `executable_basename` are optional.
/// All the ones that are specified need to match the focused window's fields for the scope to
/// match.
///
/// The expected structure of string matcher is described by `build_string_matcher`.
///
/// The `macros` field is not actually used in the `build_scope` function, but is shown for
/// completeness.
///
/// ## Errors
/// This function will return `ConfigError` if constructing a `StringMatcher` fails for any reason.
pub (crate) fn build_scope(raw_scope: &RCHash) -> Result<Option<Scope>, ConfigError> {
    const WINDOW_CLASS_FIELD: &str = "window_class";
    const WINDOW_NAME_FIELD: &str = "window_name";
    const EXECUTABLE_PATH_FIELD: &str = "executable_path";
    const EXECUTABLE_BASENAME_FIELD: &str = "executable_basename";

    let window_class_matcher = build_string_matcher(
        raw_scope.get_hash(WINDOW_CLASS_FIELD)
    )?;

    let window_name_matcher = build_string_matcher(
        raw_scope.get_hash(WINDOW_NAME_FIELD)
    )?;

    let executable_path_matcher = build_string_matcher(
        raw_scope.get_hash(EXECUTABLE_PATH_FIELD)
    )?;

    let executable_basename_matcher = build_string_matcher(
        raw_scope.get_hash(EXECUTABLE_BASENAME_FIELD)
    )?;

    Ok(
        Scope::new(
            window_class_matcher,
            window_name_matcher,
            executable_path_matcher,
            executable_basename_matcher
        ).into_option()
    )
}

#[cfg(test)]
mod tests {
    use crate::config::raw_config::{RCHash, k, RawConfig, RCHashBuilder};
    use crate::config::versions::version1::scope::build_scope;
    use crate::macros::Scope;
    use crate::match_checker::StringMatcher;

    #[test]
    fn builds_scope_out_of_matchers_hash() {
        let input = RCHashBuilder::new()
            .insert(
                k("window_class"),
                RawConfig::Hash(
                    RCHashBuilder::new()
                        .insert(k("is"), k("class"))
                        .build()
                )
            )
            .insert(
                k("window_name"),
                RawConfig::Hash(
                    RCHashBuilder::new()
                        .insert(k("is"), k("name"))
                        .build()
                )
            )
            .insert(
                k("executable_path"),
                RawConfig::Hash(
                    RCHashBuilder::new()
                        .insert(k("is"), k("exec_path"))
                        .build()
                )
            )
            .insert(
                k("executable_basename"),
                RawConfig::Hash(
                    RCHashBuilder::new()
                        .insert(k("is"), k("exec_basename"))
                        .build()
                )
            )
            .build();

        let scope = build_scope(&input).ok().unwrap().unwrap();

        assert_eq!(
            scope,
            Scope {
                window_class: Some(StringMatcher::Is("class".to_string())),
                window_name: Some(StringMatcher::Is("name".to_string())),
                executable_path: Some(StringMatcher::Is("exec_path".to_string())),
                executable_basename: Some(StringMatcher::Is("exec_basename".to_string()))
            }
        );
    }

    #[test]
    fn builds_scope_with_one_matcher_none() {
        let mut window_class_hash = RCHash::new();
        window_class_hash.insert(k("is"), k("class"));

        let mut input = RCHash::new();
        input.insert(k("window_class"), RawConfig::Hash(window_class_hash));

        let scope = build_scope(&input).ok().unwrap().unwrap();

        assert_eq!(
            scope,
            Scope {
                window_class: Some(StringMatcher::Is("class".to_string())),
                window_name: None,
                executable_path: None,
                executable_basename: None,
            }
        );

        let mut window_name_hash = RCHash::new();
        window_name_hash.insert(k("is"), k("name"));

        let mut input = RCHash::new();
        input.insert(k("window_name"), RawConfig::Hash(window_name_hash));

        let scope = build_scope(&input).ok().unwrap().unwrap();

        assert_eq!(
            scope,
            Scope {
                window_class: None,
                window_name: Some(StringMatcher::Is("name".to_string())),
                executable_path: None,
                executable_basename: None,
            }
        );
    }

    #[test]
    fn build_scope_without_either_matcher_returns_none() {
        let scope = build_scope(&RCHash::new()).ok().unwrap();
        assert!(scope.is_none());
    }

    #[test]
    fn build_scope_with_invalid_string_matcher_returns_error() {
        let mut window_class_hash = RCHash::new();
        window_class_hash.insert(k("regex"), k("foo(bar"));

        let mut input = RCHash::new();
        input.insert(k("window_class"), RawConfig::Hash(window_class_hash));

        let scope = build_scope(&input);

        assert!(scope.is_err());
    }
}
