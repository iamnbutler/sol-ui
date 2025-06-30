//! Font management using font-kit

use font_kit::canvas::{Canvas, Format, RasterizationOptions};
use font_kit::font::Font as FontKitFont;
use font_kit::hinting::HintingOptions;
use font_kit::source::SystemSource;
use pathfinder_geometry::rect::RectI;
use pathfinder_geometry::transform2d::Transform2F;
use pathfinder_geometry::vector::{Vector2F, Vector2I};
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(target_os = "macos")]
use core_text::font::CTFont;

use crate::text::RasterizedGlyph;

/// A font handle
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FontSpec {
    /// Font family name
    pub family: String,
    /// Font style
    pub style: FontStyle,
}

impl FontSpec {
    pub fn new(family: impl Into<String>, style: FontStyle) -> Self {
        Self {
            family: family.into(),
            style,
        }
    }

    pub fn system_ui() -> Self {
        Self {
            family: "Helvetica".to_string(),
            style: FontStyle::Normal,
        }
    }
}

impl Default for FontSpec {
    fn default() -> Self {
        Self::system_ui()
    }
}

/// Font style variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontStyle {
    Normal,
    Bold,
    Italic,
    BoldItalic,
}

/// Font weight for more granular control
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontWeight {
    Thin,
    Light,
    Regular,
    Medium,
    Bold,
    Black,
}

/// Font metrics information
#[derive(Debug, Clone, Copy)]
pub struct FontMetrics {
    /// The ascent of the font (distance from baseline to top)
    pub ascent: f32,
    /// The descent of the font (distance from baseline to bottom)
    pub descent: f32,
    /// The leading of the font (extra space between lines)
    pub leading: f32,
    /// The cap height of the font
    pub cap_height: f32,
    /// The x-height of the font
    pub x_height: f32,
}

/// Internal font data
/// Font data stored in the manager
struct FontData {
    font: FontKitFont,
    #[cfg(target_os = "macos")]
    native_font: CTFont,
    font_id: u32,
}

/// Font manager that handles Core Text fonts
/// Font manager using font-kit
pub struct FontManager {
    fonts: HashMap<(FontSpec, u32), Arc<FontData>>,
    next_font_id: u32,
    system_source: SystemSource,
}

