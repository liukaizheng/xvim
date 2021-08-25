use std::sync::mpsc::Receiver;

mod grid_render;

use crate::editor::DrawCommand;

use self::grid_render::GridRender;

pub struct Render {
    pub batched_draw_command_receiver: Receiver<Vec<DrawCommand>>,
    pub grid_render: GridRender,
}

impl Render {
    pub fn new(
        batched_draw_command_receiver: Receiver<Vec<DrawCommand>>,
        scale_factor: f64,
    ) -> Self {
        Render {
            batched_draw_command_receiver,
            grid_render: GridRender::new(scale_factor),
        }
    }
}
