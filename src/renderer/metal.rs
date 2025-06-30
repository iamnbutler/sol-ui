use crate::text::{ShapedText, TextSystem};
use crate::ui::{Color as UiColor, DrawCommand, DrawList, Rect};
use metal::{
    Buffer, CommandQueue, Device, Library, MTLLoadAction, MTLPrimitiveType, MTLStoreAction,
    RenderPassDescriptor, RenderPipelineDescriptor, RenderPipelineState, VertexDescriptor,
};
use std::ffi::c_void;
use std::mem;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
    pub tex_coord: [f32; 2],
}

pub struct MetalRenderer {
    device: Device,
    pipeline_state: Option<RenderPipelineState>,
    text_pipeline_state: Option<RenderPipelineState>,
}

impl MetalRenderer {
    pub fn new(device: Device) -> Self {
        Self {
            device,
            pipeline_state: None,
            text_pipeline_state: None,
        }
    }

    pub fn initialize(&mut self) -> Result<(), String> {
        // Create shader library
        let library = self.compile_shaders()?;

        // Create pipeline states
        self.pipeline_state = Some(self.create_pipeline_state(&library)?);
        self.text_pipeline_state = Some(self.create_text_pipeline_state(&library)?);

        Ok(())
    }

    fn compile_shaders(&self) -> Result<Library, String> {
        let shader_source = r#"
            #include <metal_stdlib>
            using namespace metal;

            struct Vertex {
                float2 position [[attribute(0)]];
                float4 color [[attribute(1)]];
                float2 tex_coord [[attribute(2)]];
            };

            struct VertexOut {
                float4 position [[position]];
                float4 color;
                float2 tex_coord;
            };

            vertex VertexOut vertex_main(Vertex in [[stage_in]]) {
                VertexOut out;
                out.position = float4(in.position, 0.0, 1.0);
                out.color = in.color;
                out.tex_coord = in.tex_coord;
                return out;
            }

            fragment float4 fragment_main(VertexOut in [[stage_in]]) {
                return in.color;
            }

            // Text rendering shaders
            vertex VertexOut text_vertex_main(Vertex in [[stage_in]]) {
                VertexOut out;
                out.position = float4(in.position, 0.0, 1.0);
                out.color = in.color;
                out.tex_coord = in.tex_coord;
                return out;
            }

            fragment float4 text_fragment_main(VertexOut in [[stage_in]],
                                               texture2d<float> glyph_texture [[texture(0)]],
                                               sampler glyph_sampler [[sampler(0)]]) {
                float alpha = glyph_texture.sample(glyph_sampler, in.tex_coord).r;
                return float4(in.color.rgb, in.color.a * alpha);
            }
        "#;

        let options = metal::CompileOptions::new();
        self.device
            .new_library_with_source(shader_source, &options)
            .map_err(|e| format!("Failed to compile shaders: {}", e))
    }

    fn create_pipeline_state(&self, library: &Library) -> Result<RenderPipelineState, String> {
        let vertex_function = library
            .get_function("vertex_main", None)
            .map_err(|e| format!("Failed to find vertex_main function: {}", e))?;

        let fragment_function = library
            .get_function("fragment_main", None)
            .map_err(|e| format!("Failed to find fragment_main function: {}", e))?;

        // Create vertex descriptor
        let vertex_descriptor = VertexDescriptor::new();

        // Position attribute
        let position_attr = vertex_descriptor.attributes().object_at(0).unwrap();
        position_attr.set_format(metal::MTLVertexFormat::Float2);
        position_attr.set_offset(0);
        position_attr.set_buffer_index(0);

        // Color attribute
        let color_attr = vertex_descriptor.attributes().object_at(1).unwrap();
        color_attr.set_format(metal::MTLVertexFormat::Float4);
        color_attr.set_offset(8); // 2 floats * 4 bytes
        color_attr.set_buffer_index(0);

        // Texture coordinate attribute
        let tex_coord_attr = vertex_descriptor.attributes().object_at(2).unwrap();
        tex_coord_attr.set_format(metal::MTLVertexFormat::Float2);
        tex_coord_attr.set_offset(24); // 2 floats + 4 floats * 4 bytes
        tex_coord_attr.set_buffer_index(0);

        // Vertex layout
        let layout = vertex_descriptor.layouts().object_at(0).unwrap();
        layout.set_stride(mem::size_of::<Vertex>() as u64);

        // Create pipeline descriptor
        let pipeline_descriptor = RenderPipelineDescriptor::new();
        pipeline_descriptor.set_vertex_function(Some(&vertex_function));
        pipeline_descriptor.set_fragment_function(Some(&fragment_function));
        pipeline_descriptor.set_vertex_descriptor(Some(vertex_descriptor));

        // Configure color attachment
        let color_attachment = pipeline_descriptor
            .color_attachments()
            .object_at(0)
            .unwrap();
        color_attachment.set_pixel_format(metal::MTLPixelFormat::BGRA8Unorm);

        self.device
            .new_render_pipeline_state(&pipeline_descriptor)
            .map_err(|e| format!("Failed to create pipeline state: {}", e))
    }

