use crate::cmd_line::CmdLineSettings;
use crate::settings::*;

#[derive(Clone, SettingGroup)]
pub struct WindowSettings {
    pub refresh_rate: u64,
    pub no_idle: bool,
    pub transparency: f32,
    pub fullscreen: bool,
    pub remember_window_size: bool,
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            refresh_rate: 60,
            no_idle: SETTINGS
                .get::<CmdLineSettings>()
                .neovim_args
                .contains(&"--noIdle".to_owned()),
            transparency: 1.0,
            fullscreen: false,
            remember_window_size: false,
        }
    }
}

#[derive(Clone, Default, SettingGroup)]
#[setting_prefix = "input"]
pub struct KeyboardSettings {
    pub use_logo: bool,
}
