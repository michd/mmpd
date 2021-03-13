use crate::config::raw_config::RCHash;
use crate::macros::preconditions::Precondition;
use crate::config::ConfigError;

/// Constructs a `Precondition` from a `_raw_precondition` `RCHash`.
///
/// Since preconditions aren't implemented yet (beyond a stub), this always returns a blank
/// `Precondition` instance regardless of the contents of `_raw_precondition`.
pub (crate) fn build_precondition(_raw_precondition: &RCHash) -> Result<Precondition, ConfigError> {
    Ok(Precondition::new())
}
