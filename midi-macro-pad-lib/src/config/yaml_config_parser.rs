use std::str::FromStr;

use linked_hash_map::LinkedHashMap;

use crate::config::{ConfigError, ConfigInput, Loc};
use crate::config::raw_config::RawConfig;

use super::yaml_rust::{Yaml, YamlLoader};

pub struct YamlConfigParser<'a> {
    input: &'a str
}

impl YamlConfigParser<'_> {
    pub fn new(input: &str) -> YamlConfigParser {
        YamlConfigParser { input }
    }
}

impl ConfigInput for YamlConfigParser<'_> {
    fn as_raw_config(&self) -> Result<RawConfig, ConfigError> {
        let yaml_result = YamlLoader::load_from_str(self.input);

        if let Err(err) = yaml_result {
            return Err(ConfigError::FormatError(
                err.to_string(),
                Loc { line: err.marker().line(), col: err.marker().col() }
            ));
        }

        let mut yaml = yaml_result.unwrap();

        if yaml.is_empty() {
            return Ok(RawConfig::Null)
        }

        let yaml = yaml.swap_remove(0);

        Ok(yaml_to_rawconfig(yaml))
    }
}

fn yaml_to_rawconfig(yaml: Yaml) -> RawConfig {
    match yaml {
        Yaml::Real(real) => {
            let fl = f64::from_str(&real);

            if let Ok(fl) = fl {
                RawConfig::Integer(fl as i64)
            } else {
                RawConfig::Null
            }
        }

        Yaml::Integer(i) => RawConfig::Integer(i),
        Yaml::String(s) => RawConfig::String(s),
        Yaml::Boolean(b) => RawConfig::Bool(b),

        Yaml::Array(arr) => {
            let mut rc_arr: Vec<RawConfig> = vec![];

            for item in arr {
                rc_arr.push(yaml_to_rawconfig(item));
            }

            RawConfig::Array(rc_arr)
        }

        Yaml::Hash(hash) => {
            let mut rc_hash: LinkedHashMap<RawConfig, RawConfig> = LinkedHashMap::new();

            for (key, value) in hash {
                rc_hash.insert(yaml_to_rawconfig(key), yaml_to_rawconfig(value));
            }

            RawConfig::Hash(rc_hash)
        }

        Yaml::Alias(_) => RawConfig::Null,
        Yaml::Null => RawConfig::Null,
        Yaml::BadValue => RawConfig::Null
    }
}