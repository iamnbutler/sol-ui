# Hello World

A lightweight immediate mode gui library for Rust, built on Metal for macOS.

`cargo add sol-ui`

### Basic Example

```rust
use sol_ui::{app, color::colors, draw::TextStyle, elements::{container, text}};

fn main() {
    app()
        .title("Hello World")
        .size(800.0, 600.0)
        .with_layers(move |layers| {
            layers.add_ui_layer(0, Default::default(), move || {
                Box::new(
                    container()
                        .width_full()
                        .height_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(text(
                            "Hello, World!",
                            TextStyle {
                                color: colors::WHITE,
                                size: 24.0,
                            },
                        ))
                )
            });
        })
        .run();
}
```

Lots of inspiration from [GPUI](https://gpui.rs).
