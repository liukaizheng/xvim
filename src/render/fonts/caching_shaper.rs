use log::trace;
use lru::LruCache;
use skia_safe::TextBlob;
use std::sync::Arc;
use swash::{
    shape::ShapeContext,
    text::{
        cluster::{CharCluster, Parser, Token},
        Script,
    },
    Metrics,
};
use unicode_segmentation::UnicodeSegmentation;

use super::{
    font_loader::{FontKey, FontLoader, FontPair, FontSelection},
    font_options::FontOptions,
};

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
    options: FontOptions,
    font_loader: FontLoader,
}

impl CachingShaper {
    pub fn new(scale_factor: f32) -> Self {
        let options = FontOptions::default();
        let font_size = options.size * scale_factor;
        CachingShaper {
            blob_cache: LruCache::new(10000),
            shape_context: ShapeContext::new(),
            scale_factor,
            options,
            font_loader: FontLoader::new(font_size),
        }
    }

    fn current_font_pair(&mut self) -> Arc<FontPair> {
        if let Some(font_par) = self.font_loader.get_or_load(&FontKey::from(&self.options)) {
            return font_par;
        }
        self.font_loader
            .get_or_load(&FontKey::default())
            .expect("Could not lad font")
    }

    pub fn current_size(&self) -> f32 {
        self.options.size * self.scale_factor
    }

    pub fn update_scale_factor(&mut self, scale_factor: f32) {
        trace!("scale_factor changed: {:.2}", scale_factor);
        self.scale_factor = scale_factor;
        self.reset_font_loader();
    }

    fn reset_font_loader(&mut self) {
        let font_size = self.options.size * self.scale_factor;
        trace!("Using font_size: {:.2}px", font_size);
        self.font_loader = FontLoader::new(font_size);
        self.blob_cache.clear();
    }

    pub fn update_font(&mut self, guifont_settins: &str) {
        trace!("Updating font: {}", guifont_settins);
        let options = FontOptions::parse(guifont_settins);
        let font_key = FontKey::from(&options);

        if self.font_loader.get_or_load(&font_key).is_some() {
            trace!("Font updated to: {}", guifont_settins);
            self.options = options;
            self.reset_font_loader();
        } else {
            trace!("Font can't be updated to: {}", guifont_settins);
        }
    }

    fn metrics(&mut self) -> Metrics {
        let font_pair = self.current_font_pair();
        let size = self.current_size();
        let shaper = self
            .shape_context
            .builder(font_pair.swash_font.as_ref())
            .size(size)
            .build();
        shaper.metrics()
    }

    pub fn font_base_dimensions(&mut self) -> (f32, f32) {
        let metrics = self.metrics();
        let font_height = metrics.ascent + metrics.descent + metrics.leading;
        (metrics.average_width, font_height)
    }

    fn build_clusters(
        &mut self,
        text: &str,
        bold: bool,
        italic: bool,
    ) -> Vec<(Vec<CharCluster>, Arc<FontPair>)> {
        let mut cluster = CharCluster::new();
        let mut character_index = 0;
        let mut parser = Parser::new(
            Script::Latin,
            text.graphemes(true)
                .enumerate()
                .map(|(glyph_index, unicode_segmentation)| {
                    unicode_segmentation.chars().map(move |c| {
                        let token = Token {
                            ch: c,
                            offset: character_index as u32,
                            len: c.len_utf8() as u8,
                            info: c.into(),
                            data: glyph_index as u32,
                        };
                        character_index += 1;
                        token
                    })
                })
                .flatten(),
        );
        let mut font_fallback_keys = vec![FontKey {
            italic: self.options.italic || italic,
            bold: self.options.bold || bold,
            font_selection: FontSelection::Default,
        }];
        font_fallback_keys.extend(self.options.font_list.iter().map(|font_name| FontKey {
            italic: self.options.italic || italic,
            bold: self.options.bold || bold,
            font_selection: font_name.into(),
        }));
        let mut results = Vec::new();

        'cluster: while parser.next(&mut cluster) {
            font_fallback_keys.push(FontKey {
                italic,
                bold,
                font_selection: cluster.chars()[0].ch.into(),
            });
            let mut best = None;
            for fallback_key in font_fallback_keys.iter().rev() {
                if let Some(font_pair) = self.font_loader.get_or_load(fallback_key) {
                    let charmap = font_pair.swash_font.as_ref().charmap();
                    match cluster.map(|ch| charmap.map(ch)) {
                        swash::text::cluster::Status::Complete => {
                            results.push((cluster.to_owned(), font_pair.clone()));
                            continue 'cluster;
                        }
                        swash::text::cluster::Status::Keep => {
                            best = Some(font_pair);
                        }
                        swash::text::cluster::Status::Discard => {}
                    }
                }
            }
            if let Some(best) = best {
                results.push((cluster.to_owned(), best.clone()));
            }
            font_fallback_keys.pop();
        }

        let mut grouped_results = Vec::new();
        let mut current_group = Vec::new();
        let mut current_font_option = None;
        for (cluster, font) in results {
            if let Some(current_font) = current_font_option.clone() {
                if current_font == font {
                    current_group.push(cluster);
                } else {
                    grouped_results.push((current_group, current_font));
                    current_group = vec![cluster];
                    current_font_option = Some(font);
                }
            } else {
                current_group = vec![cluster];
                current_font_option = Some(font);
            }
        }
        if !current_group.is_empty() {
            grouped_results.push((current_group, current_font_option.unwrap()));
        }
        grouped_results
    }

    /*pub fn shape(&mut self, cells: &[String], bold: bool, italic: bool) -> Vec<TextBlob> {
        let mut resulting_blobs = Vec::new();
        let current_size = self.current_size();
        let text = cells.concat();
        trace!("Shaping text: {}", text);
        for (cluster_group, font_pair) in self.build_clusters(&text, bold, italic) {
            let mut shaper = self
                .shape_context
                .builder(font_pair.swash_font.as_ref())
                .size(current_size)
                .build();

        }
    }*/
}
