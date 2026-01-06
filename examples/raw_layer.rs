//! Example demonstrating raw layers with custom shader code

use sol_ui::{
    app::app,
    layer::{LayerManager, LayerOptions},
};
use tracing::{info, info_span};
use tracing_subscriber::{EnvFilter, fmt};

fn main() {
    // Initialize tracing
    let _subscriber = fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("sol_ui=info")),
        )
        .with_target(true)
        .with_thread_ids(true)
        .with_timer(fmt::time::uptime())
        .init();

    info!("Starting Raw Layer Example");
    let _main_span = info_span!("main").entered();

    app()
        .title("Toy UI - Cosmic Energy Visualization")
        .size(800.0, 600.0)
        .with_layers(|layer_manager: &mut LayerManager| {
            // Layer 0: Animated gradient background using raw layer
            info!("Setting up Layer 0: Animated gradient background");
            layer_manager.add_raw_layer(0, LayerOptions::default().with_clear().with_clear_color(0.0, 0.0, 0.0, 1.0), |ctx| {
                ctx.request_animation_frame();

                // Draw an animated aurora background
                let aurora_shader = r#"
                    float4 shader_main(float2 uv, float2 resolution, float time) {
                        float2 pos = (uv - 0.5) * 2.0;

                        // Create flowing aurora waves
                        float wave1 = sin(pos.x * 3.0 + time * 0.7) * cos(pos.y * 2.0 + time * 0.3);
                        float wave2 = sin(pos.x * 2.0 - time * 0.5) * sin(pos.y * 3.0 + time * 0.4);
                        float wave3 = cos(pos.x * 4.0 + pos.y * 2.0 + time * 0.6);

                        float aurora = (wave1 + wave2 + wave3) / 3.0;
                        aurora = aurora * 0.5 + 0.5; // Normalize to 0-1

                        // Height-based intensity
                        float height_factor = 1.0 - abs(pos.y * 0.7);
                        height_factor = pow(height_factor, 2.0);
                        aurora *= height_factor;

                        // Deep space background gradient
                        float3 bg_color = mix(
                            float3(0.0, 0.0, 0.02),  // Almost black
                            float3(0.0, 0.02, 0.05), // Very dark blue
                            uv.y
                        );

                        // Aurora colors - cool palette
                        float3 aurora_color1 = float3(0.0, 0.8, 0.6);  // Cyan-green
                        float3 aurora_color2 = float3(0.0, 0.4, 0.8);  // Blue
                        float3 aurora_color3 = float3(0.4, 0.0, 0.6);  // Purple

                        // Mix aurora colors
                        float3 aurora_final = mix(aurora_color1, aurora_color2, sin(aurora * 3.14159 + time * 0.2));
                        aurora_final = mix(aurora_final, aurora_color3, cos(aurora * 2.0 - time * 0.3));

                        // Combine with background
                        float3 color = bg_color + aurora_final * aurora * 0.6;

                        // Add subtle stars
                        float star = pow(fract(sin(dot(uv * 200.0, float2(12.9898, 78.233))) * 43758.5453), 40.0);
                        color += float3(0.9, 0.9, 1.0) * star;

                        return float4(color, 1.0);
                    }
                "#;

                ctx.draw_fullscreen_quad(aurora_shader);
            });

            // Layer 1: 3D scene using raw layer
            info!("Setting up Layer 1: 3D scene");
            layer_manager.add_raw_layer(1, LayerOptions::default(), |ctx| {
                // Request continuous animation
                ctx.request_animation_frame();

                // Draw flowing cosmic ribbons
                let cosmic_ribbons_shader = r#"
                    float4 shader_main(float2 uv, float2 resolution, float time) {
                        float2 pos = (uv - 0.5) * float2(resolution.x / resolution.y, 1.0) * 2.0;
                        float3 color = float3(0.0);

                        // Create flowing ribbon paths
                        for (int i = 0; i < 3; i++) {
                            float fi = float(i);
                            float speed = 0.5 + fi * 0.2;

                            // Ribbon path
                            float y_offset = sin(pos.x * 2.0 + time * speed + fi * 2.0) * 0.3;
                            y_offset += cos(pos.x * 3.0 - time * speed * 0.7 + fi) * 0.2;

                            // Distance from ribbon center
                            float dist = abs(pos.y - y_offset - (fi - 1.0) * 0.5);

                            // Ribbon intensity with smooth edges
                            float ribbon = exp(-dist * dist * 30.0);

                            // Pulsing along the ribbon
                            float pulse = sin(pos.x * 5.0 - time * 2.0 + fi * 3.0) * 0.5 + 0.5;
                            ribbon *= 0.7 + pulse * 0.3;

                            // Color each ribbon differently
                            float3 ribbon_color;
                            if (i == 0) ribbon_color = float3(0.3, 0.7, 1.0);  // Light blue
                            else if (i == 1) ribbon_color = float3(0.5, 0.3, 0.9);  // Purple-blue
                            else ribbon_color = float3(0.2, 0.9, 0.8);  // Cyan

                            color += ribbon_color * ribbon * 0.8;
                        }

                        // Add some cosmic dust/glow
                        float glow = 0.02 / (length(pos) + 0.1);
                        color += float3(0.4, 0.5, 0.8) * glow;

                        // Subtle vignette
                        float vignette = 1.0 - length(pos * 0.5) * 0.7;
                        color *= vignette;

                        return float4(color, 0.5);
                    }
                "#;

                ctx.draw_fullscreen_quad(cosmic_ribbons_shader);
            });

            // Layer 4: Film grain and post effects
            info!("Setting up Layer 4: Film grain");
            layer_manager.add_raw_layer(4, LayerOptions::default(), |ctx| {
                ctx.request_animation_frame();

                // Film grain and subtle post effects
                let film_grain_shader = r#"
                    float rand(float2 co) {
                        return fract(sin(dot(co.xy, float2(12.9898, 78.233))) * 43758.5453);
                    }

                    float4 shader_main(float2 uv, float2 resolution, float time) {
                        // Film grain
                        float grain_amount = 0.03;
                        float grain = rand(uv * resolution + time) * grain_amount - grain_amount * 0.5;

                        // Animated noise pattern for organic feel
                        float2 noise_uv = uv * 50.0 + time * 10.0;
                        float noise_pattern = rand(noise_uv) * 0.02;

                        // Subtle vignette
                        float2 pos = uv - 0.5;
                        float vignette = 1.0 - length(pos) * 0.4;
                        vignette = smoothstep(0.0, 1.0, vignette);

                        // Combine effects
                        float3 color = float3(grain + noise_pattern) * vignette;

                        // Slight color tint to the grain
                        color *= float3(0.9, 0.95, 1.0);

                        return float4(color, grain_amount * vignette);
                    }
                "#;

                ctx.draw_fullscreen_quad(film_grain_shader);
            });

            // Layer 3: Particle system overlay
            info!("Setting up Layer 3: Particle system");
            layer_manager.add_raw_layer(3, LayerOptions::default(), |ctx| {
                // Request continuous animation
                ctx.request_animation_frame();

                // Draw small glowing particles
                let particle_shader = r#"
                    float hash(float2 p) {
                        return fract(sin(dot(p, float2(127.1, 311.7))) * 43758.5453);
                    }

                    float4 shader_main(float2 uv, float2 resolution, float time) {
                        float3 color = float3(0.0);

                        // More particles but much smaller
                        const int num_particles = 100;

                        for (int i = 0; i < num_particles; i++) {
                            float fi = float(i);

                            // Generate particle properties
                            float2 seed = float2(fi * 0.1, fi * 0.2);
                            float rand1 = hash(seed);
                            float rand2 = hash(seed + 1.0);
                            float rand3 = hash(seed + 2.0);
                            float rand4 = hash(seed + 3.0);
                            float rand5 = hash(seed + 4.0);

                            // Drifting motion
                            float2 base_pos = float2(rand1, rand2);
                            float2 drift = float2(
                                sin(time * 0.3 + fi * 1.7) * 0.1,
                                cos(time * 0.2 + fi * 2.3) * 0.15 + time * 0.05
                            );
                            float2 particle_pos = fract(base_pos + drift);

                            // Distance from current pixel
                            float2 diff = (uv - particle_pos) * resolution;
                            float dist = length(diff);

                            // Much smaller particles
                            float pulse = sin(time * 4.0 + fi * 3.0) * 0.2 + 0.8;
                            float glow_size = (0.5 + rand4 * 0.5) * pulse; // Much smaller

                            // Sharp, bright points
                            float glow = exp(-dist * dist / (glow_size * glow_size));

                            // Cool color palette matching aurora
                            float3 particle_color;
                            float color_choice = rand5;
                            if (color_choice < 0.3) particle_color = float3(0.4, 0.9, 1.0);     // Light cyan
                            else if (color_choice < 0.6) particle_color = float3(0.6, 0.8, 1.0); // Light blue
                            else if (color_choice < 0.8) particle_color = float3(0.8, 0.7, 1.0); // Light purple
                            else particle_color = float3(1.0, 0.9, 0.8);                         // Warm white

                            // Brightness variation
                            float brightness = 0.5 + rand3 * 0.5;
                            particle_color *= brightness;

                            // Fade near edges
                            float edge_fade = smoothstep(0.0, 0.1, particle_pos.x) *
                                            smoothstep(1.0, 0.9, particle_pos.x) *
                                            smoothstep(0.0, 0.1, particle_pos.y) *
                                            smoothstep(1.0, 0.9, particle_pos.y);

                            color += particle_color * glow * edge_fade * 2.0;
                        }

                        // Very subtle bloom
                        color = pow(color, float3(0.95));

                        // Alpha based on luminance
                        float alpha = min(length(color), 1.0);
                        return float4(color, alpha);
                    }
                "#;

                ctx.draw_fullscreen_quad(particle_shader);
            });

        })
        .run();
}
