use raw_config::RawConfig;

use crate::macros::Macro;

pub mod yaml_config_parser;
pub mod raw_config;

extern crate yaml_rust;

// Should be implemented by YamlConfigParser, JSONConfigParser, or what have you
pub trait ConfigInput {
    fn as_raw_config(&self) -> Result<RawConfig, ConfigError>;
}

pub struct Config {
    pub macros: Vec<Macro>
}

pub enum ConfigError {
    FormatError(String, Loc),
    UnsupportedVersion(String),
    InvalidConfig(String)
}

impl ConfigError {
    pub fn description(&self) -> String {
        String::from(match self {
            ConfigError::FormatError(desc, _loc) => {
                desc
                // TODO incorporate loc in description output
            }

            ConfigError::UnsupportedVersion(desc) => desc,
            ConfigError::InvalidConfig(desc) => desc,
        })
    }
}

pub struct Loc {
    pub line: usize,
    pub col: usize
}

