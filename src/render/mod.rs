mod fonts;
mod grid_render;

use std::sync::mpsc::Receiver;

use skia_safe::Canvas;

use crate::editor::DrawCommand;

use self::grid_render::GridRender;

pub struct Render {
    pub batched_draw_command_receiver: Receiver<Vec<DrawCommand>>,
    pub grid_render: GridRender,
    font_changed: bool,
}

impl Render {
    pub fn new(
        batched_draw_command_receiver: Receiver<Vec<DrawCommand>>,
        scale_factor: f64,
    ) -> Self {
        Render {
            batched_draw_command_receiver,
            grid_render: GridRender::new(scale_factor),
            font_changed: false,
        }
    }

    pub fn draw_frame(&mut self, root_canvas: &mut Canvas, dt: f32) -> bool {
        let draw_commands = self
            .batched_draw_command_receiver
            .try_iter()
            .map(|batch| batch.into_iter())
            .flatten()
            .collect::<Vec<_>>();
        self.font_changed = false;
        for draw_command in draw_commands {
            self.handle_draw_command(root_canvas, draw_command);
        }
        self.font_changed
    }

    fn handle_draw_command(&mut self, _root_canvas: &mut Canvas, draw_command: DrawCommand) {
        match draw_command {
            _ => {}
        }
    }
}
