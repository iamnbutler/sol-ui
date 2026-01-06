//! README Counter Example
//!
//! A simple counter app demonstrating:
//! - Raw layer with animated shader background
//! - UI layer with Button elements
//! - Shared state via Rc<RefCell<>>

use sol_ui::{
    app::app,
    color::colors,
    element::{button, container, row, text},
    layer::LayerOptions,
    style::TextStyle,
};
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let counter = Rc::new(RefCell::new(0));

    app()
        .title("Sol UI - Counter Example")
        .size(800.0, 600.0)
        .with_layers(move |layers| {
            // Layer 0: Animated starfield background
            layers.add_raw_layer(0, LayerOptions::default().with_clear(), |ctx| {
                ctx.request_animation_frame();

                let shader = r#"
                    float4 shader_main(float2 uv, float2 resolution, float time) {
                        float3 bg_color = float3(0.02, 0.03, 0.08);
                        float2 center = float2(0.5, 0.5);
                        float2 centered_uv = uv - center;
                        float rotation_speed = 0.002;
                        float angle = time * rotation_speed;
                        float cos_angle = cos(angle);
                        float sin_angle = sin(angle);
                        float2 rotated_uv = float2(
                            centered_uv.x * cos_angle - centered_uv.y * sin_angle,
                            centered_uv.x * sin_angle + centered_uv.y * cos_angle
                        ) + center;
                        float2 grid_uv = rotated_uv * 220.0;
                        float2 grid_id = floor(grid_uv);
                        float2 grid_pos = fract(grid_uv);
                        float random = fract(sin(dot(grid_id, float2(12.9898, 78.233))) * 43758.5453);
                        float star_threshold = 0.95;
                        float star_exists = step(star_threshold, random);
                        float2 star_pos = float2(
                            fract(random * 17.0),
                            fract(random * 31.0)
                        );
                        float star_dist = length(grid_pos - star_pos);
                        float twinkle_speed = 0.5 + random * 0.8;
                        float stagger_offset = random * 100.0;
                        float twinkle = 0.3 + 0.7 * (0.5 + 0.5 * sin(time * twinkle_speed + stagger_offset));
                        float star_size = 0.02 + random * 0.03;
                        float star_brightness = star_exists * twinkle * smoothstep(star_size, 0.0, star_dist);
                        float3 star_color = float3(0.8 + random * 0.2, 0.9 + random * 0.1, 1.0);
                        float3 final_color = bg_color + star_color * star_brightness;
                        return float4(final_color, 1.0);
                    }
                "#;

                ctx.draw_fullscreen_quad(shader);
            });

            // Layer 1: Interactive UI using Button elements
            let counter_clone = counter.clone();
            layers.add_ui_layer(1, LayerOptions::default().with_input(), move || {
                let count = *counter_clone.borrow();
                let inc = counter_clone.clone();
                let dec = counter_clone.clone();
                let reset = counter_clone.clone();

                Box::new(
                    container()
                        .width_full()
                        .height_full()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .gap(20.0)
                        .child(text(
                            "Sol UI Demo",
                            TextStyle {
                                color: colors::WHITE,
                                size: 24.0,
                                ..Default::default()
                            },
                        ))
                        .child(text(
                            format!("Count: {}", count),
                            TextStyle {
                                color: colors::GRAY_200,
                                size: 20.0,
                                ..Default::default()
                            },
                        ))
                        .child(
                            row()
                                .gap(10.0)
                                .child(
                                    button("+")
                                        .with_id(1)
                                        .padding(16.0)
                                        .on_click_simple(move || {
                                            *inc.borrow_mut() += 1;
                                        })
                                )
                                .child(
                                    button("Reset")
                                        .with_id(2)
                                        .background(colors::GRAY_700)
                                        .hover_background(colors::GRAY_600)
                                        .press_background(colors::GRAY_800)
                                        .padding(16.0)
                                        .on_click_simple(move || {
                                            *reset.borrow_mut() = 0;
                                        })
                                )
                                .child(
                                    button("-")
                                        .with_id(3)
                                        .background(colors::RED_500)
                                        .hover_background(colors::RED_400)
                                        .press_background(colors::RED_600)
                                        .padding(16.0)
                                        .on_click_simple(move || {
                                            *dec.borrow_mut() -= 1;
                                        })
                                )
                        )
                )
            });
        })
        .run();
}