    fn create_text_pipeline_state(&self, library: &Library) -> Result<RenderPipelineState, String> {
        let vertex_function = library
            .get_function("text_vertex_main", None)
            .map_err(|e| format!("Failed to find text_vertex_main function: {}", e))?;

        let fragment_function = library
            .get_function("text_fragment_main", None)
            .map_err(|e| format!("Failed to find text_fragment_main function: {}", e))?;

        // Create vertex descriptor
        let vertex_descriptor = VertexDescriptor::new();

        // Position attribute
        let position_attr = vertex_descriptor.attributes().object_at(0).unwrap();
        position_attr.set_format(metal::MTLVertexFormat::Float2);
        position_attr.set_offset(0);
        position_attr.set_buffer_index(0);

        // Color attribute
        let color_attr = vertex_descriptor.attributes().object_at(1).unwrap();
        color_attr.set_format(metal::MTLVertexFormat::Float4);
        color_attr.set_offset(8);
        color_attr.set_buffer_index(0);

        // Texture coordinate attribute
        let tex_coord_attr = vertex_descriptor.attributes().object_at(2).unwrap();
        tex_coord_attr.set_format(metal::MTLVertexFormat::Float2);
        tex_coord_attr.set_offset(24);
        tex_coord_attr.set_buffer_index(0);

        // Vertex layout
        let layout = vertex_descriptor.layouts().object_at(0).unwrap();
        layout.set_stride(mem::size_of::<Vertex>() as u64);

        // Create pipeline descriptor
        let pipeline_descriptor = RenderPipelineDescriptor::new();
        pipeline_descriptor.set_vertex_function(Some(&vertex_function));
        pipeline_descriptor.set_fragment_function(Some(&fragment_function));
        pipeline_descriptor.set_vertex_descriptor(Some(vertex_descriptor));

        // Configure color attachment with alpha blending
        let color_attachment = pipeline_descriptor
            .color_attachments()
            .object_at(0)
            .unwrap();
        color_attachment.set_pixel_format(metal::MTLPixelFormat::BGRA8Unorm);
        color_attachment.set_blending_enabled(true);
        color_attachment.set_source_rgb_blend_factor(metal::MTLBlendFactor::SourceAlpha);
        color_attachment
            .set_destination_rgb_blend_factor(metal::MTLBlendFactor::OneMinusSourceAlpha);
        color_attachment.set_source_alpha_blend_factor(metal::MTLBlendFactor::One);
        color_attachment
            .set_destination_alpha_blend_factor(metal::MTLBlendFactor::OneMinusSourceAlpha);

        self.device
            .new_render_pipeline_state(&pipeline_descriptor)
            .map_err(|e| format!("Failed to create text pipeline state: {}", e))
    }

