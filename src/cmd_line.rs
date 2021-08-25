use clap::{App, Arg};

use crate::settings::SETTINGS;
use crate::window::Dimensions;

#[derive(Clone, Debug)]
pub struct CmdLineSettings {
    pub verbosity: u64,
    pub log_to_file: bool,
    pub neovim_bin: Option<String>,
    pub neovim_args: Vec<String>,
    pub files_to_open: Vec<String>,
    pub frameless: bool,
    pub geometry: Dimensions,
    pub multi_grid: bool,
    pub maximized: bool,
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
            geometry: Dimensions::default(),
            multi_grid: false,
            maximized: false,
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
        )
        .arg(
            Arg::with_name("geometry")
                .long("geometry")
                .takes_value(true)
                .help("Specify the Geometry of the window"),
        )
        .arg(
            Arg::with_name("multi_grid")
                .long("multigrid")
                .help("Enable Multigrid"),
        )
        .arg(
            Arg::with_name("maximized")
                .long("maximized")
                .help("Maxmize the window"),
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
        geometry: parse_window_geometry(matches.value_of("geometry").map(|i| i.to_owned()))?,
        multi_grid: std::env::var("XVIM_MULTIGRID").is_ok() || matches.is_present("multi_grid"),
        maximized: std::env::var("XVIM_MAXIMIZED").is_ok() || matches.is_present("maximized"),
    });
    Ok(())
}

fn parse_window_geometry(geometry: Option<String>) -> Result<Dimensions, String> {
    let window_size = Ok(Dimensions::default());
    geometry.map_or(window_size, |input| {
        let invalid_parse_err = format!(
            "Invalid geometry: {}\n Valid format: <width>x<height>",
            input
        );
        input
            .split('x')
            .map(|dimension| {
                dimension
                    .parse::<u64>()
                    .map_err(|_| invalid_parse_err.as_str())
                    .and_then(|dimension| {
                        if dimension > 0 {
                            Ok(dimension)
                        } else {
                            Err("invalid geometry: should be greater than 0")
                        }
                    })
            })
            .collect::<Result<Vec<_>, &str>>()
            .and_then(|dimensions| {
                if let [width, height] = dimensions[..] {
                    Ok(Dimensions { width, height })
                } else {
                    Err(invalid_parse_err.as_str())
                }
            })
            .map_err(|msg| msg.to_owned())
    })
}
