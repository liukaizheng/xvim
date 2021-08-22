mod cmd_line;
mod settings;
mod window;

#[macro_use]
extern crate xvim_derive;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;

use cmd_line::CmdLineSettings;
use log::trace;
use settings::SETTINGS;

fn main() {
    if let Err(err) = cmd_line::handle_command_line_arguments() {
        eprintln!("{}", err);
        return;
    }

    trace!("Xvim version: {}", crate_version!());

    #[cfg(target_os = "windows")]
    windows_fix_dpi();

    window::WindowSettings::register();
}

#[cfg(target_os = "windows")]
fn windows_fix_dpi() {
    unsafe {
        winapi::um::winuser::SetProcessDpiAwarenessContext(
            winapi::shared::windef::DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        );
    }
}
