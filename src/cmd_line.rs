use clap::{App, Arg};

use crate::settings::SETTINGS;

#[derive(Clone, Debug)]
pub struct CmdLineSettings {
    pub verbosity: u64,
    pub log_to_file: bool,
    pub neovim_bin: Option<String>,
    pub neovim_args: Vec<String>,
    pub files_to_open: Vec<String>,
    pub frameless: bool,
}

impl Default for CmdLineSettings {
    fn default() -> Self {
        Self {
            verbosity: 0,
            log_to_file: false,
            neovim_bin: None,
            neovim_args: Vec::new(),
            files_to_open: Vec::new(),
            frameless: false,
        }
    }
}

pub fn handle_command_line_arguments() -> Result<(), String> {
    let clapp = App::new("Xvim")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .multiple(true)
                .help("Increas verbosity level (repeat up to 4 times: implies -nofork)"),
        )
        .arg(
            Arg::with_name("log_to_file")
                .long("log")
                .help("Log to a file"),
        )
        .arg(
            Arg::with_name("files")
                .multiple(true)
                .takes_value(true)
                .help("Files to open"),
        )
        .arg(
            Arg::with_name("xvim_args")
                .multiple(true)
                .takes_value(true)
                .last(true)
                .help("Specify arguments to pass down to xvim"),
        )
        .arg(
            Arg::with_name("frameless")
                .long("frameless")
                .help("Removes the window frame"),
        );

    let matches = clapp.get_matches();
    SETTINGS.set::<CmdLineSettings>(&CmdLineSettings {
        verbosity: matches.occurrences_of("verbosity"),
        log_to_file: matches.is_present("log_to_file"),
        files_to_open: matches
            .values_of("files")
            .map(|opt| opt.map(String::from).collect())
            .unwrap_or_default(),
        neovim_bin: std::env::var("XVIM_BIN").ok(),
        neovim_args: matches
            .values_of("neovim_args")
            .map(|opt| opt.map(String::from).collect())
            .unwrap_or_default(),
        frameless: matches.is_present("frameless") || std::env::var("XVIM_FRAMELESS").is_ok(),
    });
    Ok(())
}
