use std::{
    io::{self, Error, ErrorKind},
    process::Stdio,
};

use nvim_rs::{error::LoopError, Handler, Neovim};
use tokio::{process::Command, task::JoinHandle};
use tokio_util::compat::TokioAsyncReadCompatExt;
use log::{warn, error, info};

use super::tx_wrapper::{TxWrapper, WrapTx};

use crate::{cmd_line::CmdLineSettings, settings::SETTINGS};

pub async fn new_child_cmd<H>(
    cmd: &mut Command,
    hander: H,
) -> io::Result<(Neovim<TxWrapper>, JoinHandle<Result<(), Box<LoopError>>>)>
where
    H: Handler<Writer = TxWrapper>,
{
    let mut child = cmd.stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| Error::new(ErrorKind::Other, "Can't open stdout"))?
        .compat();
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| Error::new(ErrorKind::Other, "Can't open stdint"))?
        .wrap_tx();
    let (neovim, io) = Neovim::<TxWrapper>::new(stdout, stdin, hander);
    let io_handle = tokio::task::spawn(io);
    Ok((neovim, io_handle))
}

pub fn create_nvim_commad() -> Command {
    let mut cmd = build_nvim_cmd();
    cmd.arg("--embed")
        .args(&SETTINGS.get::<CmdLineSettings>().neovim_args)
        .args(&SETTINGS.get::<CmdLineSettings>().files_to_open);
	info!("Starting neovim with: {:?}", cmd);

    #[cfg(not(debug_assertions))]
    cmd.stderr(Stdio::piped());

    #[cfg(debug_assertions)]
    cmd.stderr(Stdio::inherit());

    #[cfg(windows)]
    set_windows_creation_flags(&mut cmd);

    cmd
}

fn build_nvim_cmd() -> Command {
    if let Some(path) = SETTINGS.get::<CmdLineSettings>().neovim_bin {
        if let Some(cmd) = platform_build_nvim_cmd(&path) {
            return cmd;
        } else {
            warn!("NEOVIM_BIN is invalid falling back to first bin in PATH");
        }
    }

    if let Ok(path) = which::which("nvim") {
        if let Some(cmd) = platform_build_nvim_cmd(path.to_str().unwrap()) {
            cmd
        } else {
            error!("nvim does not have proper permissions!");
            std::process::exit(1);
        }
    } else {
        error!("nvim not found!");
        std::process::exit(1);
    }
}

#[cfg(windows)]
fn platform_build_nvim_cmd(bin: &str) -> Option<Command> {
    use std::path::Path;
    if Path::new(&bin).exists() {
        Some(Command::new(bin))
    } else {
        None
    }
}

#[cfg(windows)]
fn set_windows_creation_flags(cmd: &mut Command) {
    cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOWflags()
}