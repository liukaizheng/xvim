mod create;
mod events;
mod ui_commands;
mod handler;
mod tx_wrapper;

use crate::logging_sender::LoggingUnboundedSender;
pub use events::*;
use std::sync::{atomic::AtomicBool, Arc};
use tokio::{runtime::Runtime, sync::mpsc::UnboundedReceiver};
pub use ui_commands::*;

pub struct Bridge {
    runtime: Runtime,
}

pub fn start_bridge(
    ui_command_sender: LoggingUnboundedSender<UiCommand>,
    ui_command_receiver: UnboundedReceiver<UiCommand>,
    redraw_event_sender: LoggingUnboundedSender<RedrawEvent>,
    running: Arc<AtomicBool>,
) -> Bridge {
    let runtime = Runtime::new().unwrap();
    runtime.spawn(start_neovim_runtime(
        ui_command_sender,
        ui_command_receiver,
        redraw_event_sender,
        running,
    ));
    Bridge { runtime }
}

async fn start_neovim_runtime(
    ui_command_sender: LoggingUnboundedSender<UiCommand>,
    ui_command_receiver: UnboundedReceiver<UiCommand>,
    redraw_event_sender: LoggingUnboundedSender<RedrawEvent>,
    running: Arc<AtomicBool>,
) {

}
