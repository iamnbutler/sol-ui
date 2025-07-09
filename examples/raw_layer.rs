//! Example demonstrating raw layers with custom shader code

use toy_ui::{
    app,
    layer::{LayerManager, LayerOptions},
};
use tracing::{info, info_span};
use tracing_subscriber::{EnvFilter, fmt};

fn main() {
    // Initialize tracing
    let _subscriber = fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("toy_ui=info")),
        )
        .with_target(true)
        .with_thread_ids(true)
        .with_timer(fmt::time::uptime())
        .init();

    info!("Starting Raw Layer Example");
    let _main_span = info_span!("main").entered();

    app()
        .title("Toy UI - Animated Fractals Demo")
        .size(800.0, 600.0)
        .with_layers(|layer_manager: &mut LayerManager| {
            // Layer 0: Animated gradient background using raw layer
            info!("Setting up Layer 0: Animated gradient background");
            layer_manager.add_raw_layer(0, LayerOptions::default().with_clear().with_clear_color(0.0, 0.0, 0.0, 1.0), |ctx| {
                ctx.request_animation_frame();

                let plasma_shader = r#"
                    float4 shader_main(float2 uv, float2 resolution, float time) {
                        float2 pos = (uv - 0.5) * 3.0;

                        float pattern1 = sin(pos.x * 4.0 + time * 2.0) * cos(pos.y * 3.0 + time * 1.5);
                        float pattern2 = sin(distance(pos, float2(sin(time * 0.8), cos(time * 0.6))) * 5.0 - time * 3.0);
                        float pattern3 = sin(pos.x * sin(time * 0.5) * 3.0 + pos.y * cos(time * 0.3) * 4.0);

                        float plasma = (pattern1 + pattern2 + pattern3) / 3.0;

                        float3 color1 = float3(0.1, 0.0, 0.2);  // Deep purple
                        float3 color2 = float3(0.0, 0.1, 0.3);  // Deep blue
                        float3 color3 = float3(0.2, 0.0, 0.3);  // Magenta
                        float3 color4 = float3(0.0, 0.2, 0.4);  // Teal

                        float t = (sin(plasma * 3.14159 + time) + 1.0) * 0.5;
                        float3 color = mix(color1, color2, smoothstep(0.0, 0.33, t));
                        color = mix(color, color3, smoothstep(0.33, 0.66, t));
                        color = mix(color, color4, smoothstep(0.66, 1.0, t));

                        color += float3(0.1, 0.05, 0.15) * abs(plasma);

                        return float4(color, 1.0);
                    }
                "#;

                ctx.draw_fullscreen_quad(plasma_shader);
            });

            // Layer 1: 3D scene using raw layer
            info!("Setting up Layer 1: 3D scene");
            layer_manager.add_raw_layer(1, LayerOptions::default(), |ctx| {
                // Request continuous animation
                ctx.request_animation_frame();

                // Draw an animated Julia set fractal
                let julia_shader = r#"
                    float4 shader_main(float2 uv, float2 resolution, float time) {
                        // Julia set with animated parameter
                        float2 z = (uv - 0.5) * 3.0;

                        // Animate the Julia parameter in a loop
                        float angle = time * 0.5;
                        float radius = 0.7885;  // This radius creates interesting patterns
                        float2 c = float2(radius * cos(angle), radius * sin(angle));

                        int iterations = 0;
                        const int max_iterations = 80;
                        float escape_radius = 2.0;

                        for (int i = 0; i < max_iterations; i++) {
                            float x = z.x * z.x - z.y * z.y + c.x;
                            float y = 2.0 * z.x * z.y + c.y;
                            z = float2(x, y);

                            if (length(z) > escape_radius) {
                                iterations = i;
                                break;
                            }
                        }

                        // Smooth coloring
                        float smooth_iter = float(iterations);
                        if (iterations < max_iterations) {
                            float log_zn = log(length(z));
                            float nu = log(log_zn / log(2.0)) / log(2.0);
                            smooth_iter = float(iterations) + 1.0 - nu;
                        }

                        // Create a beautiful color gradient
                        float t = smooth_iter / float(max_iterations);

                        float3 color;
                        if (iterations == max_iterations) {
                            // Points inside the Julia set - dark blue
                            color = float3(0.0, 0.05, 0.1);
                        } else {
                            // Create a multi-color gradient
                            float h = t * 6.0 + time * 0.1;  // Slowly shift colors
                            float3 c1 = float3(0.5, 0.0, 0.5);  // Purple
                            float3 c2 = float3(0.0, 0.3, 0.7);  // Blue
                            float3 c3 = float3(0.0, 0.7, 0.7);  // Cyan
                            float3 c4 = float3(0.9, 0.5, 0.0);  // Orange

                            float segment = fmod(h, 4.0);
                            if (segment < 1.0) {
                                color = mix(c1, c2, segment);
                            } else if (segment < 2.0) {
                                color = mix(c2, c3, segment - 1.0);
                            } else if (segment < 3.0) {
                                color = mix(c3, c4, segment - 2.0);
                            } else {
                                color = mix(c4, c1, segment - 3.0);
                            }

                            // Add brightness variation based on escape time
                            color *= 0.5 + 0.5 * sin(smooth_iter * 0.5);
                        }

                        // Add slight transparency for blending
                        return float4(color, 0.85);
                    }
                "#;

                ctx.draw_fullscreen_quad(julia_shader);
            });

            // Layer 2: Particle system overlay using raw layer
            info!("Setting up Layer 2: Particle system");
            // Skip layer 2 - we'll just have background and mandelbrot

            // Layer 3: UI overlay (showing we can mix UI and raw layers)
            // info!("Setting up Layer 3: UI overlay");
            // layer_manager.add_ui_layer(3, LayerOptions::default(), || {
            //     use palette::Srgba;
            //     use toy_ui::draw::TextStyle;
            //     use toy_ui::elements::{container, text};

            //     Box::new(
            //         container().padding(20.0).child(
            //             container()
            //                 .background(Srgba::new(0.0, 0.0, 0.0, 0.8))
            //                 .padding(20.0)
            //                 .corner_radius(10.0)
            //                 .flex_col()
            //                 .gap(8.0)
            //                 .child(text(
            //                     "Fractal Rendering Demo",
            //                     TextStyle {
            //                         size: 12.0,
            //                         color: Srgba::new(1.0, 1.0, 1.0, 1.0),
            //                     },
            //                 ))
            //                 .child(text(
            //                     "Real-time GPU fractals with Metal shaders",
            //                     TextStyle {
            //                         size: 8.0,
            //                         color: Srgba::new(0.8, 0.8, 0.8, 1.0),
            //                     },
            //                 ))
            //                 .child(text(
            //                     "Currently rendering:",
            //                     TextStyle {
            //                         size: 7.0,
            //                         color: Srgba::new(0.7, 0.7, 0.7, 1.0),
            //                     },
            //                 ))
            //                 .child(text(
            //                     "• Animated plasma background",
            //                     TextStyle {
            //                         size: 7.0,
            //                         color: Srgba::new(0.6, 0.8, 1.0, 1.0),
            //                     },
            //                 ))
            //                 .child(text(
            //                     "• Julia set fractal (rotating parameter)",
            //                     TextStyle {
            //                         size: 7.0,
            //                         color: Srgba::new(0.6, 0.8, 1.0, 1.0),
            //                     },
            //                 ))
            //                 .child(text(
            //                     "• Smooth color gradients",
            //                     TextStyle {
            //                         size: 7.0,
            //                         color: Srgba::new(0.6, 0.8, 1.0, 1.0),
            //                     },
            //                 ))
            //                 .child(text(
            //                     "• All rendered in real-time on GPU",
            //                     TextStyle {
            //                         size: 7.0,
            //                         color: Srgba::new(0.6, 0.8, 1.0, 1.0),
            //                     },
            //                 )),
            //         ),
            //     )
            // });
        })
        .run();
}
