//! A [Parley](https://crates.io/crates/parley) based system for laying out and rendering rich text

use glam::Vec2;
use metal::{Device, Texture};
use parley::{
    FontContext, FontStack, FontWeight, GlyphRun, Layout, LayoutContext, LineHeight,
    PositionedLayoutItem, StyleProperty,
};
use std::collections::HashMap;
use swash::FontRef;
use swash::scale::{Render, ScaleContext, Source};

use crate::color::{Color, ColorExt};
use std::time::Instant;
use tracing::{debug, info, info_span};

/// Text rendering configuration
#[derive(Debug, Clone)]
pub struct TextConfig {
    /// Font family stack
    ///
    /// Will use the first available font in the stack
    pub font_stack: FontStack<'static>,
    /// Font size in logical pixels
    pub size: f32,
    /// Font weight
    pub weight: FontWeight,
    /// Text color
    pub color: Color,
    /// Line height multiplier
    pub line_height: f32,
}

impl Default for TextConfig {
    fn default() -> Self {
        Self {
            font_stack: FontStack::from("system-ui"),
            size: 16.0,
            weight: FontWeight::NORMAL,
            color: Color::new(0.0, 0.0, 0.0, 1.0),
            line_height: 1.2,
        }
    }
}

/// Information about a glyph in the atlas
#[derive(Debug, Clone, Copy)]
pub struct GlyphInfo {
    /// UV coordinates in the atlas (0.0 to 1.0)
    pub uv_min: (f32, f32),
    pub uv_max: (f32, f32),
    /// Size of the glyph in pixels
    pub width: u32,
    pub height: u32,
    /// Offset from the glyph origin to the top-left of the bitmap
    pub left: i32,
    pub top: i32,
}

/// Key for identifying a glyph in the atlas
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GlyphKey {
    font_id: u64,
    glyph_id: u16,
    size: u32,
}

/// A shelf in the atlas for packing glyphs
#[derive(Debug)]
struct Shelf {
    y: u32,
    height: u32,
    next_x: u32,
}

/// Glyph atlas that manages glyph textures
pub struct GlyphAtlas {
    texture: Texture,
    width: u32,
    height: u32,
    glyphs: HashMap<GlyphKey, GlyphInfo>,
    shelves: Vec<Shelf>,
}

impl GlyphAtlas {
    /// Create a new glyph atlas with the given dimensions
    pub fn new(device: &Device, width: u32, height: u32) -> Result<Self, String> {
        let descriptor = metal::TextureDescriptor::new();
        descriptor.set_pixel_format(metal::MTLPixelFormat::R8Unorm);
        descriptor.set_width(width as u64);
        descriptor.set_height(height as u64);
        descriptor
            .set_usage(metal::MTLTextureUsage::ShaderRead | metal::MTLTextureUsage::ShaderWrite);
        descriptor.set_storage_mode(metal::MTLStorageMode::Managed);

        let texture = device.new_texture(&descriptor);

        // Clear the texture to transparent
        let zeros = vec![0u8; (width * height) as usize];
        texture.replace_region(
            metal::MTLRegion {
                origin: metal::MTLOrigin { x: 0, y: 0, z: 0 },
                size: metal::MTLSize {
                    width: width as u64,
                    height: height as u64,
                    depth: 1,
                },
            },
            0,
            zeros.as_ptr() as *const _,
            width as u64,
        );

        Ok(Self {
            texture,
            width,
            height,
            glyphs: HashMap::new(),
            shelves: vec![],
        })
    }

    /// Check if a glyph is in the atlas
    pub fn contains(&self, font_id: u64, glyph_id: u16, size: u32) -> bool {
        let key = GlyphKey {
            font_id,
            glyph_id,
            size,
        };
        self.glyphs.contains_key(&key)
    }

    /// Add a glyph to the atlas
    pub fn add_glyph(
        &mut self,
        font_id: u64,
        glyph_id: u16,
        size: u32,
        data: &[u8],
        width: u32,
        height: u32,
        left: i32,
        top: i32,
    ) -> Result<(), String> {
        let key = GlyphKey {
            font_id,
            glyph_id,
            size,
        };

        if self.glyphs.contains_key(&key) {
            return Ok(());
        }

        let (x, y) = self.find_position(width, height)?;

        // Upload glyph data to texture
        if !data.is_empty() && width > 0 && height > 0 {
            self.texture.replace_region(
                metal::MTLRegion {
                    origin: metal::MTLOrigin {
                        x: x as u64,
                        y: y as u64,
                        z: 0,
                    },
                    size: metal::MTLSize {
                        width: width as u64,
                        height: height as u64,
                        depth: 1,
                    },
                },
                0,
                data.as_ptr() as *const _,
                width as u64,
            );
        }

        let uv_min = (x as f32 / self.width as f32, y as f32 / self.height as f32);
        let uv_max = (
            (x + width) as f32 / self.width as f32,
            (y + height) as f32 / self.height as f32,
        );

        let info = GlyphInfo {
            uv_min,
            uv_max,
            width,
            height,
            left,
            top,
        };

        self.glyphs.insert(key, info);
        Ok(())
    }

