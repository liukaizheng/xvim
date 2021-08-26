mod renderer;

use std::{
    sync::{atomic::AtomicBool, mpsc::Receiver, Arc},
    time::{Duration, Instant},
};

use crate::settings::SETTINGS;
use crate::{
    bridge::UiCommand,
    cmd_line::CmdLineSettings,
    editor::{DrawCommand, WindowCommand},
    logging_sender::LoggingUnboundedSender,
    render::Render,
    window::WindowSettings,
};

use glutin::{
    self,
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    window::{self, Icon},
    ContextBuilder, WindowedContext,
};
use image::{load_from_memory, GenericImageView, Pixel};
use log::trace;

use self::renderer::SkiaRenderer;

static ICON: &[u8] = include_bytes!("../../../assets/xvim.ico");

pub struct GlutinWindowWrapper {
    windowed_context: WindowedContext<glutin::PossiblyCurrent>,
    skia_renderer: SkiaRenderer,
    render: Render,
    ui_command_sender: LoggingUnboundedSender<UiCommand>,
    window_command_receiver: Receiver<WindowCommand>,
    title: String,
}

impl GlutinWindowWrapper {
    fn handle_window_commands(&mut self) {
        let window_commands = self.window_command_receiver.try_iter().collect::<Vec<_>>();
        for command in window_commands {
            match command {
                WindowCommand::TitleChanged(new_title) => {
                    self.handle_title_changed(new_title);
                }
                WindowCommand::SetMouseEnable(_) => todo!(),
            }
        }
    }

    fn draw_frame(&mut self, _dt: f32) {}

    fn handle_title_changed(&mut self, new_title: String) {
        self.title = new_title;
        self.windowed_context.window().set_title(&self.title);
    }

    fn handle_event(&mut self, event: Event<()>, _running: &Arc<AtomicBool>) {
        match event {
            Event::NewEvents(_) => {}
            Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::Resized(_) => {
                    trace!("unhandled Resized event");
                }
                glutin::event::WindowEvent::Moved(_) => {
                    trace!("unhanded Moved event");
                }
                glutin::event::WindowEvent::CloseRequested
                | glutin::event::WindowEvent::Destroyed => {
                    self.ui_command_sender.send(UiCommand::Quit).ok().unwrap();
                }

                glutin::event::WindowEvent::DroppedFile(_) => {
                    trace!("unhandled DroppedFile event");
                }
                glutin::event::WindowEvent::HoveredFile(_) => {
                    trace!("unhandled HoverdFile event");
                }
                glutin::event::WindowEvent::HoveredFileCancelled => {
                    trace!("unhandled HoveredFileCancelled event");
                }
                glutin::event::WindowEvent::ReceivedCharacter(_) => {
                    trace!("undhandled ReceivedCharacter event");
                }
                glutin::event::WindowEvent::Focused(_) => {
                    trace!("unhandled Focus event");
                }
                glutin::event::WindowEvent::KeyboardInput {
                    device_id,
                    input,
                    is_synthetic,
                } => {
                    trace!("unhandled keyboard input");
                }
                glutin::event::WindowEvent::ModifiersChanged(_) => {
                    trace!("unhandled ModifiersChanged event");
                }
                glutin::event::WindowEvent::CursorMoved {
                    device_id,
                    position,
                    modifiers,
                } => {
                    trace!("unhandled cursormove event");
                }
                glutin::event::WindowEvent::CursorEntered { device_id } => {
                    trace!("unhandled cursorenter event");
                }
                glutin::event::WindowEvent::CursorLeft { device_id } => {
                    trace!("unhandled cursorleft event");
                }
                glutin::event::WindowEvent::MouseWheel {
                    device_id,
                    delta,
                    phase,
                    modifiers,
                } => {
                    trace!("unhandled mouseWheel event");
                }
                glutin::event::WindowEvent::MouseInput {
                    device_id,
                    state,
                    button,
                    modifiers,
                } => {
                    trace!("unhandled mouseInput event");
                }
                glutin::event::WindowEvent::TouchpadPressure {
                    device_id,
                    pressure,
                    stage,
                } => {
                    trace!("unhandled touchpadPressure");
                }
                glutin::event::WindowEvent::AxisMotion {
                    device_id,
                    axis,
                    value,
                } => {
                    trace!("unhandled axisMotion");
                }
                glutin::event::WindowEvent::Touch(_) => {
                    trace!("unhandled touch");
                }
                glutin::event::WindowEvent::ScaleFactorChanged {
                    scale_factor,
                    new_inner_size,
                } => {
                    trace!("unhandled scaleFactorChanged");
                }
                glutin::event::WindowEvent::ThemeChanged(_) => {
                    trace!("unhandled theme changed");
                }
            },
            Event::DeviceEvent { device_id, event } => {
                trace!("unhandled device event");
            }
            Event::UserEvent(_) => {
                trace!("unhandled user event");
            }
            Event::Suspended => {
                trace!("unhandled suspend event");
            }
            Event::Resumed => {
                trace!("unhandled resumed event");
            }
            Event::MainEventsCleared => {
                trace!("unhandled mainevent clear");
            }
            Event::RedrawRequested(_) => {
                trace!("unhandled redraw event");
            }
            Event::RedrawEventsCleared => {
                trace!("unhandled redrawEvent clear");
            }
            Event::LoopDestroyed => {
                trace!("unhandled loop destroyed event");
            }
            Event::DeviceEvent { device_id, event } => {
                trace!("unhandled device events");
            }
        }
    }
}

