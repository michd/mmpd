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
/// macros:
///     # list of macros
/// ```
/// Both `window_class` and `window_name` are optional. If both are specified, then for the scope
/// to match, the string matchers for both must matched the focused window's fields.
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

    let window_class_matcher = build_string_matcher(
        raw_scope.get_hash(WINDOW_CLASS_FIELD)
    )?;

    let window_name_matcher = build_string_matcher(
        raw_scope.get_hash(WINDOW_NAME_FIELD)
    )?;

    let has_any_matchers = vec![&window_class_matcher, &window_name_matcher]
        .iter()
        .any(|matcher| matcher.is_some());

    Ok(if has_any_matchers {
        Some(
            Scope::new(window_class_matcher, window_name_matcher)
        )
    } else {
        None
    })
}

#[cfg(test)]
mod tests {
    use crate::config::raw_config::{RCHash, k, RawConfig};
    use crate::config::versions::version1::scope::build_scope;
    use crate::macros::Scope;
    use crate::match_checker::StringMatcher;

    #[test]
    fn builds_scope_out_of_matchers_hash() {
        let mut window_class_hash = RCHash::new();
        window_class_hash.insert(k("is"), k("class"));

        let mut window_name_hash = RCHash::new();
        window_name_hash.insert(k("is"), k("name"));

        let mut input = RCHash::new();
        input.insert(k("window_class"), RawConfig::Hash(window_class_hash));
        input.insert(k("window_name"), RawConfig::Hash(window_name_hash));

        let scope = build_scope(&input).ok().unwrap().unwrap();

        assert_eq!(
            scope,
            Scope {
                window_class: Some(StringMatcher::Is("class".to_string())),
                window_name: Some(StringMatcher::Is("name".to_string()))
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
                window_name: None
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
                window_name: Some(StringMatcher::Is("name".to_string()))
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
