//! The Config module contains everything required to parse a config file (in YAML, perhaps later
//! other types) into a fully formed Config object, ready to be used by the main application.

use crate::macros::Macro;

pub mod raw_config;
pub mod input_formats;
pub mod versions;

/// Configuration owner used by the main program. An instance of this holds all data that gets
/// parsed from a configuration file into relevant data structures like `Macro`.
pub struct Config {
    /// List of macros specified in config file
    pub macros: Vec<Macro>
}

/// Represents an error that occurred while trying to load or parse configuration
pub enum ConfigError {
    /// The file has illegal syntax for the type it's trying to be parsed as.
    FormatError(
        /// Description of the error
        String,

        /// Location in the file where the error was encountered
        Loc
    ),

    /// The file specified a configuration version that is not implemented in this version of the
    /// program.
    UnsupportedVersion(
        /// Description of the error
        String),

    /// Missing or incorrect data encountered in the config file that meant part of it or the whole
    /// thing could not be parsed into a Config instance.
    InvalidConfig(
        /// Description of the error
        String
    )
}

impl ConfigError {
    /// Retrieves the error description as a string, regardless of which sub-type of error it is
    pub fn description(&self) -> String {
        match self {
            ConfigError::FormatError(desc, loc) => {
                format!("{} At line {}, column {}", desc, loc.line, loc.col)
            }

            ConfigError::UnsupportedVersion(desc) => desc.to_string(),
            ConfigError::InvalidConfig(desc) => desc.to_string(),
        }
    }
}

/// Location within a file expressed in line and column
pub struct Loc {
    pub line: usize,
    pub col: usize
}
