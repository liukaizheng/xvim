use std::sync::Arc;

use async_trait::async_trait;
use nvim_rs::Handler;
use parking_lot::Mutex;
use tokio::task;

use log::trace;

use crate::settings::SETTINGS;
use crate::{bridge::parse_redraw_event, logging_sender::LoggingUnboundedSender};

use super::{tx_wrapper::TxWrapper, RedrawEvent, UiCommand};

#[derive(Clone)]
pub struct NeovimHandler {
    ui_command_sender: Arc<Mutex<LoggingUnboundedSender<UiCommand>>>,
    redraw_event_sender: Arc<Mutex<LoggingUnboundedSender<RedrawEvent>>>,
}

impl NeovimHandler {
    pub fn new(
        ui_command_sender: LoggingUnboundedSender<UiCommand>,
        redraw_event_sender: LoggingUnboundedSender<RedrawEvent>,
    ) -> NeovimHandler {
        NeovimHandler {
            ui_command_sender: Arc::new(Mutex::new(ui_command_sender)),
            redraw_event_sender: Arc::new(Mutex::new(redraw_event_sender)),
        }
    }
}

#[async_trait]
impl Handler for NeovimHandler {
    type Writer = TxWrapper;

    async fn handle_notify(
        &self,
        event_name: String,
        args: Vec<nvim_rs::Value>,
        _neovim: nvim_rs::Neovim<TxWrapper>,
    ) {
        trace!("Neovim nofification: {:?}", &event_name);
        let redraw_event_sender = self.redraw_event_sender.clone();
        task::spawn_blocking(move || match event_name.as_ref() {
            "redraw" => {
                for events in args {
                    let parsed_events = parse_redraw_event(events).expect("parsed redraw event");
                    for parsed_event in parsed_events {
                        let redraw_event_sender = redraw_event_sender.lock();
                        redraw_event_sender.send(parsed_event).ok();
                    }
                }
            }
            "setting_changed" => {
                SETTINGS.handle_changed_notification(args);
            }
            _ => {}
        })
        .await
        .ok();
    }
}
