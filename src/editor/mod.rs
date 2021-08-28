mod style;
mod window;
mod draw_command_batcher;
mod cursor;


pub use style::*;
use window::*;
use draw_command_batcher::*;
pub use cursor::*;

use log::debug;
use std::collections::HashMap;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{bridge::RedrawEvent, logging_sender::LoggingBoundedSender};

#[derive(Debug)]
pub enum DrawCommand {
    CloseWindow(u64),
}

#[derive(Debug)]
pub enum WindowCommand {
    TitleChanged(String),
    SetMouseEnable(bool),
}

struct Editor {
    pub windows: HashMap<u64, Window>,
    pub draw_command_batcher: DrawCommandBatcher,
    pub window_command_sender: LoggingBoundedSender<WindowCommand>,
}

impl Editor {
    pub fn new(
        batched_draw_command_sender: LoggingBoundedSender<Vec<DrawCommand>>,
        window_command_sender: LoggingBoundedSender<WindowCommand>,
    ) -> Self {
        Self {
            windows: HashMap::new(),
            draw_command_batcher: DrawCommandBatcher::new(batched_draw_command_sender),
            window_command_sender,
        }
    }

    pub fn handle_redraw_event(&mut self, event: RedrawEvent) {
        match event {
            RedrawEvent::SetTitle { title } => {
                self.window_command_sender
                    .send(WindowCommand::TitleChanged(title))
                    .ok();
            }
            _ => {
                debug!("unhandled event {:?}", event);
            }
        }
    }
}

pub fn start_editor(
    mut redraw_event_receiver: UnboundedReceiver<RedrawEvent>,
    batched_draw_command_sender: LoggingBoundedSender<Vec<DrawCommand>>,
    window_command_sender: LoggingBoundedSender<WindowCommand>,
) {
    std::thread::spawn(move || {
        let mut editor = Editor::new(batched_draw_command_sender, window_command_sender);
        while let Some(redraw_event) = redraw_event_receiver.blocking_recv() {
            editor.handle_redraw_event(redraw_event);
        }
    });
}

