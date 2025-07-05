# Toy UI

A lightweight UI framework for Rust, built on Metal for macOS. This is an experimental project exploring immediate-mode UI concepts with native performance.

## Features

- Immediate-mode UI API
- Hardware-accelerated rendering via Metal
- Text rendering with Core Text
- Basic UI elements (text, rectangles, groups, windows)
- Layout helpers (horizontal/vertical stacking, spacing)
- Color support via the `palette` crate

## Usage

### Adding as a Dependency

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

## API Overview

### App Builder

Create and configure your application window:

```rust
app()
    .size(width, height)
    .title("Window Title")
    .layer(render_fn)
    .run()
```

### UiContext

The main UI building interface, passed to your render function:

- `text(string)` - Render text
- `rect(size, color)` - Draw a colored rectangle
- `space(pixels)` - Add vertical spacing
- `horizontal(fn)` - Layout children horizontally
- `vertical(fn)` - Layout children vertically
- `group(fn)` / `group_styled(color, fn)` - Group elements with optional background
- `window(title, position, size, fn)` - Create a movable window

## Requirements

- macOS (Metal rendering backend)
- Rust 2024 edition

## Project Structure

- `src/app.rs` - Application setup and main loop
- `src/platform/` - Platform-specific code (macOS window management)
- `src/renderer/` - Metal rendering implementation
- `src/ui/` - UI context and drawing commands
- `src/layer/` - Layer management for rendering
- `src/text/` - Text rendering system

## Example Project

See the `toy-ui-example` directory for a complete example of using toy-ui as a library dependency.

## Status

This is an experimental project and the API is subject to change. Not recommended for production use.

## License

[Add your license here]
