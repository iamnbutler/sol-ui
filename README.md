# Toy UI

An exploration in building an immediate mode UI framework from scratch on macOS. ~~No~~ A few shortcuts - just raw Objective-C FFI and Metal rendering.

Built with a 3D-first mindset using `glam` for math, direct Metal APIs for GPU rendering, and Core Text + font-kit for proper text shaping. Every frame redraws the entire UI from simple function calls like `ui.rect()` and `ui.text()`.

Next up: Multi-layer rendering with mixed 2D/3D content, Taffy flexbox integration, and a type-safe coordinate system that prevents mixing screen/world space at compile time.
