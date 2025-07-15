//! Simple shader example demonstrating raw layers

use sol_ui::{
    app::app,
    layer::{LayerManager, LayerOptions},
};

fn main() {
    app()
        .title("Toy UI - Simple Shader Example")
        .size(800.0, 600.0)
        .with_layers(|layer_manager: &mut LayerManager| {
            // Single raw layer with a simple gradient shader
            layer_manager.add_raw_layer(0, LayerOptions::default(), |ctx| {
                // Simple gradient shader
                let gradient_shader = r#"
                    float4 shader_main(float2 uv, float2 resolution, float time) {
                        // Simple animated gradient
                        float3 color = mix(
                            float3(0.1, 0.2, 0.5),  // Deep blue
                            float3(1.0, 0.5, 0.2),  // Orange
                            uv.y + 0.2 * sin(time + uv.x * 5.0)
                        );
                        return float4(color, 1.0);
                    }
                "#;

                ctx.draw_fullscreen_quad(gradient_shader);
            });
        })
        .run();
}
