use regex::Regex;

pub trait MatchChecker<T> {
    fn matches(&self, val: &T) -> bool;
}

pub enum StringMatcher<'a> {
    Is(&'a str),
    Contains(&'a str),
    StartsWith(&'a str),
    EndsWith(&'a str),
    Regex(Regex)
}

// TODO unit test
impl MatchChecker<&str> for StringMatcher<'_> {
    fn matches(&self, val: &&str) -> bool {
        match self {
            StringMatcher::Is(pattern) => val == pattern,
            StringMatcher::Contains(pattern) => val.contains(pattern),
            StringMatcher::StartsWith(pattern) => val.starts_with(pattern),
            StringMatcher::EndsWith(pattern) => val.ends_with(pattern),
            StringMatcher::Regex(pattern) => pattern.is_match(val)
        }
    }
}

// For the time being this is hardcoded with u32, but could potentially be made more flexible with
// a type parameter constrained to the PartialOrd trait.
pub enum NumberMatcher {
    Any,
    Range { min: Option<u32>, max: Option<u32> },
    Val(u32),
    List(Vec<NumberMatcher>),
}

impl MatchChecker<u32> for NumberMatcher {
    fn matches(&self, input_num: &u32) -> bool {
        match self {
            NumberMatcher::Any => true,

            NumberMatcher::Range { min, max } => {
                let mut match_so_far = true;

                if let Some(min) = min {
                    match_so_far = input_num >= min;
                }

                if !match_so_far {
                    return false;
                }

                if let Some(max) = max {
                    match_so_far = input_num <= max
                }

                match_so_far
            }

            NumberMatcher::Val(a) => *a == *input_num,

            NumberMatcher::List(matchers) => {
                matchers.iter().any(|m| m.matches(input_num))
            }
        }
    }
}

/// Convenience type for where a matcher is optional, allowing it to be used just like NumberMatcher
pub type NumMatch = Option<NumberMatcher>;

impl MatchChecker<u32> for NumMatch {
    fn matches(&self, val: &u32) -> bool {
        if let Some(matcher) = self {
            matcher.matches(val)
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::match_checker::{NumberMatcher, MatchChecker};

    #[test]
    fn number_matcher_any() {
        let matcher = NumberMatcher::Any;

        assert!(matcher.matches(&0));
        assert!(matcher.matches(&545));
        assert!(matcher.matches(&545646546));
    }

    #[test]
    fn number_matcher_range() {
        let min_matcher = NumberMatcher::Range { min: Some(9), max: None };

        assert!(min_matcher.matches(&9));
        assert!(min_matcher.matches(&u32::MAX));
        assert!(!min_matcher.matches(&8));
        assert!(!min_matcher.matches(&2));
        assert!(!min_matcher.matches(&u32::MIN));

        let max_matcher = NumberMatcher::Range { min: None, max: Some(12) };

        assert!(max_matcher.matches(&12));
        assert!(max_matcher.matches(&u32::MIN));
        assert!(!max_matcher.matches(&13));
        assert!(!max_matcher.matches(&25648));
        assert!(!max_matcher.matches(&u32::MAX));

        let range_matcher = NumberMatcher::Range { min: Some(42), max: Some(9001) };

        assert!(range_matcher.matches(&42));
        assert!(range_matcher.matches(&5555));
        assert!(range_matcher.matches(&9001));
        assert!(!range_matcher.matches(&41));
        assert!(!range_matcher.matches(&24));
        assert!(!range_matcher.matches(&u32::MIN));
        assert!(!range_matcher.matches(&9002));
        assert!(!range_matcher.matches(&15000));
        assert!(!range_matcher.matches(&u32::MAX));

        let all_matcher = NumberMatcher::Range { min: None, max: None };

        assert!(all_matcher.matches(&u32::MIN));
        assert!(all_matcher.matches(&847));
        assert!(all_matcher.matches(&u32::MAX));
    }

    #[test]
    fn number_matcher_val() {
        let matcher = NumberMatcher::Val(1234);

        assert!(matcher.matches(&1234));
        assert!(!matcher.matches(&1233));
        assert!(!matcher.matches(&1235));
        assert!(!matcher.matches(&u32::MIN));
        assert!(!matcher.matches(&u32::MAX));
    }

    #[test]
    fn number_matcher_list() {
        let matcher = NumberMatcher::List(vec![
            NumberMatcher::Range { min: Some(10), max: Some(20) },
            NumberMatcher::Range { min: Some(30), max: Some(40) },
            NumberMatcher::Val(4242),
            NumberMatcher::Val(5000),
            NumberMatcher::Range { min: Some(9001), max: None }
        ]);

        assert!(!matcher.matches(&u32::MIN));
        assert!(!matcher.matches(&7));
        assert!(!matcher.matches(&9));
        assert!(matcher.matches(&10));
        assert!(matcher.matches(&15));
        assert!(matcher.matches(&20));
        assert!(!matcher.matches(&21));
        assert!(!matcher.matches(&28));
        assert!(!matcher.matches(&29));
        assert!(matcher.matches(&30));
        assert!(matcher.matches(&35));
        assert!(matcher.matches(&40));
        assert!(!matcher.matches(&41));
        assert!(!matcher.matches(&2021));
        assert!(matcher.matches(&4242));
        assert!(matcher.matches(&5000));
        assert!(!matcher.matches(&7500));
        assert!(matcher.matches(&9001));
        assert!(matcher.matches(&424242));
        assert!(matcher.matches(&u32::MAX));
    }
}






