use std::{cell::RefCell, sync::mpsc::SendError};

use crate::logging_sender::LoggingBoundedSender;

use super::DrawCommand;

pub struct DrawCommandBatcher {
    batched_draw_commander: LoggingBoundedSender<Vec<DrawCommand>>,
    draw_comands: RefCell<Vec<DrawCommand>>,
}

impl DrawCommandBatcher {
    pub fn new(batched_draw_commander: LoggingBoundedSender<Vec<DrawCommand>>) -> Self {
        Self {
            batched_draw_commander,
            draw_comands: RefCell::new(Vec::new()),
        }
    }

    pub fn queue(&self, draw_comand: DrawCommand) -> Result<(), SendError<Vec<DrawCommand>>> {
        self.draw_comands.borrow_mut().push(draw_comand);
        let commands = self.draw_comands.replace(Vec::new());
        if !commands.is_empty() {
            self.batched_draw_commander.send(commands)
        } else {
            Ok(())
        }
    }

}
