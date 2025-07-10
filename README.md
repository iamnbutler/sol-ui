# Toy UI

A lightweight immediate mode gui library for Rust, built on Metal for macOS.

![](https://github.com/user-attachments/assets/79b966d3-94b0-4a12-9121-a4980cf8a960)

## Usage

There isn't a published crate yet, for now you can try it out by cloning the repository and adding the library to your project and referencing it locally:

```toml
[dependencies]
toy-ui = { path = "../toy-ui" }
```

### Basic Example

```rust
use std::cell::RefCell;
use std::rc::Rc;
use toy_ui::{
    app,
    color::colors,
    draw::TextStyle,
    elements::{container, row, text},
    interaction::Interactable,
    layer::{LayerOptions, MouseButton},
};

fn main() {
    // Shared state
    let counter = Rc::new(RefCell::new(0));

    app()
        .title("Toy UI Example")
        .size(800.0, 600.0)
        .with_layers(move |layers| {
            // Layer 0: Animated starfield
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

            // Layer 1: Interactive UI
            let counter_clone = counter.clone();
            layers.add_ui_layer(1, LayerOptions::default().with_input(), move || {
                let count = *counter_clone.borrow();

                Box::new(
                    container()
                        .width_full()
                        .height_full()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .gap(20.0)
                        .child(text(
                            "Toy UI Demo",
                            TextStyle {
                                color: colors::WHITE,
                                size: 24.0,
                            },
                        ))
                        .child(text(
                            format!("Count: {}", count),
                            TextStyle {
                                color: colors::GRAY_200,
                                size: 20.0,
                            },
                        ))
                        .child(
                            row()
                                .gap(10.0)
                                .child(
                                    container()
                                        .width(40.0)
                                        .height(40.0)
                                        .border(colors::GRAY_900, 1.0)
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(text(
                                            "+",
                                            TextStyle {
                                                color: colors::WHITE,
                                                size: 12.0,
                                            },
                                        ))
                                        .interactive()
                                        .with_id(1)
                                        .on_click({
                                            let counter = counter_clone.clone();
                                            move |button, _, _| {
                                                if button == MouseButton::Left {
                                                    *counter.borrow_mut() += 1;
                                                }
                                            }
                                        }),
                                )
                                .child(
                                    container()
                                        .width(160.0)
                                        .height(40.0)
                                        .border(colors::GRAY_900, 1.0)
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(text(
                                            "Reset",
                                            TextStyle {
                                                color: colors::WHITE,
                                                size: 12.0,
                                            },
                                        ))
                                        .interactive()
                                        .with_id(2)
                                        .on_click({
                                            let counter = counter_clone.clone();
                                            move |button, _, _| {
                                                if button == MouseButton::Left {
                                                    *counter.borrow_mut() = 0;
                                                }
                                            }
                                        }),
                                )
                            .child(
                                container()
                                    .width(40.0)
                                    .height(40.0)
                                    .border(colors::GRAY_900, 1.0)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(text(
                                        "-",
                                        TextStyle {
                                            color: colors::WHITE,
                                            size: 12.0,
                                        },
                                    ))
                                    .interactive()
                                    .with_id(3)
                                    .on_click({
                                        let counter = counter_clone.clone();
                                        move |button, _, _| {
                                            if button == MouseButton::Left {
                                                *counter.borrow_mut() -= 1;
                                            }
                                        }
                                    }),
                            )
                        ),
                )
            });
        })
        .run();
}
```

## Features

- **Immediate mode UI** with declarative syntax
- **Hardware accelerated** rendering using Metal
- **Multi-layer system** for complex compositions
- **Interactive elements** with hover and click states
- **Raw shader layers** for custom GPU effects
- **Built-in layout system** using Taffy (Flexbox)
- **Z-order based hit testing** for proper event handling

Lots of inspiration from [GPUI](https://gpui.rs).