pub fn create_window(
    batched_draw_command_receiver: Receiver<Vec<DrawCommand>>,
    window_command_receiver: Receiver<WindowCommand>,
    ui_command_sender: LoggingUnboundedSender<UiCommand>,
    running: Arc<AtomicBool>,
) {
    let icon = {
        let icon = load_from_memory(ICON).expect("Failed to parse icon data");
        let (w, h) = icon.dimensions();
        let mut rgba = Vec::with_capacity((w * h) as usize * 4);
        for (_, _, pixel) in icon.pixels() {
            rgba.extend_from_slice(&pixel.to_rgba().0);
        }
        Icon::from_rgba(rgba, w, h).expect("Failed to create icon object")
    };
    let event_loop = EventLoop::new();
    let title = "Xvim".to_owned();
    let winit_window_builder = window::WindowBuilder::new()
        .with_title(&title)
        .with_window_icon(Some(icon))
        .with_maximized(SETTINGS.get::<CmdLineSettings>().maximized)
        .with_decorations(!SETTINGS.get::<CmdLineSettings>().frameless);
    let windowed_context = ContextBuilder::new()
        .with_pixel_format(24, 8)
        .with_stencil_buffer(8)
        .with_gl_profile(glutin::GlProfile::Core)
        .with_vsync(false)
        .with_srgb(false)
        .build_windowed(winit_window_builder, &event_loop)
        .unwrap();
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    let _window = windowed_context.window();

    let scale_factor = windowed_context.window().scale_factor();
    let render = Render::new(batched_draw_command_receiver, scale_factor);

    let skia_renderer = SkiaRenderer::new(&windowed_context);

    log::info!("window created (scale_factor: {:.4})", scale_factor);

    let mut window_wrapper = GlutinWindowWrapper {
        windowed_context,
        skia_renderer,
        render,
        ui_command_sender,
        window_command_receiver,
        title,
    };

    let mut previous_frame_start = Instant::now();
    event_loop.run(move |e, _window_target, control_flow| {
        if !running.load(std::sync::atomic::Ordering::Relaxed) {
            std::process::exit(0);
        }

        let frame_start = Instant::now();
        window_wrapper.handle_window_commands();
        window_wrapper.handle_event(e, &running);

        let refresh_rate = SETTINGS.get::<WindowSettings>().refresh_rate as f32;
        let expected_frame_length_seconds = 1.0 / refresh_rate;
        let frame_duration = Duration::from_secs_f32(expected_frame_length_seconds);
        if frame_start - previous_frame_start > frame_duration {
            let dt = previous_frame_start.elapsed().as_secs_f32();
            window_wrapper.draw_frame(dt);
            previous_frame_start = frame_start;
        }
        *control_flow = ControlFlow::WaitUntil(previous_frame_start + frame_duration)
    });
}
