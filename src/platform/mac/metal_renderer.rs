use crate::{
    color::Color,
    geometry::Rect,
    render::{DrawCommand, DrawList},
    style::{ElementStyle, Fill},
    text_system::{ShapedText, TextSystem},
};
use glam::Vec2;
use metal::{
    Buffer, CommandBufferRef, CommandQueue, Device, Library, MTLLoadAction, MTLPrimitiveType,
    MTLStoreAction, RenderPassDescriptor, RenderPipelineDescriptor, RenderPipelineState,
    VertexDescriptor,
};
use std::ffi::c_void;
use std::mem;
use std::time::Instant;
use tracing::{debug, info, info_span};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
    pub tex_coord: [f32; 2],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct FrameUniforms {
    center: [f32; 2],
    half_size: [f32; 2],
    radii: [f32; 4], // top_left, top_right, bottom_right, bottom_left
    border_width: f32,
    fill_type: u32,      // 0 = solid, 1 = linear gradient, 2 = radial gradient
    gradient_angle: f32, // For linear gradient
    _padding: f32,       // Padding to align to 16 bytes
    color1: [f32; 4],    // Solid color or gradient start/center
    color2: [f32; 4],    // Gradient end/edge (unused for solid)
    border_color: [f32; 4],
    shadow_offset: [f32; 2],
    shadow_blur: f32,
    _padding2: f32,
    shadow_color: [f32; 4],
}

pub struct MetalRenderer {
    device: Device,
    pipeline_state: Option<RenderPipelineState>,
    text_pipeline_state: Option<RenderPipelineState>,
    frame_pipeline_state: Option<RenderPipelineState>,
}

impl MetalRenderer {
    pub fn new(device: Device) -> Self {
        Self {
            device,
            pipeline_state: None,
            text_pipeline_state: None,
            frame_pipeline_state: None,
        }
    }