    /// Get information about a glyph in the atlas
    pub fn get_glyph(&self, font_id: u64, glyph_id: u16, size: u32) -> Option<&GlyphInfo> {
        let key = GlyphKey {
            font_id,
            glyph_id,
            size,
        };
        self.glyphs.get(&key)
    }

    /// Get the atlas texture
    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    /// Find a position for a glyph using shelf packing
    fn find_position(&mut self, width: u32, height: u32) -> Result<(u32, u32), String> {
        // Add 1px padding on each side (2px total) to prevent texture bleeding
        // when sampling adjacent glyphs due to bilinear filtering
        let padded_width = width + 2;
        let padded_height = height + 2;

        // Try to fit in an existing shelf
        for shelf in &mut self.shelves {
            if shelf.height >= padded_height && shelf.next_x + padded_width <= self.width {
                let x = shelf.next_x;
                shelf.next_x += padded_width;
                // +1 to skip the padding pixel at the start of the allocation
                return Ok((x + 1, shelf.y + 1));
            }
        }

        // Need a new shelf
        let next_y = if let Some(last_shelf) = self.shelves.last() {
            last_shelf.y + last_shelf.height
        } else {
            0
        };

        if next_y + padded_height > self.height {
            return Err("Atlas is full".to_string());
        }

        self.shelves.push(Shelf {
            y: next_y,
            height: padded_height,
            next_x: padded_width,
        });

        // +1 to skip the padding pixel at the start of the allocation
        Ok((1, next_y + 1))
    }
}

/// A shaped glyph ready for rendering
#[derive(Debug, Clone)]
pub struct ShapedGlyph {
    /// Font ID (unique identifier for font)
    pub font_id: u64,
    /// Glyph ID in the font
    pub glyph_id: u16,
    /// Size in pixels
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

/// Cache key for shaped text
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ShapedTextCacheKey {
    text: String,
    font_stack: String,
    size: u32,
    weight: u16,
    line_height: u32,
    max_width: Option<u32>,
    scale_factor: u32,
}

/// Text system that manages fonts, shaping, and atlas
pub struct TextSystem {
    font_context: FontContext,
    layout_context: LayoutContext,
    scale_context: ScaleContext,
    glyph_atlas: GlyphAtlas,
    /// Cache of font data to ID mappings
    font_id_cache: HashMap<Vec<u8>, u64>,
    next_font_id: u64,
    /// Cache of shaped text
    shaped_text_cache: HashMap<ShapedTextCacheKey, ShapedText>,
    /// Frame-based cache for text measurements to avoid duplicate work
    measurement_cache: HashMap<MeasurementCacheKey, Vec2>,
}

/// Key for text measurement cache
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct MeasurementCacheKey {
    text: String,
    font_stack: String,
    size: u32,
    weight: u16,
    line_height: u32,
    max_width: Option<u32>,
    scale_factor: u32,
}

impl TextSystem {
    /// Create a new text system with the given Metal device
    pub fn new(device: &Device) -> Result<Self, String> {
        let _new_span = info_span!("text_system_new").entered();
        let total_start = Instant::now();

        let start = Instant::now();
        let font_context = FontContext::new();
        info!("FontContext created in {:?}", start.elapsed());

        let start = Instant::now();
        let layout_context = LayoutContext::new();
        info!("LayoutContext created in {:?}", start.elapsed());

        let start = Instant::now();
        let scale_context = ScaleContext::new();
        info!("ScaleContext created in {:?}", start.elapsed());

        let start = Instant::now();
        let glyph_atlas = GlyphAtlas::new(device, 2048, 2048)?;
        info!("GlyphAtlas created in {:?}", start.elapsed());

        info!(
            "Total TextSystem initialization: {:?}",
            total_start.elapsed()
        );

        Ok(Self {
            font_context,
            layout_context,
            scale_context,
            glyph_atlas,
            font_id_cache: HashMap::new(),
            next_font_id: 1,
            shaped_text_cache: HashMap::new(),
            measurement_cache: HashMap::new(),
        })
    }

