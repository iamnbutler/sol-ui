//! Text shaping and layout using Core Text

use core_foundation::attributed_string::CFMutableAttributedString;
use core_foundation::base::{CFRange, TCFType};
use core_foundation::string::CFString;
use core_graphics::geometry::CGPoint;
use core_text::font::CTFont;
use core_text::line::CTLine;
use glam::Vec2;

use crate::text::font::{FontManager, FontSpec};

/// A shaped glyph ready for rendering
#[derive(Debug, Clone)]
pub struct ShapedGlyph {
    /// Font ID from the font manager
    pub font_id: u32,
    /// Glyph ID in the font
    pub glyph_id: u32,
    /// Size in pixels (for atlas lookup)
    pub size: u32,
    /// Position relative to text origin
    pub position: Vec2,
}

/// Result of text shaping
#[derive(Debug, Clone)]
pub struct ShapedText {
    /// Individual glyphs with positions
    pub glyphs: Vec<ShapedGlyph>,
    /// Total size of the shaped text
    pub size: Vec2,
}

/// Text shaper that converts strings to positioned glyphs
pub struct TextShaper;

impl TextShaper {
    pub fn new() -> Self {
        Self
    }

    /// Measure text without full shaping
    pub fn measure(
        &self,
        text: &str,
        font: &FontSpec,
        size: f32,
        font_manager: &mut FontManager,
    ) -> Vec2 {
        if text.is_empty() {
            return Vec2::ZERO;
        }

        // Load the font
        let font_id = match font_manager.load_font(font, size) {
            Ok(id) => id,
            Err(_) => return Vec2::ZERO,
        };

        // Get the Core Text font
        let ct_font = match font_manager.get_ct_font(font_id) {
            Some(font) => font,
            None => return Vec2::ZERO,
        };

        // Create attributed string
        let cf_string = CFString::new(text);
        let mut attr_string = CFMutableAttributedString::new();
        attr_string.replace_str(&cf_string, CFRange::init(0, 0));

        // Set font attribute
        let range = CFRange::init(0, attr_string.char_len());
        attr_string.set_attribute(
            range,
            unsafe { core_text::string_attributes::kCTFontAttributeName },
            &ct_font,
        );

        // Create line and measure
        let line = CTLine::new_with_attributed_string(attr_string.as_concrete_TypeRef());

        // Get line bounds
        let bounds = line.get_bounds();

        Vec2::new(bounds.size.width as f32, bounds.size.height as f32)
    }

    /// Shape text into positioned glyphs
    pub fn shape(
        &self,
        text: &str,
        font: &FontSpec,
        size: f32,
        font_manager: &mut FontManager,
    ) -> Result<ShapedText, String> {
        if text.is_empty() {
            return Ok(ShapedText {
                glyphs: vec![],
                size: Vec2::ZERO,
            });
        }

        // Load the font
        let font_id = font_manager.load_font(font, size)?;

        // Get the Core Text font
        let ct_font = font_manager
            .get_ct_font(font_id)
            .ok_or_else(|| "Failed to get Core Text font".to_string())?;

        // Create attributed string
        let cf_string = CFString::new(text);
        let mut attr_string = CFMutableAttributedString::new();
        attr_string.replace_str(&cf_string, CFRange::init(0, 0));

        // Set font attribute
        let range = CFRange::init(0, attr_string.char_len());
        attr_string.set_attribute(
            range,
            unsafe { core_text::string_attributes::kCTFontAttributeName },
            &ct_font,
        );

        // Create line
        let line = CTLine::new_with_attributed_string(attr_string.as_concrete_TypeRef());

        // Get line metrics
        let line_bounds = line.get_bounds();
        let ascent = ct_font.ascent();
        let descent = ct_font.descent();

        // Get glyph runs
        let runs = line.glyph_runs();
        let mut shaped_glyphs = Vec::new();

        for run in runs.iter() {
            // Get run attributes
            let _run_font = if let Some(attributes) = run.attributes() {
                unsafe {
                    attributes
                        .get(core_text::string_attributes::kCTFontAttributeName)
                        .downcast::<CTFont>()
                        .map(|font| font.clone())
                        .unwrap_or_else(|| ct_font.clone())
                }
            } else {
                ct_font.clone()
            };

            // Get glyphs and positions
            let glyph_count = run.glyph_count();
            let glyphs = run.glyphs();
            let positions = run.positions();

            // NOTE: The glyph IDs from Core Text are font-specific indices.
            // These IDs must be used with the exact same font instance for rasterization.
            // Mismatched fonts will result in wrong glyphs (e.g., symbols instead of letters).
            for i in 0..glyph_count as usize {
                let glyph_id = glyphs.get(i).unwrap_or(&0);
                let default_position = CGPoint::new(0.0, 0.0);
                let position = positions.get(i).unwrap_or(&default_position);

                shaped_glyphs.push(ShapedGlyph {
                    font_id,
                    glyph_id: *glyph_id as u32,
                    size: size as u32,
                    position: Vec2::new(position.x as f32, position.y as f32),
                });
            }
        }

        Ok(ShapedText {
            glyphs: shaped_glyphs,
            size: Vec2::new(line_bounds.size.width as f32, (ascent + descent) as f32),
        })
    }
}

// Extension trait for CTLine
trait CTLineExt {
    fn get_bounds(&self) -> core_graphics::geometry::CGRect;
}

impl CTLineExt for CTLine {
    fn get_bounds(&self) -> core_graphics::geometry::CGRect {
        let bounds = self.get_typographic_bounds();
        core_graphics::geometry::CGRect::new(
            &core_graphics::geometry::CGPoint::new(0.0, -bounds.descent),
            &core_graphics::geometry::CGSize::new(bounds.width, bounds.ascent + bounds.descent),
        )
    }
}
