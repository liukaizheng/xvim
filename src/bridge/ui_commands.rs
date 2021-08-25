use super::tx_wrapper::TxWrapper;
use nvim_rs::Neovim;

#[derive(Debug, Clone)]
pub enum UiCommand {
    Quit,
    Resize { width: u64, height: u64 },
}

impl UiCommand {
    pub async fn execute(self, nvim: &Neovim<TxWrapper>) {
        match self {
            UiCommand::Quit => {
                nvim.command("qa!").await.ok();
            }
            UiCommand::Resize { width, height } => {
                nvim.ui_try_resize(width.max(10) as i64, height.max(3) as i64)
                    .await
                    .expect("Resize faild");
            }
        }
    }
}
