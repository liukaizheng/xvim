use skia_safe::{Canvas, Paint, colors};
use std::sync::Arc;
use crate::editor::{Colors, Style};

use super::fonts::caching_shaper::CachingShaper;

pub struct GridRender {
    pub shaper: CachingShaper,
    pub paint: Paint,
    pub default_style: Arc<Style>,
    pub scale_factor: f64,
}

impl GridRender {
    pub fn new(scale_factor: f64) -> Self {
        let shaper = CachingShaper::new(scale_factor as f32);
        let mut paint = Paint::new(colors::WHITE, None);
        paint.set_anti_alias(false);
        let default_style = Arc::new(Style::new(Colors::new(
            Some(colors::WHITE),
            Some(colors::BLACK),
            Some(colors::GREY),
        )));
        GridRender {
            shaper,
            paint,
            default_style,
            scale_factor,
        }
    }
}
