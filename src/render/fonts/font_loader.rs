use super::swash_font::SwashFont;
use lru::LruCache;
use skia_safe::{Font, FontMgr};
use std::sync::Arc;

pub struct FontPair {
    pub skia_font: Font,
    pub swash_font: SwashFont,
}

impl FontPair {
    fn new(mut skia_font: Font) -> Option<FontPair> {
        skia_font.set_subpixel(true);
        skia_font.set_hinting(skia_safe::FontHinting::Full);
        skia_font.set_edging(skia_safe::font::Edging::SubpixelAntiAlias);

        let (font_data, index) = skia_font.typeface().unwrap().to_font_data().unwrap();
        let swash_font = SwashFont::from_data(font_data, index)?;
        Some(Self {
            skia_font,
            swash_font,
        })
    }
}

impl PartialEq for FontPair {
    fn eq(&self, other: &Self) -> bool {
        self.swash_font.key == other.swash_font.key
    }
}

pub struct FontLoader {
    font_mgr: FontMgr,
    cache: LruCache<FontKey, Arc<FontPair>>,
    font_size: f32,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct FontKey {
    pub bold: bool,
    pub italic: bool,
    pub font_selection: FontSelection,
}

impl Default for FontKey {
    fn default() -> Self {
        FontKey {
            bold: false,
            italic: false,
            font_selection: FontSelection::LastResort,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum FontSelection {
    Name(String),
    Character(char),
    LastResort,
}