    /// Convert text to vertices using shaped glyphs
    fn text_to_vertices(
        &self,
        position: glam::Vec2,
        shaped_text: &ShapedText,
        color: &UiColor,
        text_system: &TextSystem,
        screen_size: (f32, f32),
    ) -> Vec<Vertex> {
        let mut vertices = Vec::new();
        let color_array = [color.red, color.green, color.blue, color.alpha];

        for glyph in &shaped_text.glyphs {
            if let Some(info) = text_system.glyph_info(glyph.font_id, glyph.glyph_id, glyph.size) {
                // Calculate glyph position in screen space
                let glyph_x = position.x + glyph.position.x;
                let glyph_y = position.y + glyph.position.y - info.bearing_y;

                // Convert to NDC
                let x1 = (glyph_x / screen_size.0) * 2.0 - 1.0;
                let y1 = 1.0 - (glyph_y / screen_size.1) * 2.0;
                let x2 = ((glyph_x + info.width as f32) / screen_size.0) * 2.0 - 1.0;
                let y2 = 1.0 - ((glyph_y + info.height as f32) / screen_size.1) * 2.0;

                // Create two triangles for the glyph quad
                vertices.extend_from_slice(&[
                    Vertex {
                        position: [x1, y1],
                        color: color_array,
                        tex_coord: [info.uv_min.0, info.uv_min.1],
                    },
                    Vertex {
                        position: [x2, y1],
                        color: color_array,
                        tex_coord: [info.uv_max.0, info.uv_min.1],
                    },
                    Vertex {
                        position: [x1, y2],
                        color: color_array,
                        tex_coord: [info.uv_min.0, info.uv_max.1],
                    },
                    Vertex {
                        position: [x2, y1],
                        color: color_array,
                        tex_coord: [info.uv_max.0, info.uv_min.1],
                    },
                    Vertex {
                        position: [x2, y2],
                        color: color_array,
                        tex_coord: [info.uv_max.0, info.uv_max.1],
                    },
                    Vertex {
                        position: [x1, y2],
                        color: color_array,
                        tex_coord: [info.uv_min.0, info.uv_max.1],
                    },
                ]);
            }
        }

        vertices
    }

    /// Build vertices from UI draw commands
    fn build_vertices_from_draw_list(
        &self,
        draw_list: &DrawList,
        screen_size: (f32, f32),
        text_system: &mut TextSystem,
    ) -> (Vec<Vertex>, Vec<Vertex>) {
        let mut solid_vertices = Vec::new();
        let mut text_vertices = Vec::new();

        for command in draw_list.commands() {
            match command {
                DrawCommand::Rect { rect, color } => {
                    // Convert from screen coordinates to normalized device coordinates
                    let vertices_for_rect = self.rect_to_vertices(rect, color, screen_size);
                    solid_vertices.extend_from_slice(&vertices_for_rect);
                }
                DrawCommand::Text {
                    position,
                    text,
                    style,
                } => {
                    // Shape the text
                    let text_config = crate::text::TextConfig {
                        font: crate::text::FontSpec::default(),
                        size: style.size,
                        color: style.color,
                    };

                    match text_system.shape_text(text, &text_config) {
                        Ok(shaped_text) => {
                            let vertices = self.text_to_vertices(
                                *position,
                                &shaped_text,
                                &style.color,
                                text_system,
                                screen_size,
                            );
                            text_vertices.extend(vertices);
                        }
                        Err(e) => {
                            eprintln!("Failed to shape text: {}", e);
                        }
                    }
                }
                DrawCommand::PushClip { .. } | DrawCommand::PopClip => {
                    // TODO: Implement clipping
                }
            }
        }

        (solid_vertices, text_vertices)
    }

    /// Convert a rectangle to 6 vertices (2 triangles)
    fn rect_to_vertices(
        &self,
        rect: &Rect,
        color: &UiColor,
        screen_size: (f32, f32),
    ) -> [Vertex; 6] {
        // Convert from screen space to normalized device coordinates [-1, 1]
        let x1 = (rect.pos.x / screen_size.0) * 2.0 - 1.0;
        let y1 = 1.0 - (rect.pos.y / screen_size.1) * 2.0; // Flip Y axis
        let x2 = ((rect.pos.x + rect.size.x) / screen_size.0) * 2.0 - 1.0;
        let y2 = 1.0 - ((rect.pos.y + rect.size.y) / screen_size.1) * 2.0;

        let color_array = [color.red, color.green, color.blue, color.alpha];

        // Two triangles to make a rectangle
        [
            // First triangle
            Vertex {
                position: [x1, y1],
                color: color_array,
                tex_coord: [0.0, 0.0], // Not used for solid rects
            },
            Vertex {
                position: [x2, y1],
                color: color_array,
                tex_coord: [1.0, 0.0],
            },
            Vertex {
                position: [x1, y2],
                color: color_array,
                tex_coord: [0.0, 1.0],
            },
            // Second triangle
            Vertex {
                position: [x2, y1],
                color: color_array,
                tex_coord: [1.0, 0.0],
            },
            Vertex {
                position: [x2, y2],
                color: color_array,
                tex_coord: [1.0, 1.0],
            },
            Vertex {
                position: [x1, y2],
                color: color_array,
                tex_coord: [0.0, 1.0],
            },
        ]
    }

