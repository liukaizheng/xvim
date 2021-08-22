#[derive(Debug, Clone)]
pub enum UiCommand {
    Quit,
    Resize { width: u64, height: u64 },
}
