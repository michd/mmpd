use crate::config::ConfigError;
use crate::config::raw_config::RawConfig;
use crate::config::input_formats::yaml_config_parser::YamlConfigInput;

pub (crate) mod yaml_config_parser;

/// "Low level" parsers that parse into RawConfig should implement this. For example,
/// YAML, TOML or JSON parsers. They parse into the intermediary RawConfig format, which
/// is then further processed into our relevant data structures.
pub trait ConfigInputParser {
    /// Attempts parsing the configuration file contents into the intermediary `RawConfig` format.
    /// If anything about it fails, returns a `ConfigError`.
    fn parse(&self, raw_input: &str) -> Result<RawConfig, ConfigError>;
}

pub fn get_parser_for_extension(ext: &str) -> Option<Box<dyn ConfigInputParser>> {
    let ext = ext.to_lowercase();

    match ext.as_ref() {
        "yml" | "yaml" => Some(YamlConfigInput::new()),
        _ => None
    }
}
