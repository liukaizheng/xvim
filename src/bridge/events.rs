#[derive(Clone, Debug)]
pub enum RedrawEvent {
    SetTitle { title: String },
}
