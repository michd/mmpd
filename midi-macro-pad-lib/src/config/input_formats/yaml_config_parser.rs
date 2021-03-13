//! Parses YAML string into YAML representation tree, into `RawConfig` tree

use std::str::FromStr;

use yaml_rust::{Yaml, YamlLoader};

use crate::config::{ConfigError, Loc};
use crate::config::input_formats::ConfigInputParser;
use crate::config::raw_config::RawConfig;

extern crate yaml_rust;

/// Config input parser, parsing YAML into `RawConfig`.
pub struct YamlConfigInput { }

impl YamlConfigInput {
    /// Creates a new parser instance from the full text contents of the configuration file
    pub fn new() -> Box<dyn ConfigInputParser> {
        Box::new(YamlConfigInput {})
    }
}

impl ConfigInputParser for YamlConfigInput {
    /// Attempts parsing the YAML file contents into the intermediary `RawConfig` format.
    /// If anything about it fails, returns a `ConfigError`.
    fn parse(&self, raw_input: &str) -> Result<RawConfig, ConfigError> {
        let mut yaml = YamlLoader::load_from_str(raw_input).map_err(|err| {
           ConfigError::FormatError(
               err.to_string(),
               Loc { line: err.marker().line(), col: err.marker().col() }
           )
        })?;

        Ok(if yaml.is_empty() {
            RawConfig::Null
        } else {
            yaml_to_raw_config(&yaml.swap_remove(0))
        })
    }
}

fn yaml_to_raw_config(yaml: &Yaml) -> RawConfig {
    match yaml {
        Yaml::Real(real) => f64::from_str(real)
            .map_or(
                RawConfig::Null,
                |f| RawConfig::Integer(f as i64)
            ),

        Yaml::Integer(i) => RawConfig::Integer(*i),
        Yaml::String(s) => RawConfig::String(s.to_string()),
        Yaml::Boolean(b) => RawConfig::Bool(*b),

        Yaml::Array(arr) => RawConfig::Array(
            arr
                .iter()
                .map( yaml_to_raw_config)
                .collect()
        ),

        Yaml::Hash(hash) => RawConfig::Hash(
            hash
                .iter()
                .map(|(key, value)| {
                    (yaml_to_raw_config(key), yaml_to_raw_config(value))
                })
                .collect()
        ),

        Yaml::Alias(_) => RawConfig::Null,
        Yaml::Null => RawConfig::Null,
        Yaml::BadValue => RawConfig::Null
    }
}