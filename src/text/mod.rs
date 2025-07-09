pub mod atlas;
pub mod font;
pub mod shaper;

pub use atlas::{GlyphAtlas, GlyphInfo};
pub use font::{FontManager, FontSpec, FontStyle};
pub use shaper::{ShapedText, TextShaper};

use crate::color::Color;
use glam::Vec2;

/// Text rendering configuration
#[derive(Debug, Clone)]
pub struct TextConfig {
    /// Font to use for rendering
    pub font: FontSpec,
    /// Font size in logical pixels
    pub size: f32,
    /// Text color
    pub color: Color,
}

impl Default for TextConfig {
    fn default() -> Self {
        Self {
            font: FontSpec::default(),
            size: 16.0,
            color: Color::new(0.0, 0.0, 0.0, 1.0),
        }
    }
}

/// A text rendering system that manages fonts, shaping, and atlas
pub struct TextSystem {
    font_manager: FontManager,
    text_shaper: TextShaper,
    glyph_atlas: GlyphAtlas,
}

impl TextSystem {
    /// Create a new text system with the given Metal device
    pub fn new(device: &metal::Device) -> Result<Self, String> {
        let font_manager = FontManager::new()?;
        let text_shaper = TextShaper::new();
        let glyph_atlas = GlyphAtlas::new(device, 1024, 1024)?;

        Ok(Self {
            font_manager,
            text_shaper,
            glyph_atlas,
        })
    }

    /// Measure text with the given configuration
    pub fn measure_text(&mut self, text: &str, config: &TextConfig) -> Vec2 {
        self.text_shaper
            .measure(text, &config.font, config.size, &mut self.font_manager)
    }

    /// Shape and prepare text for rendering
    pub fn shape_text(&mut self, text: &str, config: &TextConfig) -> Result<ShapedText, String> {
        // Shape the text
        let shaped =
            self.text_shaper
                .shape(text, &config.font, config.size, &mut self.font_manager)?;

        // Ensure all glyphs are in the atlas
        for glyph in &shaped.glyphs {
            if !self
                .glyph_atlas
                .contains(glyph.font_id, glyph.glyph_id, glyph.size)
            {
                // Rasterize and add to atlas
                let raster_data =
                    self.font_manager
                        .rasterize_glyph(glyph.font_id, glyph.glyph_id, glyph.size)?;

                self.glyph_atlas.add_glyph(
                    glyph.font_id,
                    glyph.glyph_id,
                    glyph.size,
                    raster_data,
                )?;
            }
        }

        Ok(shaped)
    }

    /// Get the glyph atlas texture
    pub fn atlas_texture(&self) -> &metal::Texture {
        self.glyph_atlas.texture()
    }

    /// Get information about a glyph in the atlas
    pub fn glyph_info(&self, font_id: u32, glyph_id: u32, size: u32) -> Option<&GlyphInfo> {
        self.glyph_atlas.get_glyph(font_id, glyph_id, size)
    }

    /// Clear the glyph atlas (useful for memory management)
    pub fn clear_atlas(&mut self) -> Result<(), String> {
        self.glyph_atlas.clear()
    }
}

/// Rasterized glyph data
#[derive(Debug)]
pub struct RasterizedGlyph {
    /// Width of the glyph bitmap
    pub width: u32,
    /// Height of the glyph bitmap
    pub height: u32,
    /// Horizontal bearing (offset from cursor to left edge of glyph)
    pub bearing_x: f32,
    /// Vertical bearing (offset from baseline to top edge of glyph)
    pub bearing_y: f32,
    /// Horizontal advance (how far to move cursor after this glyph)
    pub advance: f32,
    /// Grayscale bitmap data (one byte per pixel)
    pub data: Vec<u8>,
}