    pub fn initialize(&mut self) -> Result<(), String> {
        // Create shader library
        let start = Instant::now();
        let library = self.compile_shaders()?;
        info!("Shaders compiled in {:?}", start.elapsed());

        // Create pipeline states
        self.pipeline_state = Some(self.create_pipeline_state(&library)?);
        self.text_pipeline_state = Some(self.create_text_pipeline_state(&library)?);
        self.frame_pipeline_state = Some(self.create_frame_pipeline_state(&library)?);

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

            // SDF Frame rendering shaders
            struct FrameUniforms {
                float2 center;
                float2 half_size;
                float4 radii; // top_left, top_right, bottom_right, bottom_left
                float border_width;
                uint fill_type; // 0 = solid, 1 = linear gradient, 2 = radial gradient
                float gradient_angle;
                float _padding;
                float4 color1; // Solid color or gradient start/center
                float4 color2; // Gradient end/edge
                float4 border_color;
                float2 shadow_offset;
                float shadow_blur;
                float _padding2;
                float4 shadow_color;
            };

            float sdRoundedRect(float2 p, float2 half_size, float4 radii) {
                // Select the appropriate radius based on quadrant
                float radius = p.x > 0.0 ?
                    (p.y > 0.0 ? radii.z : radii.y) :
                    (p.y > 0.0 ? radii.w : radii.x);

                float2 q = abs(p) - half_size + radius;
                return min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - radius;
            }

            vertex VertexOut frame_vertex_main(Vertex in [[stage_in]]) {
                VertexOut out;
                out.position = float4(in.position, 0.0, 1.0);
                out.color = in.color;
                out.tex_coord = in.tex_coord;
                return out;
            }

            fragment float4 frame_fragment_main(VertexOut in [[stage_in]],
                                              constant FrameUniforms& uniforms [[buffer(0)]]) {
                // Convert from texture coordinates to local space coordinates
                // tex_coord can be outside 0-1 range due to shadow expansion
                // Map (0,0)-(1,1) to (-half_size, +half_size) in frame space
                float2 normalized = in.tex_coord;
                float2 p = (normalized - float2(0.5, 0.5)) * uniforms.half_size * 2.0;

                // Shadow calculation (behind the main shape)
                float shadow_alpha = 0.0;
                if (uniforms.shadow_color.a > 0.0) {
                    float2 shadow_p = p - uniforms.shadow_offset;
                    float shadow_d = sdRoundedRect(shadow_p, uniforms.half_size, uniforms.radii);

                    // Handle both hard and soft shadows
                    if (uniforms.shadow_blur > 0.0) {
                        // Soft shadow using blur
                        shadow_alpha = uniforms.shadow_color.a * (1.0 - smoothstep(-uniforms.shadow_blur, uniforms.shadow_blur, shadow_d));
                    } else {
                        // Hard shadow (0 blur)
                        shadow_alpha = (shadow_d <= 0.0) ? uniforms.shadow_color.a : 0.0;
                    }
                }

                float d = sdRoundedRect(p, uniforms.half_size, uniforms.radii);

                // Anti-aliasing
                float aa = fwidth(d) * 0.5;

                // Fill mask
                float fill_mask = 1.0 - smoothstep(-aa, aa, d);

                // Calculate fill color based on fill type
                float4 fill_color = uniforms.color1;
                if (uniforms.fill_type == 1) { // Linear gradient
                    // Calculate gradient coordinate
                    float2 gradient_dir = float2(cos(uniforms.gradient_angle), sin(uniforms.gradient_angle));
                    float t = dot(p, gradient_dir) / dot(uniforms.half_size * 2.0, abs(gradient_dir));
                    t = (t + 1.0) * 0.5; // Normalize to 0-1
                    fill_color = mix(uniforms.color1, uniforms.color2, t);
                } else if (uniforms.fill_type == 2) { // Radial gradient
                    float t = length(p) / length(uniforms.half_size);
                    fill_color = mix(uniforms.color1, uniforms.color2, smoothstep(0.0, 1.0, t));
                }

                // Border mask (only if border width > 0)
                float4 color = fill_color;
                if (uniforms.border_width > 0.0) {
                    float border_inner = d + uniforms.border_width;
                    float border_mask = smoothstep(-aa, aa, border_inner) * fill_mask;
                    color = mix(fill_color, uniforms.border_color, border_mask);
                }

                // Apply fill mask to color
                color.a *= fill_mask;

                // Composite frame over shadow using proper alpha blending
                // out_color = shadow_color * (1 - frame_alpha) + frame_color
                float3 final_rgb = uniforms.shadow_color.rgb * shadow_alpha * (1.0 - color.a) + color.rgb * color.a;
                float final_alpha = shadow_alpha * (1.0 - color.a) + color.a;

                return float4(final_rgb, final_alpha);
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

        // Buffer layout
        let layout = vertex_descriptor.layouts().object_at(0).unwrap();
        layout.set_stride(32); // Total size of Vertex struct
        layout.set_step_function(metal::MTLVertexStepFunction::PerVertex);

        let pipeline_descriptor = RenderPipelineDescriptor::new();
        pipeline_descriptor.set_vertex_function(Some(&vertex_function));
        pipeline_descriptor.set_fragment_function(Some(&fragment_function));
        pipeline_descriptor.set_vertex_descriptor(Some(vertex_descriptor));

        let attachment = pipeline_descriptor
            .color_attachments()
            .object_at(0)
            .unwrap();
        attachment.set_pixel_format(metal::MTLPixelFormat::BGRA8Unorm);
        attachment.set_blending_enabled(true);
        attachment.set_source_rgb_blend_factor(metal::MTLBlendFactor::SourceAlpha);
        attachment.set_destination_rgb_blend_factor(metal::MTLBlendFactor::OneMinusSourceAlpha);
        attachment.set_source_alpha_blend_factor(metal::MTLBlendFactor::SourceAlpha);
        attachment.set_destination_alpha_blend_factor(metal::MTLBlendFactor::OneMinusSourceAlpha);

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

        let vertex_descriptor = VertexDescriptor::new();

        // Same vertex descriptor as solid pipeline
        let position_attr = vertex_descriptor.attributes().object_at(0).unwrap();
        position_attr.set_format(metal::MTLVertexFormat::Float2);
        position_attr.set_offset(0);
        position_attr.set_buffer_index(0);

        let color_attr = vertex_descriptor.attributes().object_at(1).unwrap();
        color_attr.set_format(metal::MTLVertexFormat::Float4);
        color_attr.set_offset(8);
        color_attr.set_buffer_index(0);

        let tex_coord_attr = vertex_descriptor.attributes().object_at(2).unwrap();
        tex_coord_attr.set_format(metal::MTLVertexFormat::Float2);
        tex_coord_attr.set_offset(24);
        tex_coord_attr.set_buffer_index(0);

        let layout = vertex_descriptor.layouts().object_at(0).unwrap();
        layout.set_stride(32);
        layout.set_step_function(metal::MTLVertexStepFunction::PerVertex);

        let pipeline_descriptor = RenderPipelineDescriptor::new();
        pipeline_descriptor.set_vertex_function(Some(&vertex_function));
        pipeline_descriptor.set_fragment_function(Some(&fragment_function));
        pipeline_descriptor.set_vertex_descriptor(Some(vertex_descriptor));

        let attachment = pipeline_descriptor
            .color_attachments()
            .object_at(0)
            .unwrap();
        attachment.set_pixel_format(metal::MTLPixelFormat::BGRA8Unorm);
        attachment.set_blending_enabled(true);
        attachment.set_source_rgb_blend_factor(metal::MTLBlendFactor::SourceAlpha);
        attachment.set_destination_rgb_blend_factor(metal::MTLBlendFactor::OneMinusSourceAlpha);
        attachment.set_source_alpha_blend_factor(metal::MTLBlendFactor::SourceAlpha);
        attachment.set_destination_alpha_blend_factor(metal::MTLBlendFactor::OneMinusSourceAlpha);

        self.device
            .new_render_pipeline_state(&pipeline_descriptor)
            .map_err(|e| format!("Failed to create text pipeline state: {}", e))
    }

    fn create_frame_pipeline_state(
        &self,
        library: &Library,
    ) -> Result<RenderPipelineState, String> {
        let vertex_function = library
            .get_function("frame_vertex_main", None)
            .map_err(|e| format!("Failed to find frame_vertex_main function: {}", e))?;

        let fragment_function = library
            .get_function("frame_fragment_main", None)
            .map_err(|e| format!("Failed to find frame_fragment_main function: {}", e))?;

        let vertex_descriptor = VertexDescriptor::new();

        // Same vertex descriptor as other pipelines
        let position_attr = vertex_descriptor.attributes().object_at(0).unwrap();
        position_attr.set_format(metal::MTLVertexFormat::Float2);
        position_attr.set_offset(0);
        position_attr.set_buffer_index(0);

        let color_attr = vertex_descriptor.attributes().object_at(1).unwrap();
        color_attr.set_format(metal::MTLVertexFormat::Float4);
        color_attr.set_offset(8);
        color_attr.set_buffer_index(0);

        let tex_coord_attr = vertex_descriptor.attributes().object_at(2).unwrap();
        tex_coord_attr.set_format(metal::MTLVertexFormat::Float2);
        tex_coord_attr.set_offset(24);
        tex_coord_attr.set_buffer_index(0);

        let layout = vertex_descriptor.layouts().object_at(0).unwrap();
        layout.set_stride(32);
        layout.set_step_function(metal::MTLVertexStepFunction::PerVertex);

        let pipeline_descriptor = RenderPipelineDescriptor::new();
        pipeline_descriptor.set_vertex_function(Some(&vertex_function));
        pipeline_descriptor.set_fragment_function(Some(&fragment_function));
        pipeline_descriptor.set_vertex_descriptor(Some(vertex_descriptor));

        let attachment = pipeline_descriptor
            .color_attachments()
            .object_at(0)
            .unwrap();
        attachment.set_pixel_format(metal::MTLPixelFormat::BGRA8Unorm);
        attachment.set_blending_enabled(true);
        attachment.set_source_rgb_blend_factor(metal::MTLBlendFactor::SourceAlpha);
        attachment.set_destination_rgb_blend_factor(metal::MTLBlendFactor::OneMinusSourceAlpha);
        attachment.set_source_alpha_blend_factor(metal::MTLBlendFactor::SourceAlpha);
        attachment.set_destination_alpha_blend_factor(metal::MTLBlendFactor::OneMinusSourceAlpha);

        self.device
            .new_render_pipeline_state(&pipeline_descriptor)
            .map_err(|e| format!("Failed to create frame pipeline state: {}", e))
    }

    /// Convert text to vertices using shaped glyphs
    fn text_to_vertices(
        &self,
        position: glam::Vec2,
        shaped_text: &ShapedText,
        color: &Color,
        text_system: &TextSystem,
        screen_size: (f32, f32),
        scale_factor: f32,
    ) -> Vec<Vertex> {
        let mut vertices = Vec::new();
        let color_array = [color.red, color.green, color.blue, color.alpha];

        for glyph in &shaped_text.glyphs {
            if let Some(info) = text_system.glyph_info(glyph.font_id, glyph.glyph_id, glyph.size) {
                // Calculate glyph position in screen space
                // glyph.position is the baseline position from the shaper
                // info.bearing_y is the distance from baseline to top of glyph
                let glyph_x = position.x + glyph.position.x + info.left as f32;
                let glyph_y = position.y + glyph.position.y - info.top as f32;

                // Convert to NDC
                // Note: glyph positions are in logical pixels, screen_size is in logical pixels
                let physical_width = screen_size.0 * scale_factor;
                let physical_height = screen_size.1 * scale_factor;
                let x1 = (glyph_x * scale_factor / physical_width) * 2.0 - 1.0;
                let y1 = 1.0 - (glyph_y * scale_factor / physical_height) * 2.0;
                let x2 =
                    ((glyph_x + info.width as f32) * scale_factor / physical_width) * 2.0 - 1.0;
                let y2 =
                    1.0 - ((glyph_y + info.height as f32) * scale_factor / physical_height) * 2.0;

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
    /// Convert draw list commands to vertex data
    fn draw_list_to_vertices(
        &self,
        draw_list: &DrawList,
        screen_size: (f32, f32),
        scale_factor: f32,
        text_system: &mut TextSystem,
    ) -> (Vec<Vertex>, Vec<Vertex>, Vec<(Rect, ElementStyle)>) {
        let _vertices_span = info_span!(
            "draw_list_to_vertices",
            command_count = draw_list.commands().len()
        )
        .entered();
        let mut solid_vertices = Vec::new();
        let mut text_vertices = Vec::new();
        let mut frames = Vec::new();

        for command in draw_list.commands() {
            match command {
                DrawCommand::Rect { rect, color } => {
                    // Convert rect to vertices (two triangles)
                    let vertices = self.rect_to_vertices(rect, *color, screen_size, scale_factor);
                    solid_vertices.extend_from_slice(&vertices);
                }
                DrawCommand::Frame { rect, style } => {
                    // Collect frames for separate rendering pass
                    frames.push((*rect, style.clone()));
                }
                DrawCommand::Text {
                    position,
                    text,
                    style,
                } => {
                    // Shape and render text
                    let text_config = crate::text_system::TextConfig {
                        font_stack: parley::FontStack::from("system-ui"),
                        size: style.size,
                        color: style.color.clone(),
                        weight: parley::FontWeight::NORMAL,
                        line_height: 1.2,
                    };
                    let shaped = {
                        let _shape_span = info_span!("shape_text", text_len = text.len()).entered();
                        text_system.shape_text(text, &text_config, None, scale_factor)
                    };
                    if let Ok(shaped) = shaped {
                        let color = &style.color;
                        let vertices = self.text_to_vertices(
                            *position,
                            &shaped,
                            color,
                            text_system,
                            screen_size,
                            scale_factor,
                        );
                        text_vertices.extend_from_slice(&vertices);
                    }
                }
                DrawCommand::PushClip { .. } | DrawCommand::PopClip => {
                    // TODO: Implement clipping
                }
            }
        }

        (solid_vertices, text_vertices, frames)
    }

    /// Convert a rect to 6 vertices (two triangles)
    fn rect_to_vertices(
        &self,
        rect: &Rect,
        color: Color,
        screen_size: (f32, f32),
        scale_factor: f32,
    ) -> [Vertex; 6] {
        // Convert from screen coordinates to normalized device coordinates
        // Note: positions are in logical pixels, screen_size is in logical pixels
        // We need to convert to physical pixels for proper NDC calculation
        let physical_width = screen_size.0 * scale_factor;
        let physical_height = screen_size.1 * scale_factor;
        let x1 = (rect.pos.x() * scale_factor / physical_width) * 2.0 - 1.0;
        let y1 = 1.0 - (rect.pos.y() * scale_factor / physical_height) * 2.0;
        let x2 = ((rect.pos.x() + rect.size.width()) * scale_factor / physical_width) * 2.0 - 1.0;
        let y2 = 1.0 - ((rect.pos.y() + rect.size.height()) * scale_factor / physical_height) * 2.0;

        let color_array = [color.red, color.green, color.blue, color.alpha];

        // Two triangles to make a rectangle
        [
            Vertex {
                position: [x1, y1],
                color: color_array,
                tex_coord: [0.0, 0.0],
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

    /// Convert a frame rect to vertices with texture coordinates for SDF rendering
    fn frame_to_vertices(
        &self,
        rect: &Rect,
        style: &ElementStyle,
        screen_size: (f32, f32),
        scale_factor: f32,
    ) -> ([Vertex; 6], FrameUniforms) {
        // Expand bounds for shadow if present
        let (shadow_expand_left, shadow_expand_right, shadow_expand_top, shadow_expand_bottom) =
            if let Some(shadow) = &style.shadow {
                // Expand by blur radius plus offset to ensure shadow is fully visible
                let blur = shadow.blur;
                (
                    blur - shadow.offset.x.min(0.0), // left expansion
                    blur + shadow.offset.x.max(0.0), // right expansion
                    blur - shadow.offset.y.min(0.0), // top expansion
                    blur + shadow.offset.y.max(0.0), // bottom expansion
                )
            } else {
                (0.0, 0.0, 0.0, 0.0)
            };

        // Convert to clip space (-1 to 1) with shadow expansion
        // Note: positions are in logical pixels, screen_size is in logical pixels
        let physical_width = screen_size.0 * scale_factor;
        let physical_height = screen_size.1 * scale_factor;
        let x1 = ((rect.pos.x() - shadow_expand_left) * scale_factor / physical_width) * 2.0 - 1.0;
        let y1 = 1.0 - ((rect.pos.y() - shadow_expand_top) * scale_factor / physical_height) * 2.0;
        let x2 = ((rect.pos.x() + rect.size.width() + shadow_expand_right) * scale_factor / physical_width)
            * 2.0
            - 1.0;
        let y2 = 1.0
            - ((rect.pos.y() + rect.size.height() + shadow_expand_bottom) * scale_factor / physical_height)
                * 2.0;

        // For frames, we use a dummy color since actual colors come from uniforms
        let color_array = [1.0, 1.0, 1.0, 1.0];

        // Calculate texture coordinates that map the expanded bounds correctly
        // We need to map so that the original rect bounds are at the correct position
        let u0 = -shadow_expand_left / rect.size.width();
        let v0 = -shadow_expand_top / rect.size.height();
        let u1 = 1.0 + shadow_expand_right / rect.size.width();
        let v1 = 1.0 + shadow_expand_bottom / rect.size.height();

        // Create vertices
        let vertices = [
            Vertex {
                position: [x1, y1],
                color: color_array,
                tex_coord: [u0, v0],
            },
            Vertex {
                position: [x2, y1],
                color: color_array,
                tex_coord: [u1, v0],
            },
            Vertex {
                position: [x1, y2],
                color: color_array,
                tex_coord: [u0, v1],
            },
            Vertex {
                position: [x2, y1],
                color: color_array,
                tex_coord: [u1, v0],
            },
            Vertex {
                position: [x2, y2],
                color: color_array,
                tex_coord: [u1, v1],
            },
            Vertex {
                position: [x1, y2],
                color: color_array,
                tex_coord: [u0, v1],
            },
        ];

        // Create uniforms
        let uniforms = FrameUniforms {
            center: [
                rect.pos.x() + rect.size.width() / 2.0,
                rect.pos.y() + rect.size.height() / 2.0,
            ],
            half_size: [rect.size.width() / 2.0, rect.size.height() / 2.0],
            radii: [
                style.corner_radii.top_left,
                style.corner_radii.top_right,
                style.corner_radii.bottom_right,
                style.corner_radii.bottom_left,
            ],
            border_width: style.border_width,
            fill_type: match &style.fill {
                Fill::Solid(_) => 0,
                Fill::LinearGradient { .. } => 1,
                Fill::RadialGradient { .. } => 2,
            },
            gradient_angle: if let Fill::LinearGradient { angle, .. } = &style.fill {
                *angle
            } else {
                0.0
            },
            _padding: 0.0,
            color1: match &style.fill {
                Fill::Solid(color) => [color.red, color.green, color.blue, color.alpha],
                Fill::LinearGradient { start, .. } => {
                    [start.red, start.green, start.blue, start.alpha]
                }
                Fill::RadialGradient { center, .. } => {
                    [center.red, center.green, center.blue, center.alpha]
                }
            },
            color2: match &style.fill {
                Fill::Solid(color) => [color.red, color.green, color.blue, color.alpha],
                Fill::LinearGradient { end, .. } => [end.red, end.green, end.blue, end.alpha],
                Fill::RadialGradient { edge, .. } => [edge.red, edge.green, edge.blue, edge.alpha],
            },
            border_color: [
                style.border_color.red,
                style.border_color.green,
                style.border_color.blue,
                style.border_color.alpha,
            ],
            shadow_offset: if let Some(shadow) = &style.shadow {
                [shadow.offset.x, shadow.offset.y]
            } else {
                [0.0, 0.0]
            },
            shadow_blur: if let Some(shadow) = &style.shadow {
                shadow.blur
            } else {
                0.0
            },
            _padding2: 0.0,
            shadow_color: if let Some(shadow) = &style.shadow {
                [
                    shadow.color.red,
                    shadow.color.green,
                    shadow.color.blue,
                    shadow.color.alpha,
                ]
            } else {
                [0.0, 0.0, 0.0, 0.0]
            },
        };

        (vertices, uniforms)
    }

    /// Create a vertex buffer from vertices
    fn create_vertex_buffer(&self, vertices: &[Vertex]) -> Buffer {
        let vertex_data = vertices.as_ptr() as *const c_void;
        let vertex_data_size = (vertices.len() * mem::size_of::<Vertex>()) as u64;
        self.device.new_buffer_with_data(
            vertex_data,
            vertex_data_size,
            metal::MTLResourceOptions::CPUCacheModeDefaultCache,
        )
    }

    /// Render draw commands to an existing render encoder
    fn render_draw_list_with_encoder(
        &mut self,
        draw_list: &DrawList,
        encoder: &metal::RenderCommandEncoderRef,
        screen_size: (f32, f32),
        scale_factor: f32,
        text_system: &mut TextSystem,
    ) {
        let _encoder_span = info_span!("render_with_encoder").entered();
        // Get pipeline states
        let Some(pipeline_state) = &self.pipeline_state else {
            eprintln!("Pipeline state not initialized");
            return;
        };
        let Some(text_pipeline_state) = &self.text_pipeline_state else {
            eprintln!("Text pipeline state not initialized");
            return;
        };

        // Convert draw commands to vertices
        let (solid_vertices, text_vertices, frames) = {
            let _convert_span = info_span!("convert_draw_commands_to_vertices").entered();
            self.draw_list_to_vertices(draw_list, screen_size, scale_factor, text_system)
        };

        debug!(
            "Converted to {} solid vertices, {} text vertices, {} frames",
            solid_vertices.len(),
            text_vertices.len(),
            frames.len()
        );

        // Draw solid geometry first
        if !solid_vertices.is_empty() {
            let _solid_span =
                info_span!("draw_solid_geometry", vertex_count = solid_vertices.len()).entered();
            let buffer = self.create_vertex_buffer(&solid_vertices);
            encoder.set_render_pipeline_state(&pipeline_state);
            encoder.set_vertex_buffer(0, Some(&buffer), 0);
            encoder.draw_primitives(MTLPrimitiveType::Triangle, 0, solid_vertices.len() as u64);
        }

        // Draw text geometry with texture
        if !text_vertices.is_empty() {
            let _text_span =
                info_span!("draw_text_geometry", vertex_count = text_vertices.len()).entered();
            let buffer = self.create_vertex_buffer(&text_vertices);
            let texture = text_system.atlas_texture();
            encoder.set_render_pipeline_state(&text_pipeline_state);
            encoder.set_vertex_buffer(0, Some(&buffer), 0);
            encoder.set_fragment_texture(0, Some(texture));

            // Create and set sampler state
            let sampler_descriptor = metal::SamplerDescriptor::new();
            sampler_descriptor.set_min_filter(metal::MTLSamplerMinMagFilter::Linear);
            sampler_descriptor.set_mag_filter(metal::MTLSamplerMinMagFilter::Linear);
            let sampler_state = self.device.new_sampler(&sampler_descriptor);
            encoder.set_fragment_sampler_state(0, Some(&sampler_state));

            encoder.draw_primitives(MTLPrimitiveType::Triangle, 0, text_vertices.len() as u64);
        }

        // Draw frames with SDF rendering
        if !frames.is_empty() {
            let _frames_span = info_span!("draw_frames", frame_count = frames.len()).entered();
            encoder.set_render_pipeline_state(self.frame_pipeline_state.as_ref().unwrap());

            for (rect, style) in frames {
                // Create frame vertices with proper texture coordinates for SDF
                let (vertices, uniforms) =
                    self.frame_to_vertices(&rect, &style, screen_size, scale_factor);
                let vertex_buffer = self.create_vertex_buffer(&vertices);

                // Create uniforms buffer
                let uniforms_buffer = self.device.new_buffer_with_data(
                    &uniforms as *const _ as *const _,
                    std::mem::size_of::<FrameUniforms>() as u64,
                    metal::MTLResourceOptions::CPUCacheModeDefaultCache,
                );

                encoder.set_vertex_buffer(0, Some(&vertex_buffer), 0);
                encoder.set_fragment_buffer(0, Some(&uniforms_buffer), 0);
                encoder.draw_primitives(MTLPrimitiveType::Triangle, 0, vertices.len() as u64);
            }
        }
    }

    /// Legacy render method for backwards compatibility
    pub fn render_frame(
        &mut self,
        command_queue: &CommandQueue,
        drawable: &metal::MetalDrawableRef,
        clear_color: metal::MTLClearColor,
        draw_list: &DrawList,
        screen_size: (f32, f32),
        scale_factor: f32,
        text_system: &mut TextSystem,
    ) {
        // Create command buffer
        let command_buffer = command_queue.new_command_buffer();

        // Set up render pass descriptor
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
        let encoder = command_buffer.new_render_command_encoder(&render_pass_descriptor);

        // Use the shared rendering logic
        self.render_draw_list_with_encoder(
            draw_list,
            encoder,
            screen_size,
            scale_factor,
            text_system,
        );

        // End encoding
        encoder.end_encoding();

        // Present drawable and commit
        command_buffer.present_drawable(&drawable);
        command_buffer.commit();
    }

    /// Render a draw list for the layer system
    pub fn render_draw_list(
        &mut self,
        draw_list: &DrawList,
        command_buffer: &CommandBufferRef,
        drawable: &metal::MetalDrawableRef,
        screen_size: (f32, f32),
        scale_factor: f32,
        text_system: &mut TextSystem,
        load_action: metal::MTLLoadAction,
        clear_color: metal::MTLClearColor,
    ) {
        let _render_span = info_span!(
            "metal_render_draw_list",
            commands = draw_list.commands().len()
        )
        .entered();
        // Create render pass descriptor
        let render_pass_descriptor = RenderPassDescriptor::new();
        let color_attachment = render_pass_descriptor
            .color_attachments()
            .object_at(0)
            .unwrap();
        color_attachment.set_texture(Some(drawable.texture()));
        color_attachment.set_load_action(load_action);
        color_attachment.set_clear_color(clear_color);
        color_attachment.set_store_action(MTLStoreAction::Store);

        // Create render encoder
        let encoder = command_buffer.new_render_command_encoder(&render_pass_descriptor);

        // Render the draw list
        self.render_draw_list_with_encoder(
            draw_list,
            encoder,
            screen_size,
            scale_factor,
            text_system,
        );

        // End encoding
        encoder.end_encoding();
    }

    /// Draw a fullscreen quad with a custom fragment shader
    pub fn draw_fullscreen_quad(
        &mut self,
        command_buffer: &CommandBufferRef,
        drawable: &metal::MetalDrawableRef,
        shader_source: &str,
        size: Vec2,
        time: f32,
    ) {
        info!(
            "draw_fullscreen_quad called with size: {:?}, time: {}",
            size, time
        );

        // Combine vertex and fragment shaders
        let full_shader = format!(
            r#"
            #include <metal_stdlib>
            using namespace metal;

            struct VertexOut {{
                float4 position [[position]];
                float2 uv;
            }};

            struct Uniforms {{
                float2 resolution;
                float time;
                float _padding;
            }};

            vertex VertexOut fullscreen_vertex(uint vid [[vertex_id]]) {{
                VertexOut out;

                // Generate fullscreen triangle
                float2 positions[3] = {{
                    float2(-1.0, -1.0),
                    float2( 3.0, -1.0),
                    float2(-1.0,  3.0)
                }};

                out.position = float4(positions[vid], 0.0, 1.0);
                out.uv = (positions[vid] + 1.0) * 0.5;
                out.uv.y = 1.0 - out.uv.y; // Flip Y

                return out;
            }}

            {}

            fragment float4 custom_fragment(VertexOut in [[stage_in]],
                                          constant Uniforms &uniforms [[buffer(0)]]) {{
                return shader_main(in.uv, uniforms.resolution, uniforms.time);
            }}
            "#,
            shader_source
        );

        // Compile shader
        let options = metal::CompileOptions::new();
        info!("Compiling shader...");
        let library = match self.device.new_library_with_source(&full_shader, &options) {
            Ok(lib) => {
                info!("Shader compiled successfully!");
                lib
            }
            Err(e) => {
                eprintln!("Failed to compile custom shader: {}", e);
                eprintln!("Full shader source:\n{}", full_shader);
                return;
            }
        };

        // Create pipeline state
        let vert_func = library.get_function("fullscreen_vertex", None).unwrap();
        let frag_func = library.get_function("custom_fragment", None).unwrap();

        let pipeline_descriptor = RenderPipelineDescriptor::new();
        pipeline_descriptor.set_vertex_function(Some(&vert_func));
        pipeline_descriptor.set_fragment_function(Some(&frag_func));

        let attachment = pipeline_descriptor
            .color_attachments()
            .object_at(0)
            .unwrap();
        attachment.set_pixel_format(metal::MTLPixelFormat::BGRA8Unorm);
        attachment.set_blending_enabled(true);
        attachment.set_source_rgb_blend_factor(metal::MTLBlendFactor::SourceAlpha);
        attachment.set_destination_rgb_blend_factor(metal::MTLBlendFactor::OneMinusSourceAlpha);

        let pipeline_state = match self.device.new_render_pipeline_state(&pipeline_descriptor) {
            Ok(state) => state,
            Err(e) => {
                eprintln!("Failed to create pipeline state: {}", e);
                return;
            }
        };

        // Create uniforms
        #[repr(C)]
        struct Uniforms {
            resolution: [f32; 2],
            time: f32,
            _padding: f32,
        }

        let uniforms = Uniforms {
            resolution: [size.x, size.y],
            time,
            _padding: 0.0,
        };

        let uniforms_buffer = self.device.new_buffer_with_data(
            &uniforms as *const _ as *const _,
            std::mem::size_of::<Uniforms>() as u64,
            metal::MTLResourceOptions::CPUCacheModeDefaultCache,
        );

        // Create render pass descriptor
        let render_pass_descriptor = RenderPassDescriptor::new();
        let color_attachment = render_pass_descriptor
            .color_attachments()
            .object_at(0)
            .unwrap();
        color_attachment.set_texture(Some(drawable.texture()));
        color_attachment.set_load_action(MTLLoadAction::Load);
        color_attachment.set_store_action(MTLStoreAction::Store);

        // Create render encoder
        let encoder = command_buffer.new_render_command_encoder(&render_pass_descriptor);
        encoder.set_render_pipeline_state(&pipeline_state);
        encoder.set_fragment_buffer(0, Some(&uniforms_buffer), 0);

        // Draw fullscreen triangle
        info!("Drawing fullscreen triangle...");
        encoder.draw_primitives(MTLPrimitiveType::Triangle, 0, 3);
        encoder.end_encoding();
        info!("Fullscreen quad rendered");
    }
}