impl FontManager {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            fonts: HashMap::new(),
            next_font_id: 1,
            system_source: SystemSource::new(),
        })
    }

    /// Load a font with the given specification
    pub fn load_font(&mut self, font_spec: &FontSpec, size: f32) -> Result<u32, String> {
        let key = (font_spec.clone(), size as u32);

        if let Some(font_data) = self.fonts.get(&key) {
            return Ok(font_data.font_id);
        }

        // Load font using font-kit
        let font = self.load_font_from_spec(font_spec)?;
        let font_id = self.next_font_id;
        self.next_font_id += 1;

        #[cfg(target_os = "macos")]
        let native_font = {
            // Create a CTFont from the font specification for text shaping
            use core_text::font as ct_font;
            // Try to match the font-kit font with a Core Text font
            // IMPORTANT: We should use the same font that font-kit loaded!
            let postscript_name = font
                .postscript_name()
                .unwrap_or_else(|| font_spec.family.clone());

            let ct_font_result = ct_font::new_from_name(&postscript_name, size as f64);
            let ct_font = ct_font_result.unwrap_or_else(|_| {
                // Fallback to Helvetica if font not found
                ct_font::new_from_name("Helvetica", size as f64).unwrap()
            });

            ct_font
        };

        let font_data = Arc::new(FontData {
            font,
            #[cfg(target_os = "macos")]
            native_font,
            font_id,
        });
        self.fonts.insert(key, font_data);

        Ok(font_id)
    }

    /// Get font metrics for a loaded font
    pub fn get_metrics(&self, font_id: u32) -> Option<FontMetrics> {
        self.fonts
            .values()
            .find(|data| data.font_id == font_id)
            .map(|data| {
                let metrics = data.font.metrics();
                let units_per_em = metrics.units_per_em as f32;

                FontMetrics {
                    ascent: metrics.ascent / units_per_em,
                    descent: metrics.descent / units_per_em,
                    leading: metrics.line_gap / units_per_em,
                    cap_height: metrics.cap_height / units_per_em,
                    x_height: metrics.x_height / units_per_em,
                }
            })
    }

    /// Get the Core Text font for a font ID (for text shaping)
    #[cfg(target_os = "macos")]
    pub fn get_ct_font(&self, font_id: u32) -> Option<CTFont> {
        self.fonts
            .values()
            .find(|data| data.font_id == font_id)
            .map(|data| data.native_font.clone())
    }

    /// Rasterize a glyph
    /// Rasterize a glyph at the specified size
    pub fn rasterize_glyph(
        &self,
        font_id: u32,
        glyph_id: u32,
        size: u32,
    ) -> Result<RasterizedGlyph, String> {
        let font_data = self
            .fonts
            .values()
            .find(|data| data.font_id == font_id)
            .ok_or_else(|| format!("Font {} not found", font_id))?;

        let font = &font_data.font;

        // Get glyph raster bounds
        let device_size = size as f32;
        let raster_bounds = font
            .raster_bounds(
                glyph_id,
                size as f32,
                Transform2F::default(),
                HintingOptions::None,
                RasterizationOptions::GrayscaleAa,
            )
            .map_err(|e| format!("Failed to get raster bounds: {:?}", e))?;

        let width = raster_bounds.width() as u32;
        let height = raster_bounds.height() as u32;

        // Get glyph advance
        let advance = font
            .advance(glyph_id)
            .map_err(|e| format!("Failed to get advance: {:?}", e))?;
        let advance = advance.x() * size as f32 / font.metrics().units_per_em as f32;

        // Create canvas for rasterization
        let mut canvas = Canvas::new(Vector2I::new(width as i32, height as i32), Format::A8);

        // Rasterize the glyph
        font.rasterize_glyph(
            &mut canvas,
            glyph_id,
            size as f32,
            Transform2F::from_translation(Vector2F::new(
                -raster_bounds.origin().x() as f32,
                -raster_bounds.origin().y() as f32,
            )),
            HintingOptions::None,
            RasterizationOptions::GrayscaleAa,
        )
        .map_err(|e| format!("Failed to rasterize glyph: {:?}", e))?;

        // Get the pixel data
        let data = canvas.pixels;

        Ok(RasterizedGlyph {
            width,
            height,
            bearing_x: raster_bounds.origin().x() as f32,
            bearing_y: -raster_bounds.origin().y() as f32,
            advance,
            data,
        })
    }

    /// Load a font from a font specification
    fn load_font_from_spec(&self, font_spec: &FontSpec) -> Result<FontKitFont, String> {
        // Simply try to load the requested font
        let family_name = font_kit::family_name::FamilyName::Title(font_spec.family.clone());

        self.system_source
            .select_best_match(&[family_name], &font_spec.style.into())
            .map_err(|e| format!("Failed to select font: {:?}", e))?
            .load()
            .map_err(|e| format!("Failed to load font: {:?}", e))
    }
}

impl From<FontStyle> for font_kit::properties::Properties {
    fn from(style: FontStyle) -> Self {
        use font_kit::properties::{Properties, Style, Weight};

        match style {
            FontStyle::Normal => Properties {
                style: Style::Normal,
                weight: Weight::NORMAL,
                stretch: Default::default(),
            },
            FontStyle::Bold => Properties {
                style: Style::Normal,
                weight: Weight::BOLD,
                stretch: Default::default(),
            },
            FontStyle::Italic => Properties {
                style: Style::Italic,
                weight: Weight::NORMAL,
                stretch: Default::default(),
            },
            FontStyle::BoldItalic => Properties {
                style: Style::Italic,
                weight: Weight::BOLD,
                stretch: Default::default(),
            },
        }
    }
}
