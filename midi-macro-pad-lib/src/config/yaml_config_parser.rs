use crate::config::{ConfigInput, RawConfig};

struct YamlConfigParser<'a> {
    input: &'a str
}

impl YamlConfigParser {
    pub fn new(input: &str) -> YamlConfigParser {
        YamlConfigParser { input }
    }
}

impl ConfigInput for YamlConfigParser {
    fn as_raw_config(&self) -> RawConfig {
        // TODO
        RawConfig::Null
    }
}