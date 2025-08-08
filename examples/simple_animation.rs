//! Simple animation example showing how to use request_animation_frame

use sol_ui::{
    app::app,
    layer::{LayerManager, LayerOptions},
};

fn main() {
    app()
        .title("Toy UI - Simple Animation")
        .size(800.0, 600.0)
        .with_layers(|layer_manager: &mut LayerManager| {
            // Add a raw layer with a simple animated shader
            layer_manager.add_raw_layer(0, LayerOptions::default(), |ctx| {
                // Request continuous animation
                ctx.request_animation_frame();

                // Simple pulsing circle animation
                let shader = r#"
                    float4 shader_main(float2 uv, float2 resolution, float time) {
                        // Center the coordinate system
                        float2 pos = uv - 0.5;
                        float aspect = resolution.x / resolution.y;
                        pos.x *= aspect;

                        // Animated radius
                        float radius = 0.2 + 0.1 * sin(time * 2.0);

                        // Distance from center
                        float dist = length(pos);

                        // Smooth circle
                        float circle = smoothstep(radius + 0.01, radius - 0.01, dist);

                        // Animated color
                        float3 color = mix(
                            float3(0.2, 0.3, 0.8),  // Blue
                            float3(0.8, 0.3, 0.2),  // Red
                            (sin(time) + 1.0) * 0.5
                        );

                        return float4(color * circle, 1.0);
                    }
                "#;

                ctx.draw_fullscreen_quad(shader);
            });
        })
        .run();
}
