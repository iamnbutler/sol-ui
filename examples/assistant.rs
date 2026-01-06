use sol_ui::{
    app::app,
    element::{row, text},
    layer::{LayerManager, LayerOptions},
};

fn main() {
    app()
        .title("Toy UI - Assistant Example")
        .size(550.0, 750.0)
        .with_layers(|layer_manager: &mut LayerManager| {
            layer_manager.add_raw_layer(0, LayerOptions::default()
                .with_clear()
                .with_clear_color(0.113, 0.113, 0.113, 1.0), |ctx| {
                ctx.request_animation_frame();
                // Film grain noise shader on solid #292929 background
                let noise_shader = r#"
                    float4 shader_main(float2 uv, float2 resolution, float time) {
                        float3 bg = float3(0.113, 0.113, 0.113);

                        // Coarser grain: scale down resolution for lower-frequency noise
                        float2 coord = uv * resolution / 16.0;
                        float n = fract(sin(dot(coord + time * 100.0, float2(12.9898, 78.233))) * 43758.5453);

                        // Remap noise to ±1 to avoid washing out background
                        float noise_val = n * 2.0 - 1.0;
                        // Blend noise symmetrically around background
                        float3 color = bg + noise_val * 0.15;

                        return float4(color, 1.0);
                    }
                "#;

                ctx.draw_fullscreen_quad(noise_shader);
            });
let ui_layer_options: LayerOptions = LayerOptions::default()
                .with_input();

            layer_manager.add_ui_layer(10, ui_layer_options, move || {
                row().child(
                    row().child(text("assistant"))
                )
            });
        })
        .run();
}
