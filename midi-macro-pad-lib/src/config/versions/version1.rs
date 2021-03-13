mod primitive_matchers;
mod scope;
mod macros;
mod event_matchers;
mod precondition;
mod actions;

use crate::config::versions::ConfigVersionProcessor;
use crate::config::raw_config::{RCHash, AccessHelpers, RawConfig};
use crate::config::{ConfigError, Config};
use crate::config::versions::version1::scope::build_scope;
use crate::config::versions::version1::macros::build_scope_macros;

pub (crate) struct Version1Processor {
    // Ideas:
    // - Ability to specify how strict to be: If there's any error in trying to parse a
    //   data structure, continue by discarding it as null, perhaps adding a warning into
    //   a list kept in this struct. Otherwise (if strict mode) fail entire config load for
    //   any incorrect/missing data encountered
}

impl Version1Processor {
    /// Provides an instance of Version1Processor presented as "an implementation of
    /// `ConfigVersionProcessor`"
    pub (crate) fn new() -> Box<dyn ConfigVersionProcessor> {
        Box::new(Version1Processor {})
    }
}

impl ConfigVersionProcessor for Version1Processor {
    /// Processes a top level RCHash into a fully formed Config instances, or returns a ConfigError
    /// if something doesn't work out correctly.
    ///
    /// ## Notes on the version 1 format
    ///
    /// At the top level, there are 3 possible expected fields:
    /// - `scopes`:
    ///     Contains window class/name matching, as well as a list of macros that apply to that
    ///     scope. Note that in the parsed Config struct, this is organised differently; there is
    ///     one list of macros, each of which may or may not come with a scope. In the program it
    ///     is more practical that way, but in the context of authoring a configuration file, it
    ///     makes sense to specify a series of macros that apply to a given scope.
    /// - `global_macros`:
    ///     Contains all macros that apply regardless of focused window: macros without a scope.
    ///
    /// Further documentation and examples on the format can be found in /docs/config.md
    ///
    /// ## Arguments
    /// raw_config: Top level hash parsed from the config input file
    fn process(&self, raw_config: RCHash) -> Result<Config, ConfigError> {
        const SCOPES_FIELD: &str = "scopes";
        const MACROS_FIELD: &str = "macros";
        const GLOBAL_MACROS_FIELD: &str = "global_macros";

        let mut config = Config {
            macros: vec![]
        };

        if let Some(raw_scopes) = raw_config.get_array(SCOPES_FIELD) {
            for raw_scope in raw_scopes {
                if let RawConfig::Hash(raw_scope) = raw_scope {
                    let scope = build_scope(raw_scope)?;
                    if let None = scope { continue; }

                    let raw_macros = raw_scope.get_array(MACROS_FIELD);
                    if let None = raw_macros { continue; }

                    config.macros.extend(
                        build_scope_macros(
                            raw_macros.unwrap(),
                            Some(scope.unwrap())
                        )?
                    );
                }
            }
        }

        if let Some(raw_macros) = raw_config.get_array(GLOBAL_MACROS_FIELD) {
            config.macros.extend(build_scope_macros(raw_macros, None)?);
        }

        Ok(config)
    }
}