    /// Called at the start of each frame - maintains caches
    pub fn begin_frame(&mut self) {
        // Text measurements are deterministic and can persist across frames.
        // Only clear if cache gets too large to prevent unbounded memory growth.
        const MAX_MEASUREMENT_CACHE_SIZE: usize = 1000;
        if self.measurement_cache.len() > MAX_MEASUREMENT_CACHE_SIZE {
            debug!(
                "Measurement cache exceeded {} entries, clearing",
                MAX_MEASUREMENT_CACHE_SIZE
            );
            self.measurement_cache.clear();
        }

        // Similarly for shaped text cache
        const MAX_SHAPED_TEXT_CACHE_SIZE: usize = 500;
        if self.shaped_text_cache.len() > MAX_SHAPED_TEXT_CACHE_SIZE {
            debug!(
                "Shaped text cache exceeded {} entries, clearing",
                MAX_SHAPED_TEXT_CACHE_SIZE
            );
            self.shaped_text_cache.clear();
        }
    }

    /// Measure text with the given configuration
    pub fn measure_text(
        &mut self,
        text: &str,
        config: &TextConfig,
        max_width: Option<f32>,
        scale_factor: f32,
    ) -> Vec2 {
        let _measure_span = info_span!("measure_text", text_len = text.len()).entered();
        if text.is_empty() {
            return Vec2::ZERO;
        }

        // Create cache key
        let cache_key = MeasurementCacheKey {
            text: text.to_string(),
            font_stack: format!("{:?}", config.font_stack),
            size: (config.size * 100.0) as u32,
            weight: config.weight.value() as u16,
            line_height: (config.line_height * 100.0) as u32,
            max_width: max_width.map(|w| (w * 100.0) as u32),
            scale_factor: (scale_factor * 100.0) as u32,
        };

        // Check cache
        if let Some(&cached_size) = self.measurement_cache.get(&cache_key) {
            debug!(
                "Using cached measurement for '{}' -> {}x{}",
                if text.len() > 20 {
                    format!("{}...", &text[..20])
                } else {
                    text.to_string()
                },
                cached_size.x,
                cached_size.y
            );
            return cached_size;
        }

        // Create a layout
        let mut builder = self.layout_context.ranged_builder(
            &mut self.font_context,
            text,
            scale_factor,
            false, // no pixel snapping for measurement
        );

        // Apply text styles
        let brush = config.color.as_u8_arr();
        builder.push_default(StyleProperty::Brush(brush));
        builder.push_default(config.font_stack.clone());
        builder.push_default(StyleProperty::FontSize(config.size));
        builder.push_default(StyleProperty::FontWeight(config.weight));
        builder.push_default(StyleProperty::LineHeight(LineHeight::FontSizeRelative(
            config.line_height,
        )));

        let mut layout: Layout<[u8; 4]> = builder.build(text);
        layout.break_all_lines(max_width);

        let size = Vec2::new(layout.width(), layout.height());

        // Store in cache
        self.measurement_cache.insert(cache_key, size);

        debug!(
            "Measured text '{}' -> {}x{} (cached)",
            if text.len() > 20 {
                format!("{}...", &text[..20])
            } else {
                text.to_string()
            },
            size.x,
            size.y
        );
        size
    }

