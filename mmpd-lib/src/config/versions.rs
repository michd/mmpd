mod version1;

use crate::config::{Config, ConfigError};
use crate::config::raw_config::RCHash;
use crate::config::versions::version1::Version1Processor;

/// A `ConfigVersionProcessor` uses its own internal rules to parse an `RCHash` instance into an
/// instance of `Config`.
///
/// Returns `Err(ConfigError)` if there is any error in parsing the data.
pub trait ConfigVersionProcessor {
    fn process(&self, raw_config: RCHash) -> Result<Config, ConfigError>;
}

/// Given a config format version number, returns a config processor implementation to parse
/// a RawConfig hash.
/// If the version is not supported, returns None.
pub fn get_processor(version: i64) -> Option<Box<dyn ConfigVersionProcessor>> {
    match version {
        1 => Some(Version1Processor::new()),
        _ => None
    }
}
