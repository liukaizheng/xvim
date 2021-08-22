mod bridge;
mod cmd_line;
mod editor;
mod logging_sender;
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

use crate::{bridge::start_bridge, logging_sender::LoggingUnboundedSender};

fn main() {
    if let Err(err) = cmd_line::handle_command_line_arguments() {
        eprintln!("{}", err);
        return;
    }

    trace!("Xvim version: {}", crate_version!());

    #[cfg(target_os = "windows")]
    windows_fix_dpi();

    window::WindowSettings::register();

    let running = Arc::new(AtomicBool::new(true));

    let (redraw_event_sender, redraw_event_receiver) = unbounded_channel();
    let logging_redraw_event_sender =
        LoggingUnboundedSender::attach(redraw_event_sender, "redraw_event".to_owned());
    /* let (batched_draw_command_sender, batched_draw_command_receiver) = channel();
    let logging_batched_draw_command_sender = LoggingBoundedSender::attach(
        batched_draw_command_sender,
        "batched_draw_command".to_owned(),
    );*/
    let (ui_command_sender, ui_command_receiver) = unbounded_channel();
    let logging_ui_command_sender =
        LoggingUnboundedSender::attach(ui_command_sender, "ui_command".to_owned());

    let _bridge = start_bridge(
        logging_ui_command_sender,
        ui_command_receiver,
        logging_redraw_event_sender,
        running,
    );

}

#[cfg(target_os = "windows")]
fn windows_fix_dpi() {
    unsafe {
        winapi::um::winuser::SetProcessDpiAwarenessContext(
            winapi::shared::windef::DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        );
    }
}
