# Toy UI

A lightweight UI framework for Rust, built on Metal for macOS. This is an experimental project exploring immediate-mode UI concepts with native performance.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
toy-ui = { path = "../toy-ui" }  # Or use git URL when published
glam = "0.30.4"      # For vector math
palette = "0.7.6"    # For colors
```

### Basic Example

```rust
use toy_ui::{app, ui::UiContext};
use palette::Srgba;

fn main() {
    app()
        .size(800.0, 600.0)
        .title("My App")
        .layer(|ui: &mut UiContext| {
            // Your UI code here
            ui.text("Hello, World!");

            ui.space(20.0);

            // Draw colored rectangles
            ui.horizontal(|ui| {
                ui.rect(
                    glam::vec2(50.0, 50.0),
                    Srgba::new(1.0, 0.0, 0.0, 1.0), // Red
                );
                ui.rect(
                    glam::vec2(50.0, 50.0),
                    Srgba::new(0.0, 1.0, 0.0, 1.0), // Green
                );
            });
        })
        .run();
}
```

Lots of inspiration from [GPUI](https://gpui.rs).
