pub mod yaml_config_parser;

extern crate yaml_rust;

use linked_hash_map::LinkedHashMap;
use crate::macros::{Scope, Macro};

/*

Design for configs:

- A trait: ConfigInput {
  as_raw_config(&self) -> RawConfig
}

- RawConfigValue = a generic enum that roughly matches the types in common with yaml or json:
    - Integer (not dealing with floats at this time, any float converted to int)
    - String
    - Boolean
    - Array (of more RawConfigValues)
    - Hash (LinkedHashMap from linked-hash-map crate
    - Null

 ---

 YamlConfigParser:

    loads config from string
    implements ConfigInput's as_raw_config (which clones any data)
 */

// Should be implemented by YamlConfigParser, JSONConfigParser, or what have you
trait ConfigInput {
    fn as_raw_config(&self) -> RawConfig;
}

#[derive(Debug)]
enum RawConfig {
    Null,
    Integer(i64),
    String(String),
    Boolean(bool),
    Array(Vec<RawConfig>),
    Hash(LinkedHashMap<RawConfig, RawConfig>),
}

pub struct Config<'a> {
    version: usize,
    scopes: Vec<Scope<'a>>,
    macros: Vec<Macro<'a>>
}

enum ConfigError {
    // TODO: as I implement parsing, figure out different sorts of errors
    InvalidConfig(String)
}

impl RawConfig {
    //pub fn process() -> Result<Config, ConfigError> {
        // TODO
    //}
}

