//! Viewport culling demonstration
//!
//! This example shows how the rendering system automatically culls elements
//! that are outside the viewport, preventing unnecessary draw calls.

use palette::Srgba;
use toy_ui::{
    app,
    draw::TextStyle,
    layer::{LayerManager, LayerOptions},
    layout::{col, group, row, text},
};

fn main() {
    app()
        .title("Viewport Culling Demo")
        .size(800.0, 600.0)
        .with_layers(|layer_manager: &mut LayerManager| {
            // Main layer with a scrollable grid
            layer_manager.add_ui_layer(0, LayerOptions::default(), || {
                // LargeUse a reasonable grid size that won't cause 6-second resize times
                let grid_size = 15;
                let cell_size = 60.0;
                let padding = 5.0;

                let mut main_container = col().gap(10.0).p(10.0);

                // Header with information
                main_container = main_container.child(
                    group().bg(Srgba::new(0.1, 0.1, 0.1, 0.9)).p(20.0).child(
                        col()
                            .gap(10.0)
                            .child(text(
                                "Viewport Culling Demo",
                                TextStyle {
                                    size: 24.0,
                                    color: Srgba::new(1.0, 1.0, 1.0, 1.0),
                                },
                            ))
                            .child(text(
                                format!(
                                    "Grid: {}x{} = {} cells ({:.0}x{:.0} pixels)",
                                    grid_size,
                                    grid_size,
                                    grid_size * grid_size,
                                    grid_size as f32 * (cell_size + padding),
                                    grid_size as f32 * (cell_size + padding)
                                ),
                                TextStyle {
                                    size: 16.0,
                                    color: Srgba::new(0.8, 0.8, 0.8, 1.0),
                                },
                            ))
                            .child(text(
                                "The renderer automatically culls off-screen elements",
                                TextStyle {
                                    size: 14.0,
                                    color: Srgba::new(0.7, 0.7, 0.7, 1.0),
                                },
                            ))
                            .child(text(
                                "Try resizing the window to see different cells",
                                TextStyle {
                                    size: 14.0,
                                    color: Srgba::new(1.0, 1.0, 0.0, 1.0),
                                },
                            )),
                    ),
                );

                // Create the grid
                let mut grid_container = col().gap(padding);

                for row_idx in 0..grid_size {
                    let mut row_container = row().gap(padding);

                    for col_idx in 0..grid_size {
                        // Create a color gradient across the grid
                        let r = (col_idx as f32 / (grid_size - 1) as f32).min(1.0);
                        let g = (row_idx as f32 / (grid_size - 1) as f32).min(1.0);
                        let b = 0.5 + 0.5 * ((row_idx + col_idx) as f32 / (2.0 * grid_size as f32));
                        let color = Srgba::new(r, g, b, 1.0);

                        // Calculate text color for contrast
                        let brightness = r * 0.299 + g * 0.587 + b * 0.114;
                        let text_color = if brightness > 0.5 {
                            Srgba::new(0.0, 0.0, 0.0, 1.0)
                        } else {
                            Srgba::new(1.0, 1.0, 1.0, 1.0)
                        };

                        // Create cell
                        let cell = group()
                            .size(cell_size, cell_size)
                            .bg(color)
                            .justify_center()
                            .items_center()
                            .child(text(
                                format!("{},{}", row_idx, col_idx),
                                TextStyle {
                                    size: 14.0,
                                    color: text_color,
                                },
                            ));

                        row_container = row_container.child(cell);
                    }

                    grid_container = grid_container.child(row_container);
                }

                // Wrap grid in a scrollable area (once we have scroll support)
                // For now, just add it directly
                main_container = main_container.child(
                    group()
                        .bg(Srgba::new(0.05, 0.05, 0.05, 1.0))
                        .p(10.0)
                        .child(grid_container),
                );

                Box::new(main_container)
            });

            // Performance info layer
            layer_manager.add_ui_layer(1, LayerOptions::default(), || {
                Box::new(
                    col()
                        .gap(10.0)
                        .child(
                            group()
                                .bg(Srgba::new(0.0, 0.3, 0.0, 0.9))
                                .p(15.0)
                                .m(10.0)
                                .child(
                                    col()
                                        .gap(8.0)
                                        .child(text(
                                            "Performance Features:",
                                            TextStyle {
                                                size: 18.0,
                                                color: Srgba::new(1.0, 1.0, 1.0, 1.0),
                                            },
                                        ))
                                        .child(text(
                                            "✓ Draw lists are cached between frames",
                                            TextStyle {
                                                size: 14.0,
                                                color: Srgba::new(0.9, 1.0, 0.9, 1.0),
                                            },
                                        ))
                                        .child(text(
                                            "✓ Only visible elements are sent to Metal",
                                            TextStyle {
                                                size: 14.0,
                                                color: Srgba::new(0.9, 1.0, 0.9, 1.0),
                                            },
                                        ))
                                        .child(text(
                                            "✓ Layout is only recomputed on window resize",
                                            TextStyle {
                                                size: 14.0,
                                                color: Srgba::new(0.9, 1.0, 0.9, 1.0),
                                            },
                                        ))
                                        .child(text(
                                            "✓ Text measurements are cached within frames",
                                            TextStyle {
                                                size: 14.0,
                                                color: Srgba::new(0.9, 1.0, 0.9, 1.0),
                                            },
                                        )),
                                ),
                        )
                        .child(
                            group()
                                .bg(Srgba::new(0.3, 0.0, 0.0, 0.9))
                                .p(15.0)
                                .m(10.0)
                                .child(
                                    col()
                                        .gap(8.0)
                                        .child(text(
                                            "Current Limitations:",
                                            TextStyle {
                                                size: 18.0,
                                                color: Srgba::new(1.0, 1.0, 1.0, 1.0),
                                            },
                                        ))
                                        .child(text(
                                            "✗ All UI elements are built even if off-screen",
                                            TextStyle {
                                                size: 14.0,
                                                color: Srgba::new(1.0, 0.9, 0.9, 1.0),
                                            },
                                        ))
                                        .child(text(
                                            "✗ No virtualized scrolling for large lists",
                                            TextStyle {
                                                size: 14.0,
                                                color: Srgba::new(1.0, 0.9, 0.9, 1.0),
                                            },
                                        ))
                                        .child(text(
                                            "✗ Full UI tree rebuild on window resize",
                                            TextStyle {
                                                size: 14.0,
                                                color: Srgba::new(1.0, 0.9, 0.9, 1.0),
                                            },
                                        )),
                                ),
                        ),
                )
            });
        })
        .run();
}