    pub fn render_frame(
        &self,
        command_queue: &CommandQueue,
        drawable: &metal::MetalDrawableRef,
        clear_color: metal::MTLClearColor,
        draw_list: &DrawList,
        screen_size: (f32, f32),
        text_system: &mut TextSystem,
    ) {
        let Some(pipeline_state) = &self.pipeline_state else {
            return;
        };
        let Some(text_pipeline_state) = &self.text_pipeline_state else {
            return;
        };

        // Build vertices from draw list
        let (solid_vertices, text_vertices) =
            self.build_vertices_from_draw_list(draw_list, screen_size, text_system);
        if solid_vertices.is_empty() && text_vertices.is_empty() {
            // Still clear the screen even if no UI elements
            let command_buffer = command_queue.new_command_buffer();
            let render_pass_descriptor = RenderPassDescriptor::new();
            let color_attachment = render_pass_descriptor
                .color_attachments()
                .object_at(0)
                .unwrap();
            color_attachment.set_texture(Some(drawable.texture()));
            color_attachment.set_load_action(MTLLoadAction::Clear);
            color_attachment.set_clear_color(clear_color);
            color_attachment.set_store_action(MTLStoreAction::Store);

            let render_encoder = command_buffer.new_render_command_encoder(&render_pass_descriptor);
            render_encoder.end_encoding();

            command_buffer.present_drawable(&drawable);
            command_buffer.commit();
            return;
        }

        // Create vertex buffers
        let solid_vertex_buffer = if !solid_vertices.is_empty() {
            let vertex_data = solid_vertices.as_ptr() as *const c_void;
            let vertex_data_size = (solid_vertices.len() * mem::size_of::<Vertex>()) as u64;
            Some(self.device.new_buffer_with_data(
                vertex_data,
                vertex_data_size,
                metal::MTLResourceOptions::CPUCacheModeDefaultCache,
            ))
        } else {
            None
        };

        let text_vertex_buffer = if !text_vertices.is_empty() {
            let vertex_data = text_vertices.as_ptr() as *const c_void;
            let vertex_data_size = (text_vertices.len() * mem::size_of::<Vertex>()) as u64;
            Some(self.device.new_buffer_with_data(
                vertex_data,
                vertex_data_size,
                metal::MTLResourceOptions::CPUCacheModeDefaultCache,
            ))
        } else {
            None
        };

        // Create command buffer
        let command_buffer = command_queue.new_command_buffer();

        // Create render pass descriptor
        let render_pass_descriptor = RenderPassDescriptor::new();
        let color_attachment = render_pass_descriptor
            .color_attachments()
            .object_at(0)
            .unwrap();

        color_attachment.set_texture(Some(drawable.texture()));
        color_attachment.set_load_action(MTLLoadAction::Clear);
        color_attachment.set_clear_color(clear_color);
        color_attachment.set_store_action(MTLStoreAction::Store);

        // Create render encoder
        let render_encoder = command_buffer.new_render_command_encoder(&render_pass_descriptor);

        // Draw solid geometry first
        if let Some(buffer) = solid_vertex_buffer {
            render_encoder.set_render_pipeline_state(&pipeline_state);
            render_encoder.set_vertex_buffer(0, Some(&buffer), 0);
            render_encoder.draw_primitives(
                MTLPrimitiveType::Triangle,
                0,
                solid_vertices.len() as u64,
            );
        }

        // Draw text geometry with texture
        // Render text vertices if any
        if let Some(buffer) = text_vertex_buffer {
            let texture = text_system.atlas_texture();
            render_encoder.set_render_pipeline_state(&text_pipeline_state);
            render_encoder.set_vertex_buffer(0, Some(&buffer), 0);
            render_encoder.set_fragment_texture(0, Some(texture));

            // Create and set sampler state
            let sampler_descriptor = metal::SamplerDescriptor::new();
            sampler_descriptor.set_min_filter(metal::MTLSamplerMinMagFilter::Linear);
            sampler_descriptor.set_mag_filter(metal::MTLSamplerMinMagFilter::Linear);
            let sampler_state = self.device.new_sampler(&sampler_descriptor);
            render_encoder.set_fragment_sampler_state(0, Some(&sampler_state));

            render_encoder.draw_primitives(
                MTLPrimitiveType::Triangle,
                0,
                text_vertices.len() as u64,
            );
        }

        // End encoding
        render_encoder.end_encoding();

        // Present drawable and commit
        command_buffer.present_drawable(&drawable);
        command_buffer.commit();
    }
}
