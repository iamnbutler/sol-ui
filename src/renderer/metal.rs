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
}

pub struct MetalRenderer {
    device: Device,
    pipeline_state: Option<RenderPipelineState>,
}

impl MetalRenderer {
    pub fn new(device: Device) -> Self {
        Self {
            device,
            pipeline_state: None,
        }
    }

    pub fn initialize(&mut self) -> Result<(), String> {
        // Create shader library
        let library = self.compile_shaders()?;

        // Create pipeline state
        self.pipeline_state = Some(self.create_pipeline_state(&library)?);

        Ok(())
    }

    fn compile_shaders(&self) -> Result<Library, String> {
        let shader_source = r#"
            #include <metal_stdlib>
            using namespace metal;

            struct Vertex {
                float2 position [[attribute(0)]];
                float4 color [[attribute(1)]];
            };

            struct VertexOut {
                float4 position [[position]];
                float4 color;
            };

            vertex VertexOut vertex_main(Vertex in [[stage_in]]) {
                VertexOut out;
                out.position = float4(in.position, 0.0, 1.0);
                out.color = in.color;
                return out;
            }

            fragment float4 fragment_main(VertexOut in [[stage_in]]) {
                return in.color;
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

    /// Build vertices from UI draw commands
    fn build_vertices_from_draw_list(
        &self,
        draw_list: &DrawList,
        screen_size: (f32, f32),
    ) -> Vec<Vertex> {
        let mut vertices = Vec::new();

        for command in draw_list.commands() {
            match command {
                DrawCommand::Rect { rect, color } => {
                    // Convert from screen coordinates to normalized device coordinates
                    let vertices_for_rect = self.rect_to_vertices(rect, color, screen_size);
                    vertices.extend_from_slice(&vertices_for_rect);
                }
                DrawCommand::Text { .. } => {
                    // TODO: Implement text rendering
                }
                DrawCommand::PushClip { .. } | DrawCommand::PopClip => {
                    // TODO: Implement clipping
                }
            }
        }

        vertices
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
            },
            Vertex {
                position: [x2, y1],
                color: color_array,
            },
            Vertex {
                position: [x1, y2],
                color: color_array,
            },
            // Second triangle
            Vertex {
                position: [x2, y1],
                color: color_array,
            },
            Vertex {
                position: [x2, y2],
                color: color_array,
            },
            Vertex {
                position: [x1, y2],
                color: color_array,
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
    ) {
        let Some(pipeline_state) = &self.pipeline_state else {
            return;
        };

        // Build vertices from draw list
        let vertices = self.build_vertices_from_draw_list(draw_list, screen_size);
        if vertices.is_empty() {
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

        // Create vertex buffer
        let vertex_data = vertices.as_ptr() as *const c_void;
        let vertex_data_size = (vertices.len() * mem::size_of::<Vertex>()) as u64;
        let vertex_buffer = self.device.new_buffer_with_data(
            vertex_data,
            vertex_data_size,
            metal::MTLResourceOptions::CPUCacheModeDefaultCache,
        );

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

        // Set pipeline state
        render_encoder.set_render_pipeline_state(&pipeline_state);

        // Set vertex buffer
        render_encoder.set_vertex_buffer(0, Some(&vertex_buffer), 0);

        // Draw all triangles
        render_encoder.draw_primitives(MTLPrimitiveType::Triangle, 0, vertices.len() as u64);

        // End encoding
        render_encoder.end_encoding();

        // Present drawable and commit
        command_buffer.present_drawable(&drawable);
        command_buffer.commit();
    }
}