    /// Shape and prepare text for rendering
    pub fn shape_text(
        &mut self,
        text: &str,
        config: &TextConfig,
        max_width: Option<f32>,
        scale_factor: f32,
    ) -> Result<ShapedText, String> {
        let _shape_span = info_span!("shape_text", text_len = text.len()).entered();
        if text.is_empty() {
            return Ok(ShapedText {
                glyphs: vec![],
                size: Vec2::ZERO,
            });
        }

        // Create cache key
        let cache_key = ShapedTextCacheKey {
            text: text.to_string(),
            font_stack: format!("{:?}", config.font_stack),
            size: (config.size * 100.0) as u32,
            weight: config.weight.value() as u16,
            line_height: (config.line_height * 100.0) as u32,
            max_width: max_width.map(|w| (w * 100.0) as u32),
            scale_factor: (scale_factor * 100.0) as u32,
        };

        // Check cache
        let cache_check = info_span!("check_shaped_text_cache").entered();
        if let Some(cached) = self.shaped_text_cache.get(&cache_key) {
            // Ensure all glyphs are still in the atlas
            let mut all_glyphs_cached = true;
            for glyph in &cached.glyphs {
                if !self
                    .glyph_atlas
                    .contains(glyph.font_id, glyph.glyph_id, glyph.size)
                {
                    all_glyphs_cached = false;
                    break;
                }
            }

            if all_glyphs_cached {
                debug!("Using cached shaped text");
                drop(cache_check);
                return Ok(cached.clone());
            }
        }

        // Create a layout
        let mut builder = self.layout_context.ranged_builder(
            &mut self.font_context,
            text,
            scale_factor,
            true, // pixel snapping
        );

        // Apply text styles
        let brush = config.color.as_u8_arr();
        builder.push_default(StyleProperty::Brush(brush));
        builder.push_default(config.font_stack.clone());
        builder.push_default(StyleProperty::FontSize(config.size));
        builder.push_default(StyleProperty::FontWeight(config.weight));
        builder.push_default(StyleProperty::LineHeight(LineHeight::FontSizeRelative(
            config.line_height,
        )));

        let mut layout: Layout<[u8; 4]> = builder.build(text);
        layout.break_all_lines(max_width);

        let mut shaped_glyphs = Vec::new();

        // Process each line and glyph run
        for line in layout.lines() {
            for item in line.items() {
                if let PositionedLayoutItem::GlyphRun(glyph_run) = item {
                    self.process_glyph_run(&glyph_run, &mut shaped_glyphs)?;
                }
            }
        }

        let shaped_text = ShapedText {
            glyphs: shaped_glyphs,
            size: Vec2::new(layout.width(), layout.height()),
        };

        // Store in cache
        self.shaped_text_cache
            .insert(cache_key, shaped_text.clone());

        Ok(shaped_text)
    }

    /// Process a glyph run, rasterizing glyphs as needed
    fn process_glyph_run(
        &mut self,
        glyph_run: &GlyphRun<'_, [u8; 4]>,
        shaped_glyphs: &mut Vec<ShapedGlyph>,
    ) -> Result<(), String> {
        let run = glyph_run.run();
        let font = run.font();
        let font_size = run.font_size();
        let normalized_coords = run.normalized_coords();

        // Get or create font ID
        let font_id = self.get_or_create_font_id(font.data.as_ref());

        // Convert to swash font
        let font_ref = FontRef::from_index(font.data.as_ref(), font.index as usize)
            .ok_or_else(|| "Failed to create font reference".to_string())?;

        // Create scaler for this run
        let mut scaler = self
            .scale_context
            .builder(font_ref)
            .size(font_size)
            .hint(true)
            .normalized_coords(normalized_coords)
            .build();

        let mut run_x = glyph_run.offset();
        let run_y = glyph_run.baseline();

        // Process each glyph
        for glyph in glyph_run.glyphs() {
            let glyph_x = run_x + glyph.x;
            let glyph_y = run_y - glyph.y;
            run_x += glyph.advance;

            // Ensure glyph is in atlas
            let size_u32 = font_size.round() as u32;
            let needs_rasterization = !self.glyph_atlas.contains(font_id, glyph.id, size_u32);

            if needs_rasterization {
                // Render the glyph
                let rendered = Render::new(&[Source::Outline])
                    .format(swash::zeno::Format::Alpha)
                    .render(&mut scaler, glyph.id)
                    .ok_or_else(|| "Failed to render glyph".to_string())?;

                // Add to atlas
                self.glyph_atlas.add_glyph(
                    font_id,
                    glyph.id,
                    size_u32,
                    &rendered.data,
                    rendered.placement.width,
                    rendered.placement.height,
                    rendered.placement.left,
                    rendered.placement.top,
                )?;
            }

            shaped_glyphs.push(ShapedGlyph {
                font_id,
                glyph_id: glyph.id,
                size: size_u32,
                position: Vec2::new(glyph_x, glyph_y),
            });
        }

        Ok(())
    }

    /// Get or create a font ID for the given font data
    fn get_or_create_font_id(&mut self, font_data: &[u8]) -> u64 {
        let key = font_data.to_vec();
        if let Some(&id) = self.font_id_cache.get(&key) {
            id
        } else {
            let id = self.next_font_id;
            self.next_font_id += 1;
            self.font_id_cache.insert(key, id);
            id
        }
    }

    /// Get the glyph atlas texture
    pub fn atlas_texture(&self) -> &Texture {
        self.glyph_atlas.texture()
    }

    /// Get information about a glyph in the atlas
    pub fn glyph_info(&self, font_id: u64, glyph_id: u16, size: u32) -> Option<&GlyphInfo> {
        self.glyph_atlas.get_glyph(font_id, glyph_id, size)
    }
}
