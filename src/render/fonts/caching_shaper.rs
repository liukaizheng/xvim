use lru::LruCache;
use skia_safe::TextBlob;
use swash::shape::ShapeContext;

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
struct ShapeKey {
    pub cells: Vec<String>,
    pub bold: bool,
    pub italic: bool,
}

impl ShapeKey {
    pub fn new() -> Self {
        ShapeKey {
            cells: Vec::new(),
            bold: false,
            italic: false,
        }
    }
}

pub struct CachingShaper {
    blob_cache: LruCache<ShapeKey, Vec<TextBlob>>,
    shape_context: ShapeContext,
    scale_factor: f32,
}

impl CachingShaper {
    pub fn new(scale_factor: f32) -> Self {
        CachingShaper {
            blob_cache: LruCache::new(10000),
            shape_context: ShapeContext::new(),
            scale_factor,
        }
    }
}
