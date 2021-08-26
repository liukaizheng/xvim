mod bridge;
mod cmd_line;
mod editor;
mod logging_sender;
mod render;
mod settings;
mod window;

#[macro_use]
extern crate xvim_derive;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;

use log::trace;
use std::sync::{atomic::AtomicBool, mpsc::channel, Arc};
use tokio::sync::mpsc::unbounded_channel;

use crate::cmd_line::*;
use crate::settings::*;
use crate::window::create_window;
use crate::{
    bridge::start_bridge,
    logging_sender::{LoggingBoundedSender, LoggingUnboundedSender},
};

#[cfg(not(test))]
use flexi_logger::{Cleanup, Criterion, Duplicate, FileSpec, Logger, Naming};

fn main() {
    if let Err(err) = cmd_line::handle_command_line_arguments() {
        eprintln!("{}", err);
        return;
    }

    #[cfg(not(test))]
    init_logger();

    trace!("Xvim version: {}", crate_version!());

    #[cfg(target_os = "windows")]
    windows_fix_dpi();

    window::WindowSettings::register();

    let running = Arc::new(AtomicBool::new(true));

    let (redraw_event_sender, redraw_event_receiver) = unbounded_channel();
    let logging_redraw_event_sender =
        LoggingUnboundedSender::attach(redraw_event_sender, "redraw_event".to_owned());
    let (batched_draw_command_sender, batched_draw_command_receiver) = channel();
    let logging_batched_draw_command_sender = LoggingBoundedSender::attach(
        batched_draw_command_sender,
        "batched_draw_command".to_owned(),
    );
    let (ui_command_sender, ui_command_receiver) = unbounded_channel();
    let logging_ui_command_sender =
        LoggingUnboundedSender::attach(ui_command_sender, "ui_command".to_owned());

    let (window_command_sender, window_command_receiver) = channel();
    let logging_window_command_sender =
        LoggingBoundedSender::attach(window_command_sender, "window_command".to_owned());

    let _bridge = start_bridge(
        logging_ui_command_sender.clone(),
        ui_command_receiver,
        logging_redraw_event_sender,
        running.clone(),
    );

    create_window(
        batched_draw_command_receiver,
        window_command_receiver,
        logging_ui_command_sender,
        running,
    );
}

#[cfg(not(test))]
pub fn init_logger() {
    let settings = SETTINGS.get::<CmdLineSettings>();

    let verbosity = match settings.verbosity {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };
    let logger = match settings.log_to_file {
        true => Logger::try_with_env_or_str("xvim")
            .unwrap()
            .duplicate_to_stderr(Duplicate::Error)
            .log_to_file(FileSpec::default())
            .rotate(
                Criterion::Size(10_000_000),
                Naming::Timestamps,
                Cleanup::KeepLogFiles(1),
            ),
        false => Logger::try_with_env_or_str(format!("neovide = {}", verbosity)).unwrap(),
    };
    logger.start().expect("Could not start logger");
}

#[cfg(target_os = "windows")]
fn windows_fix_dpi() {
    unsafe {
        winapi::um::winuser::SetProcessDpiAwarenessContext(
            winapi::shared::windef::DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        );
    }
}
