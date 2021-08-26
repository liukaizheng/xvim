mod create;
mod events;
mod handler;
mod tx_wrapper;
mod ui_commands;

pub use events::*;
pub use handler::*;
pub use tx_wrapper::*;
pub use ui_commands::*;

use self::create::create_nvim_commad;
use crate::settings::SETTINGS;
use crate::{cmd_line::CmdLineSettings, logging_sender::LoggingUnboundedSender};
use log::{error, info, trace};
use nvim_rs::{UiAttachOptions, Value};
use std::sync::{atomic::AtomicBool, Arc};
use tokio::{runtime::Runtime, sync::mpsc::UnboundedReceiver};

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
    mut ui_command_receiver: UnboundedReceiver<UiCommand>,
    redraw_event_sender: LoggingUnboundedSender<RedrawEvent>,
    running: Arc<AtomicBool>,
) {
    let handler = NeovimHandler::new(ui_command_sender.clone(), redraw_event_sender.clone());
    let (nvim, io_handler) = create::new_child_cmd(&mut create_nvim_commad(), handler)
        .await
        .expect("Could not locate or start neovim process");
    if nvim.get_api_info().await.is_err() {
        error!("Cannot get neovim api info, either neovide is launched with an unknown command line option or neovim version not supported!");
    }
    let close_wathcer_running = running.clone();
    tokio::spawn(async move {
        info!("Close watcher started");
        match io_handler.await {
            Err(join_error) => error!("Error join IO Loop: '{}'", join_error),
            Ok(Err(error)) => {
                if !error.is_channel_closed() {
                    error!("Error: {}", error);
                }
            }
            Ok(Ok(())) => {}
        }
        close_wathcer_running.store(false, std::sync::atomic::Ordering::Relaxed);
    });

    match nvim.command_output("echo has('nvim-0.4')").await.as_deref() {
        Ok("1") => {}
        _ => {
            error!("Neovide requires nvim version 0.4 or higher. Download the latest version here https://github.com/neovim/neovim/wiki/Installing-Neovim");
            std::process::exit(0);
        }
    }

    nvim.set_var("xvim", Value::Boolean(true))
        .await
        .expect("Cound not communicate with neovim process");

    if let Err(command_error) = nvim.command("runtime! ginit.vim").await {
        nvim.command(&format!(
            "echomsg \"error encountered in ginit.vim {:?}\"",
            command_error
        ))
        .await
        .ok();
    }

    nvim.set_client_info(
        "xvim",
        vec![
            (Value::from("major"), Value::from(0u64)),
            (Value::from("minor"), Value::from(0u64)),
        ],
        "ui",
        vec![],
        vec![],
    )
    .await
    .ok();
    let xvim_channel = nvim
        .list_chans()
        .await
        .ok()
        .and_then(|channel_values| parse_channel_list(channel_values).ok())
        .and_then(|channel_list| {
            channel_list.iter().find_map(|channel| match channel {
                ChannelInfo {
                    id,
                    client: Some(ClientInfo { name, .. }),
                    ..
                } if name == "xvim" => Some(*id),
                _ => None,
            })
        })
        .unwrap_or(0);

    info!("Xvim registered to nvim with channel id {}", xvim_channel);

    nvim.set_option("lazydraw", Value::Boolean(false))
        .await
        .ok();
    nvim.set_option("termguicolors", Value::Boolean(true))
        .await
        .ok();

    let setting = SETTINGS.get::<CmdLineSettings>();
    let geometry = setting.geometry;
    let mut options = UiAttachOptions::new();
    options.set_linegrid_external(true);
    options.set_multigrid_external(setting.multi_grid);
    nvim.ui_attach(geometry.width as i64, geometry.height as i64, &options)
        .await
        .expect("Could not attach ui to neovim process");
    info!("Neovim process attached");
    let nvim = Arc::new(nvim);

    let ui_command_running = running.clone();
    let input_nvim = nvim.clone();
    tokio::spawn(async move {
        loop {
            if !ui_command_running.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }

            match ui_command_receiver.recv().await {
                Some(ui_commands) => {
                    let input_nvim = input_nvim.clone();
                    tokio::spawn(async move {
                        ui_commands.execute(&input_nvim).await;
                    });
                }
                None => {
                    ui_command_running.store(false, std::sync::atomic::Ordering::Relaxed);
                    trace!("stop execute ui_command");
                    break;
                }
            }
        }
    });
    SETTINGS.read_initial_values(&nvim).await;
    SETTINGS.setup_changed_listeners(&nvim).await;
}
