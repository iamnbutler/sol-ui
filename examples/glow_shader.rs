//! Glow shader example demonstrating luminous effects

use sol_ui::{
    app::app,
    layer::{LayerManager, LayerOptions},
};

fn main() {
    app()
        .title("Toy UI - Glow Shader Example")
        .size(800.0, 600.0)
        .with_layers(|layer_manager: &mut LayerManager| {
            // Raw layer with a glow shader effect
            layer_manager.add_raw_layer(0, LayerOptions::default(), |ctx| {
                // Glow shader with animated pulsing orbs
                let glow_shader = r#"
                    float4 shader_main(float2 uv, float2 resolution, float time) {
                        // Center UV coordinates
                        float2 center_uv = uv - 0.5;
                        center_uv.x *= resolution.x / resolution.y; // Correct aspect ratio

                        // Create multiple glowing orbs
                        float3 color = float3(0.0);

                        // Orb 1: Blue-cyan glow
                        float2 orb1_pos = float2(0.3 * sin(time * 0.8), 0.2 * cos(time * 0.6));
                        float dist1 = length(center_uv - orb1_pos);
                        float glow1 = 0.015 / dist1;
                        color += float3(0.2, 0.6, 1.0) * glow1;

                        // Orb 2: Purple-magenta glow
                        float2 orb2_pos = float2(-0.25 * cos(time * 0.5), -0.15 * sin(time * 0.7));
                        float dist2 = length(center_uv - orb2_pos);
                        float glow2 = 0.012 / dist2;
                        color += float3(0.8, 0.2, 1.0) * glow2;

                        // Orb 3: Green-yellow glow
                        float2 orb3_pos = float2(-0.2 * sin(time * 0.9), 0.25 * cos(time * 0.4));
                        float dist3 = length(center_uv - orb3_pos);
                        float glow3 = 0.01 / dist3;
                        color += float3(0.4, 1.0, 0.3) * glow3;

                        // Add subtle background gradient
                        float3 bg = mix(
                            float3(0.02, 0.01, 0.05),  // Dark purple
                            float3(0.01, 0.02, 0.08),  // Dark blue
                            uv.y
                        );
                        color += bg;

                        // Apply glow intensity with pulsing
                        float pulse = 0.8 + 0.2 * sin(time * 2.0);
                        color *= pulse;

                        return float4(color, 1.0);
                    }
                "#;

                ctx.draw_fullscreen_quad(glow_shader);
            });
        })
        .run();
}
