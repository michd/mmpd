pub (crate) mod midi_setup;

use clap::ArgMatches;

use directories::ProjectDirs;

use std::path::{PathBuf, Path};
use std::fs;

use mmpd_lib::config::Config;
use mmpd_lib::config::input_formats::get_parser_for_extension;

fn get_project_dir() -> Option<ProjectDirs> {
    ProjectDirs::from("me","michd", "mmpd")
}

fn get_default_config_file() -> Option<PathBuf> {
    const DEFAULT_FILENAMES: [&str; 2] = [
        "mmpd.yml",
        "mmpd.yaml"
    ];

    let config_dir = get_project_dir()
        .map(|pd| pd.config_dir().to_path_buf())
        .or_else(|| {
            eprintln!("Error: Couldn't determine default config directory");
            None
        })?;

    if !config_dir.exists() {
        return None;
    }

    let default_paths: Vec<PathBuf> =
        DEFAULT_FILENAMES
            .iter()
            .map(|filename| {
                config_dir
                    .join(Path::new(filename))
                    .to_path_buf()
            })
            .collect();

    default_paths.iter().find(|p| p.exists()).map_or_else(
        || {
            eprintln!("Error: No config file found in:");

            for path in &default_paths {
                eprintln!("\t{}", path.to_str().unwrap_or(""));
            }

            eprintln!("\nEither create one, or specify a config file with --config=<file>");
            None
        },
        |p| Some(p.to_path_buf())
    )
}

fn get_config_file(cli_matches: Option<&ArgMatches>) -> Option<PathBuf> {
    const CONFIG_PARAM: &str = "config";

    let cli_config =
        cli_matches
            .map(|cm| cm.value_of(CONFIG_PARAM))
            .flatten();

    if let Some(cli_config) = cli_config {
        // Config file specified as parameter
        let path = Path::new(cli_config);

        if path.exists() {
            Some(path.to_path_buf())
        } else {
            eprintln!("Config file not found: {}", cli_config);
            None
        }
    } else {
        // Config file from default config file location
        get_default_config_file()
    }
}

// Gets a config instance
pub (crate) fn get_config(cli_matches: Option<&ArgMatches>) -> Option<Config> {
    // Get configuration file
    let config_file = get_config_file(cli_matches)?;
    let config_file_name = config_file.to_str().unwrap_or("[none]");

    // Read config file to text
    let config_text = fs::read_to_string(&config_file).or_else(|read_err| {
        eprintln!("Unable to read config file {}", config_file_name);
        eprintln!("{}", read_err);
        Err(read_err)
    }).ok()?;

    // Get an appropriate parser based on the config file extension
    let ext = config_file.extension()
        .map(|s| s.to_str().unwrap_or("yml")).unwrap_or("yml");

    // Find a parser for this config file format
    let parser = get_parser_for_extension(ext).or_else(||{
        eprintln!("Error: unknown config file format {}", ext);
        None
    })?;

    // Parse configuration in RawConfig intermediary format
    let intermediate_config = parser.parse(&config_text).or_else(|parse_err| {
        eprintln!("Error: unable to parse config file {}", config_file_name);
        eprintln!("{}", parse_err.description());
        Err(parse_err)
    }).ok()?;

    // Process into Config object
    match intermediate_config.process() {
        Ok(config) => Some(config),

        Err(e) => {
            eprintln!("Error: unable to parse config file {}", config_file_name);
            eprintln!("{}", e.description());
            None
        }
    }
}

