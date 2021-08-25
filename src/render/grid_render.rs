pub struct GridRender {
    pub scale_factor: f64,
}

impl GridRender {
    pub fn new(scale_factor: f64) -> Self {
        GridRender { scale_factor }
    }
}
