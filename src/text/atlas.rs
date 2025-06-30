//! Glyph atlas for managing glyph textures

use metal::{Device, Texture, TextureDescriptor};
use std::collections::HashMap;

use crate::text::RasterizedGlyph;

/// Information about a glyph in the atlas
#[derive(Debug, Clone, Copy)]
pub struct GlyphInfo {
    /// UV coordinates in the atlas (0.0 to 1.0)
    pub uv_min: (f32, f32),
    pub uv_max: (f32, f32),
    /// Size of the glyph in pixels
    pub width: u32,
    pub height: u32,
    /// Glyph metrics
    pub bearing_x: f32,
    pub bearing_y: f32,
    pub advance: f32,
}

/// Key for identifying a glyph in the atlas
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GlyphKey {
    font_id: u32,
    glyph_id: u32,
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
    dirty: bool,
}

impl GlyphAtlas {
    /// Create a new glyph atlas with the given dimensions
    pub fn new(device: &Device, width: u32, height: u32) -> Result<Self, String> {
        let descriptor = TextureDescriptor::new();
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
            dirty: false,
        })
    }

    /// Check if a glyph is in the atlas
    pub fn contains(&self, font_id: u32, glyph_id: u32, size: u32) -> bool {
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
        font_id: u32,
        glyph_id: u32,
        size: u32,
        glyph_data: RasterizedGlyph,
    ) -> Result<(), String> {
        let key = GlyphKey {
            font_id,
            glyph_id,
            size,
        };

        // Check if already in atlas
        if self.glyphs.contains_key(&key) {
            return Ok(());
        }

        // Find a position for the glyph
        let (x, y) = self.find_position(glyph_data.width, glyph_data.height)?;

        // Upload glyph data to texture
        if !glyph_data.data.is_empty() {
            self.texture.replace_region(
                metal::MTLRegion {
                    origin: metal::MTLOrigin {
                        x: x as u64,
                        y: y as u64,
                        z: 0,
                    },
                    size: metal::MTLSize {
                        width: glyph_data.width as u64,
                        height: glyph_data.height as u64,
                        depth: 1,
                    },
                },
                0,
                glyph_data.data.as_ptr() as *const _,
                glyph_data.width as u64,
            );
        }

        // Calculate UV coordinates
        let uv_min = (x as f32 / self.width as f32, y as f32 / self.height as f32);
        let uv_max = (
            (x + glyph_data.width) as f32 / self.width as f32,
            (y + glyph_data.height) as f32 / self.height as f32,
        );

        // Store glyph info
        let info = GlyphInfo {
            uv_min,
            uv_max,
            width: glyph_data.width,
            height: glyph_data.height,
            bearing_x: glyph_data.bearing_x,
            bearing_y: glyph_data.bearing_y,
            advance: glyph_data.advance,
        };

        self.glyphs.insert(key, info);
        self.dirty = true;

        Ok(())
    }

    /// Get information about a glyph in the atlas
    /// Get glyph information
    pub fn get_glyph(&self, font_id: u32, glyph_id: u32, size: u32) -> Option<&GlyphInfo> {
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

    /// Clear the atlas
    pub fn clear(&mut self) -> Result<(), String> {
        self.glyphs.clear();
        self.shelves.clear();
        self.dirty = false;

        // Clear the texture
        let zeros = vec![0u8; (self.width * self.height) as usize];
        self.texture.replace_region(
            metal::MTLRegion {
                origin: metal::MTLOrigin { x: 0, y: 0, z: 0 },
                size: metal::MTLSize {
                    width: self.width as u64,
                    height: self.height as u64,
                    depth: 1,
                },
            },
            0,
            zeros.as_ptr() as *const _,
            self.width as u64,
        );

        Ok(())
    }

    /// Find a position for a glyph using shelf packing
    fn find_position(&mut self, width: u32, height: u32) -> Result<(u32, u32), String> {
        // Add padding around glyphs
        let padded_width = width + 2;
        let padded_height = height + 2;

        // Try to fit in an existing shelf
        for shelf in &mut self.shelves {
            if shelf.height >= padded_height && shelf.next_x + padded_width <= self.width {
                let x = shelf.next_x;
                shelf.next_x += padded_width;
                return Ok((x + 1, shelf.y + 1)); // +1 for padding
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

        Ok((1, next_y + 1)) // +1 for padding
    }

    /// Check if the atlas has been modified since last query
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Mark the atlas as clean
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }
}
