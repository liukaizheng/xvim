use super::{font_options::FontOptions, swash_font::SwashFont};
use lru::LruCache;
use skia_safe::{Data, Font, FontMgr, FontStyle, Typeface};
use std::sync::Arc;

static DEFAULT_FONT: &[u8] = include_bytes!("../../../assets/fonts/FiraCode-Regular.ttf");

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

impl FontLoader {
    pub fn new(font_size: f32) -> FontLoader {
        FontLoader {
            font_mgr: FontMgr::new(),
            cache: LruCache::new(10),
            font_size,
        }
    }
    pub fn get_or_load(&mut self, font_key: &FontKey) -> Option<Arc<FontPair>> {
        if let Some(cached) = self.cache.get(font_key) {
            return Some(cached.clone());
        }
        let load_font = self.load(font_key.clone())?;
        let font_arc = Arc::new(load_font);
        self.cache.put(font_key.clone(), font_arc.clone());
        Some(font_arc)
    }

    fn load(&mut self, font_key: FontKey) -> Option<FontPair> {
        let font_style = match (font_key.bold, font_key.italic) {
            (true, true) => FontStyle::bold_italic(),
            (false, true) => FontStyle::italic(),
            (true, false) => FontStyle::bold(),
            (false, false) => FontStyle::normal(),
        };
        match font_key.font_selection {
            FontSelection::Name(name) => {
                let typeface = self.font_mgr.match_family_style(name, font_style)?;
                FontPair::new(Font::from_typeface(typeface, self.font_size))
            }
            FontSelection::Character(character) => {
                let typeface = self.font_mgr.match_family_style_character(
                    "",
                    font_style,
                    &[],
                    character as i32,
                )?;
                FontPair::new(Font::from_typeface(typeface, self.font_size))
            }
            FontSelection::Default => {
                let data = Data::new_copy(DEFAULT_FONT);
                let typeface = Typeface::from_data(data, 0).unwrap();
                FontPair::new(Font::from_typeface(typeface, self.font_size))
            }
        }
    }
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
            font_selection: FontSelection::Default,
        }
    }
}

impl From<&FontOptions> for FontKey {
    fn from(options: &FontOptions) -> FontKey {
        FontKey {
            italic: options.italic,
            bold: options.bold,
            font_selection: options.primary_font(),
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum FontSelection {
    Name(String),
    Character(char),
    Default,
}

impl<T: ToString> From<&T> for FontSelection {
    fn from(t: &T) -> Self {
        FontSelection::Name(t.to_string())
    }
}

impl From<String> for FontSelection {
    fn from(t: String) -> Self {
        FontSelection::Name(t)
    }
}

impl From<char> for FontSelection {
    fn from(c: char) -> Self {
        FontSelection::Character(c)
    }
}
