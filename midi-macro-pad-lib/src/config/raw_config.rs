mod version1_processor;

use std::fmt::{Display, Formatter};
use std::fmt;

use linked_hash_map::LinkedHashMap;

use crate::config::{Config, ConfigError};
use crate::config::raw_config::version1_processor::Version1Processor;

type RCHash = LinkedHashMap<RawConfig, RawConfig>;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum RawConfig {
    Null,
    Integer(i64),
    String(String),
    Bool(bool),
    Array(Vec<RawConfig>),
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

trait AccessHelpers {
    fn get_integer(&self, key: &str) -> Option<i64>;
    fn get_string(&self, key: &str) -> Option<&str>;
    fn get_bool(&self, key: &str) -> Option<bool>;
    fn get_array(&self, key: &str) -> Option<&Vec<RawConfig>>;
    fn get_hash(&self, key: &str) -> Option<&RCHash>;
}


// Lil' shorthand
fn k(str_key: &str) -> RawConfig {
    RawConfig::String(str_key.to_string())
}

impl AccessHelpers for RCHash {
    fn get_integer(&self, key: &str) -> Option<i64> {
        match self.get(&k(key))? {
            RawConfig::Integer(i) => Some(*i),
            _ => None
        }
    }

    fn get_string(&self, key: &str) -> Option<&str> {
        match self.get(&k(key))? {
            RawConfig::String(s) => Some(s),
            _ => None
        }
    }

    fn get_bool(&self, key: &str) -> Option<bool> {
        match self.get(&k(key))? {
            RawConfig::Bool(b) => Some(*b),
            _ => None
        }
    }

    fn get_array(&self, key: &str) -> Option<&Vec<RawConfig>> {
        match self.get(&k(key))? {
            RawConfig::Array(v) => Some(v),
            _ => None
        }
    }

    fn get_hash(&self, key: &str) -> Option<&RCHash> {
        match self.get(&k(key))? {
            RawConfig::Hash(h) => Some(h),
            _ => None
        }
    }
}

pub (crate) trait ConfigProcessor {
    fn process<'a>(&self, raw_config: RCHash) -> Result<Config, ConfigError>;
}

impl RawConfig {
    pub fn process<'a>(self) -> Result<Config, ConfigError> {
        if let RawConfig::Hash(hash) = self {
            if let Some(version) = hash.get_integer("version") {
                let processor = get_processor(version);

                if let None = processor {
                    return Err(ConfigError::UnsupportedVersion(
                        format!("Unknown version {}", version)
                    ))
                }
                println!("version: {}", version);

                let processor = processor.unwrap();

                processor.process(hash)
            } else {
                Err(ConfigError::InvalidConfig("Missing version field".to_string()))
            }
        } else {
            Err(ConfigError::InvalidConfig(
                format!("Top level of config should Hash, found: {}", self)
            ))
        }
    }
}

fn get_processor(version: i64) -> Option<Box<dyn ConfigProcessor>> {
    match version {
        1 => Some(Version1Processor::new()),
        _ => None
    }
}
