#[derive(Debug)]
pub enum DrawCommand {
    CloseWindow(u64),
}

#[derive(Debug)]
pub enum WindowCommand {
    TitleChanged(String),
    SetMouseEnable(bool),
}
