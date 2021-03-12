//! Intermediary config value format and tools to parse `RawConfig` into full formed `Config`

mod version1_processor;

use std::fmt::{self, Display, Formatter};
use linked_hash_map::LinkedHashMap;

use crate::config::{Config, ConfigError};
use crate::config::raw_config::version1_processor::Version1Processor;

/// The type `LinkedHashMap<RawConfig, RawConfig>` occurs a lot throughout the parsing code, so the
/// type alias makes it less clunky to work with.
type RCHash = LinkedHashMap<RawConfig, RawConfig>;

/// Intermediary type containing raw values from config files. File format parses such as
/// YamlConfigParser parse the configuration file into this format, and from there it is
/// further parsed into Config according to the specified configuration structure.
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum RawConfig {
    /// Represents the lack of a value, or some kind of value that isn't supported
    Null,

    /// Your usual integer. RawConfig does not support representing floats at this time (because
    /// there isn't a need for it); Any float formats from config files are cast to an integer,
    /// losing whatever they had after the decimal point.
    Integer(i64),

    /// String, owned by this type.
    String(String),

    Bool(bool),

    /// List of further RawConfig values, owned by this type.
    Array(Vec<RawConfig>),

    /// Hash / object / map of RawConfig to RawConfig. In practice, the key is always expected
    /// to be a String, but this need not be enforced when parsing into RawConfig.
    Hash(RCHash),
}

impl Display for RawConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let _ = match self {
            RawConfig::Null => write!(f, "Null"),
            RawConfig::Integer(i) => write!(f, "Integer({})", i),
            RawConfig::String(s) => write!(f, "String(\"{}\")", s),
            RawConfig::Bool(b) => write!(f, "Bool({})", b),
            RawConfig::Array(a) => write!(f, "Array(len: {})", a.len()),
            RawConfig::Hash(h) => write!(f, "Hash(len: {})", h.len())
        };

        fmt::Result::Ok(())
    }
}

/// Helpers to easily grab an optional value by a string key, makes the implementation a lot less
/// repetitive.
trait AccessHelpers {
    /// Returns an integer by given string key, if such an integer exists; `None` otherwise.
    fn get_integer(&self, key: &str) -> Option<i64>;


    /// Returns a string by given string key, if such a string exists; `None` otherwise.
    fn get_string(&self, key: &str) -> Option<&str>;

    /// Returns a boolean value by a given string key, if such a boolean value exists;
    /// `None` otherwise.
    fn get_bool(&self, key: &str) -> Option<bool>;

    /// Returns a `Vec` of further `RawConfig` values by a given string key, if such a `Vec` exists;
    /// `None` otherwise.
    fn get_array(&self, key: &str) -> Option<&Vec<RawConfig>>;

    /// Returns an `RCHash` by a given string key, if such an `RCHash` exists; `None` otherwise.
    fn get_hash(&self, key: &str) -> Option<&RCHash>;
}

/// Shorthand to build a `RawConfig`-wrapped string, used in accessing an object via a key
fn k(str_key: &str) -> RawConfig {
    RawConfig::String(str_key.to_string())
}

impl AccessHelpers for RCHash {

    /// Returns an `i64` by the given key, if it exists. If the key exists but doesn't yield
    /// `RawConfig::Integer`, also returns `None`.
    fn get_integer(&self, key: &str) -> Option<i64> {
        match self.get(&k(key))? {
            RawConfig::Integer(i) => Some(*i),
            _ => None
        }
    }

    /// Returns a `&str` by the given key, if it exists. If the key exists but doesn't yield
    /// `RawConfig::String`, also returns `None`.
    fn get_string(&self, key: &str) -> Option<&str> {
        match self.get(&k(key))? {
            RawConfig::String(s) => Some(s),
            _ => None
        }
    }

    /// Returns a `bool` by the given key, if it exists. If the key exists but doesn't yield
    /// `RawConfig::String`, also returns `None`.
    fn get_bool(&self, key: &str) -> Option<bool> {
        match self.get(&k(key))? {
            RawConfig::Bool(b) => Some(*b),
            _ => None
        }
    }

    /// Returns a `&Vec<RawConfig>` by the given key, if it exists. If the key exists but doesn't
    /// yield `RawConfig::Array`, also returns `None`.
    fn get_array(&self, key: &str) -> Option<&Vec<RawConfig>> {
        match self.get(&k(key))? {
            RawConfig::Array(v) => Some(v),
            _ => None
        }
    }

    // Returns an `&RCHash` by the given key, if it exists. If the key exists but doesn't yield
    // `RawConfig::Hash`, also returns `None`.
    fn get_hash(&self, key: &str) -> Option<&RCHash> {
        match self.get(&k(key))? {
            RawConfig::Hash(h) => Some(h),
            _ => None
        }
    }
}

/// A `ConfigProcessor` uses its own internal rules to parse an `RCHash` instance into an instance
/// of `Config`. Returns `Err(ConfigError)` if there is an error in parsing the data.
pub (crate) trait ConfigProcessor {
    fn process<'a>(&self, raw_config: RCHash) -> Result<Config, ConfigError>;
}

impl RawConfig {
    /// Processes the raw config into a `Config` instance.
    /// This method relies on the this value being a `RawConfig::Hash`, otherwise a `ConfigError` is
    /// returned. The `RCHash` contained in it must contain an integer "version" field, as this will
    /// determine how further parsing is handled. If the version number doesn't match one we have
    /// can provide a ConfigProcessor for, `ConfigError::UnsupportedVersion` will be returned.
    pub fn process<'a>(self) -> Result<Config, ConfigError> {
        const VERSION_FIELD: &str = "version";

        if let RawConfig::Hash(hash) = self {
            let version = hash.get_integer(VERSION_FIELD).ok_or_else(|| {
                ConfigError::InvalidConfig("Missing version field".to_string())
            })?;

            let processor = get_processor(version).ok_or_else(|| {
                ConfigError::UnsupportedVersion(format!("Unknown version {}", version))
            })?;

            processor.process(hash)
        } else {
            Err(ConfigError::InvalidConfig(
                format!("Top level of config should be Hash, found: {}", self)
            ))
        }
    }
}

/// Given a config format version number, returns a config processor implementation to parse
/// a RawConfig hash.
/// If the version is not supported, returns None.
fn get_processor(version: i64) -> Option<Box<dyn ConfigProcessor>> {
    match version {
        1 => Some(Version1Processor::new()),
        _ => None
    }
}
