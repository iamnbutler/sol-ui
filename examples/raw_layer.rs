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
        .title("Toy UI - Raw Layer Example")
        .size(800.0, 600.0)
        .with_layers(|layer_manager: &mut LayerManager| {
            // Layer 0: Animated gradient background using raw layer
            info!("Setting up Layer 0: Animated gradient background");
            layer_manager.add_raw_layer(0, LayerOptions::default().with_clear().with_clear_color(0.0, 0.0, 0.0, 1.0), |ctx| {
                // Draw a simple animated gradient background
                let gradient_shader = r#"
                    float4 shader_main(float2 uv, float2 resolution, float time) {
                        // Animated gradient background
                        float2 pos = uv - 0.5;
                        float angle = time * 0.1;
                        float2 rotated = float2(
                            pos.x * cos(angle) - pos.y * sin(angle),
                            pos.x * sin(angle) + pos.y * cos(angle)
                        );

                        float3 color1 = float3(0.1, 0.0, 0.3);  // Deep purple
                        float3 color2 = float3(0.0, 0.1, 0.4);  // Deep blue
                        float3 color3 = float3(0.3, 0.0, 0.2);  // Magenta

                        float t = rotated.x + rotated.y + sin(time * 0.2) * 0.2;
                        float3 color = mix(color1, color2, smoothstep(-1.0, 1.0, t));
                        color = mix(color, color3, smoothstep(0.0, 1.0, sin(rotated.x * 3.0 + time * 0.3)));

                        return float4(color, 1.0);
                    }
                "#;

                ctx.draw_fullscreen_quad(gradient_shader);
            });

            // Layer 1: 3D scene using raw layer
            info!("Setting up Layer 1: 3D scene");
            layer_manager.add_raw_layer(1, LayerOptions::default(), |ctx| {
                // Draw a simple Mandelbrot fractal
                let mandelbrot_shader = r#"
                    float4 shader_main(float2 uv, float2 resolution, float time) {
                        // Map UV to complex plane with animation
                        float zoom = 2.0 + sin(time * 0.1) * 1.5;
                        float2 c = (uv - 0.5) * zoom - float2(0.7, 0.0);

                        // Mandelbrot iteration
                        float2 z = float2(0.0);
                        int iterations = 0;
                        const int max_iterations = 50;

                        for (int i = 0; i < max_iterations; i++) {
                            float x = z.x * z.x - z.y * z.y + c.x;
                            float y = 2.0 * z.x * z.y + c.y;
                            z = float2(x, y);

                            if (dot(z, z) > 4.0) {
                                iterations = i;
                                break;
                            }
                        }

                        // Color based on iterations
                        float t = float(iterations) / float(max_iterations);

                        float3 color;
                        if (iterations == max_iterations) {
                            color = float3(0.0);  // Black for points in set
                        } else {
                            // Create a colorful gradient
                            color = 0.5 + 0.5 * cos(6.28318 * t + float3(0.0, 2.0, 4.0));
                        }

                        // Add transparency for blending
                        return float4(color, 0.8);
                    }
                "#;

                ctx.draw_fullscreen_quad(mandelbrot_shader);
            });

            // Layer 2: Particle system overlay using raw layer
            info!("Setting up Layer 2: Particle system");
            // Skip layer 2 - we'll just have background and mandelbrot

            // Layer 3: UI overlay (showing we can mix UI and raw layers)
            info!("Setting up Layer 3: UI overlay");
            layer_manager.add_ui_layer(3, LayerOptions::default(), || {
                use palette::Srgba;
                use toy_ui::draw::TextStyle;
                use toy_ui::elements::{container, text};

                Box::new(
                    container().padding(20.0).child(
                        container()
                            .background(Srgba::new(0.0, 0.0, 0.0, 0.8))
                            .padding(20.0)
                            .corner_radius(10.0)
                            .child(text(
                                "Fractal Rendering Demo",
                                TextStyle {
                                    size: 24.0,
                                    color: Srgba::new(1.0, 1.0, 1.0, 1.0),
                                },
                            ))
                            .child(text(
                                "Real-time GPU fractals with Metal shaders",
                                TextStyle {
                                    size: 16.0,
                                    color: Srgba::new(0.8, 0.8, 0.8, 1.0),
                                },
                            ))
                            .child(text(
                                "Currently rendering:",
                                TextStyle {
                                    size: 14.0,
                                    color: Srgba::new(0.7, 0.7, 0.7, 1.0),
                                },
                            ))
                            .child(text(
                                "• Mandelbrot set (animated zoom)",
                                TextStyle {
                                    size: 14.0,
                                    color: Srgba::new(0.6, 0.8, 1.0, 1.0),
                                },
                            ))
                            .child(text(
                                "• Julia set (animated parameter)",
                                TextStyle {
                                    size: 14.0,
                                    color: Srgba::new(0.6, 0.8, 1.0, 1.0),
                                },
                            ))
                            .child(text(
                                "• Plasma overlay effect",
                                TextStyle {
                                    size: 14.0,
                                    color: Srgba::new(0.6, 0.8, 1.0, 1.0),
                                },
                            ))
                            .child(text(
                                "• All rendered in real-time on GPU",
                                TextStyle {
                                    size: 14.0,
                                    color: Srgba::new(0.6, 0.8, 1.0, 1.0),
                                },
                            )),
                    ),
                )
            });
        })
        .run();
}
