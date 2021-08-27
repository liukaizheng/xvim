use super::font_loader::FontSelection;

const DEFAULT_FONT_SIZE: f32 = 14.0;

#[derive(Debug, Clone)]
pub struct FontOptions {
    pub font_list: Vec<String>,
    pub size: f32,
    pub bold: bool,
    pub italic: bool,
}

impl FontOptions {
    pub fn parse(guifont_settins: &str) -> FontOptions {
        let mut font_list = Vec::new();
        let mut size = DEFAULT_FONT_SIZE;
        let mut bold = false;
        let mut italic = false;
        let mut parts = guifont_settins.split(':').filter(|part| !part.is_empty());
        if let Some(part) = parts.next() {
            font_list.extend(
                part.split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string()),
            );
        }

        for part in parts {
            if part.starts_with('h') && part.len() > 1 {
                if let Ok(parsed_size) = part[1..].parse::<f32>() {
                    size = parsed_size;
                } else if part == "b" {
                    bold = true;
                } else if part == "i" {
                    italic = true;
                }
            }
        }

        FontOptions {
            font_list,
            bold,
            italic,
            size: points_to_pixels(size),
        }
    }

    pub fn primary_font(&self) -> FontSelection {
        self.font_list
            .first()
            .map(FontSelection::from)
            .unwrap_or(FontSelection::Default)
    }
}

impl Default for FontOptions {
    fn default() -> Self {
        FontOptions {
            font_list: Vec::new(),
            bold: false,
            italic: false,
            size: points_to_pixels(DEFAULT_FONT_SIZE),
        }
    }
}

impl PartialEq for FontOptions {
    fn eq(&self, other: &Self) -> bool {
        self.font_list == other.font_list
            && (self.size - other.size).abs() < std::f32::EPSILON
            && self.bold == other.bold
            && self.italic == other.italic
    }
}

fn points_to_pixels(value: f32) -> f32 {
    if cfg!(target_os = "macos") {
        value
    } else {
        value * (96.0 / 72.0)
    }
}
