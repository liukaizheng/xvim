mod create;
mod events;
mod handler;
mod tx_wrapper;
mod ui_commands;

use crate::logging_sender::LoggingUnboundedSender;
pub use events::*;
use std::sync::{atomic::AtomicBool, Arc};
use tokio::{runtime::Runtime, sync::mpsc::UnboundedReceiver};
pub use ui_commands::*;

use handler::NeovimHandler;
use log::error;

use self::create::create_nvim_commad;

pub struct Bridge {
    _runtime: Runtime,
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
    Bridge { _runtime: runtime }
}

async fn start_neovim_runtime(
    ui_command_sender: LoggingUnboundedSender<UiCommand>,
    ui_command_receiver: UnboundedReceiver<UiCommand>,
    redraw_event_sender: LoggingUnboundedSender<RedrawEvent>,
    running: Arc<AtomicBool>,
) {
    let handler = NeovimHandler::new(ui_command_sender.clone(), redraw_event_sender.clone());
    let (mut nvim, io_handler) = create::new_child_cmd(&mut create_nvim_commad(), handler)
        .await
        .expect("Could not locate or start neovim process");
	if nvim.get_api_info().await.is_err() {
        error!("Cannot get neovim api info, either neovide is launched with an unknown command line option or neovim version not supported!");

    }
}
