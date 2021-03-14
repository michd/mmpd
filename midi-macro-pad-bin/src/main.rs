mod tasks;
mod init;

#[macro_use]
extern crate clap;
use clap::App;

use tasks::*;

fn main() {
    const CMD_MONITOR: &str = "monitor";
    const CMD_LIST_MIDI_DEVICES: &str = "list-midi-devices";

    let cli_yaml = load_yaml!("cli.yml");
    let cli_matches = App::from_yaml(cli_yaml).get_matches();

    match cli_matches.subcommand_name() {
        Some(subcommand) => {
            let arg_matches = cli_matches.subcommand_matches(subcommand);

            match subcommand {
                CMD_MONITOR => task_monitor(arg_matches),
                CMD_LIST_MIDI_DEVICES => task_list_midi_devices(arg_matches),

                _ => {
                    // Will never execute, as only subcommands listed in cli.yml are included
                }
            }
        }

        None => task_main(Some(&cli_matches))
    }
}
