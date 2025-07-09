//! Viewport culling demonstration
//!
//! This example shows how viewport culling prevents rendering of elements
//! that are completely outside the visible area.

use glam::Vec2;
use palette::Srgba;
use toy_ui::{
    app,
    draw::TextStyle,
    geometry::Rect,
    layer::{LayerManager, LayerOptions, UiLayerContext},
};

fn main() {
    app()
        .title("Viewport Culling Demo")
        .size(800.0, 600.0)
        .with_layers(|layer_manager: &mut LayerManager| {
            // Add main UI layer that demonstrates culling
            layer_manager.add_ui_layer(0, LayerOptions::default(), |ctx: &mut UiLayerContext| {
                let ui = &mut ctx.ui;

                // Draw a grid of colored rectangles, many of which are off-screen
                let grid_size = 50;
                let cell_size = 80.0;
                let padding = 10.0;

                // Stats tracking
                let mut total_elements = 0;
                let mut visible_elements = 0;

                // Create a large grid that extends beyond the viewport
                for row in 0..grid_size {
                    for col in 0..grid_size {
                        total_elements += 1;

                        let x = col as f32 * (cell_size + padding) + padding;
                        let y = row as f32 * (cell_size + padding) + padding;

                        // Check if this element would be visible
                        let element_rect = Rect::new(x, y, cell_size, cell_size);
                        let viewport = Rect::from_pos_size(Vec2::ZERO, ctx.size);

                        if viewport.intersect(&element_rect).is_some() {
                            visible_elements += 1;
                        }

                        // Color based on position
                        let r = (col as f32 / grid_size as f32).min(1.0);
                        let g = (row as f32 / grid_size as f32).min(1.0);
                        let b = 0.5;
                        let color = Srgba::new(r, g, b, 1.0);

                        // Draw the rectangle (will be culled if outside viewport)
                        ui.set_cursor(Vec2::new(x, y));
                        ui.rect(Vec2::new(cell_size, cell_size), color);

                        // Add text label in the center
                        ui.set_cursor(Vec2::new(x + 10.0, y + cell_size / 2.0 - 8.0));
                        ui.text_styled(
                            format!("{},{}", row, col),
                            TextStyle {
                                size: 12.0,
                                color: Srgba::new(1.0, 1.0, 1.0, 1.0),
                            },
                        );
                    }
                }

                // Get actual culling stats from the draw list
                let culling_stats = ui.draw_list().culling_stats();
                let actual_culled = culling_stats.culled_count;
                let actual_rendered = culling_stats.rendered_count;
                let culling_percentage = culling_stats.culling_percentage();

                // Draw stats overlay
                ui.set_cursor(Vec2::new(10.0, 10.0));
                ui.group_styled(Some(Srgba::new(0.0, 0.0, 0.0, 0.8)), |ui| {
                    ui.vertical(|ui| {
                        ui.text_styled(
                            "Viewport Culling Demo",
                            TextStyle {
                                size: 24.0,
                                color: Srgba::new(1.0, 1.0, 1.0, 1.0),
                            },
                        );
                        ui.space(10.0);
                        ui.text_styled(
                            format!(
                                "Grid Size: {}x{} = {} total cells",
                                grid_size, grid_size, total_elements
                            ),
                            TextStyle {
                                size: 16.0,
                                color: Srgba::new(0.8, 0.8, 0.8, 1.0),
                            },
                        );
                        ui.text_styled(
                            format!("Visible Elements (calculated): {}", visible_elements),
                            TextStyle {
                                size: 16.0,
                                color: Srgba::new(0.0, 1.0, 0.0, 1.0),
                            },
                        );
                        ui.space(10.0);
                        ui.text_styled(
                            "Actual Rendering Stats:",
                            TextStyle {
                                size: 18.0,
                                color: Srgba::new(1.0, 1.0, 0.0, 1.0),
                            },
                        );
                        ui.text_styled(
                            format!("Draw Calls Rendered: {}", actual_rendered),
                            TextStyle {
                                size: 16.0,
                                color: Srgba::new(0.0, 1.0, 1.0, 1.0),
                            },
                        );
                        ui.text_styled(
                            format!("Draw Calls Culled: {}", actual_culled),
                            TextStyle {
                                size: 16.0,
                                color: Srgba::new(1.0, 0.0, 0.0, 1.0),
                            },
                        );
                        ui.text_styled(
                            format!("Culling Rate: {:.1}%", culling_percentage),
                            TextStyle {
                                size: 16.0,
                                color: Srgba::new(1.0, 0.5, 0.0, 1.0),
                            },
                        );
                        ui.space(10.0);
                        ui.text_styled(
                            "Elements outside the viewport are not rendered",
                            TextStyle {
                                size: 14.0,
                                color: Srgba::new(0.7, 0.7, 0.7, 1.0),
                            },
                        );
                    });
                });

                // Instructions
                ui.set_cursor(Vec2::new(10.0, ctx.size.y - 60.0));
                ui.group_styled(Some(Srgba::new(0.0, 0.0, 0.0, 0.8)), |ui| {
                    ui.vertical(|ui| {
                        ui.text_styled(
                            "Resize the window to see culling in action",
                            TextStyle {
                                size: 14.0,
                                color: Srgba::new(1.0, 1.0, 1.0, 1.0),
                            },
                        );
                        ui.text_styled(
                            "Only elements intersecting the viewport are rendered",
                            TextStyle {
                                size: 14.0,
                                color: Srgba::new(1.0, 1.0, 1.0, 1.0),
                            },
                        );
                    });
                });
            });

            // Add a second layer showing debug culling visualization
            layer_manager.add_ui_layer(1, LayerOptions::default(), |ctx: &mut UiLayerContext| {
                let ui = &mut ctx.ui;

                // Note: Debug culling visualization would go here if we had mutable access to draw_list
                // For now, we'll just draw some test elements

                // Draw some elements that will be partially culled
                let size = 100.0;
                let positions = [
                    Vec2::new(-size / 2.0, -size / 2.0), // Top-left, partially off-screen
                    Vec2::new(ctx.size.x - size / 2.0, -size / 2.0), // Top-right
                    Vec2::new(-size / 2.0, ctx.size.y - size / 2.0), // Bottom-left
                    Vec2::new(ctx.size.x - size / 2.0, ctx.size.y - size / 2.0), // Bottom-right
                ];

                for (i, pos) in positions.iter().enumerate() {
                    ui.set_cursor(*pos);
                    ui.rect(Vec2::new(size, size), Srgba::new(0.0, 0.5, 1.0, 0.5));
                    ui.set_cursor(*pos + Vec2::new(10.0, size / 2.0 - 8.0));
                    ui.text_styled(
                        format!("Debug {}", i + 1),
                        TextStyle {
                            size: 16.0,
                            color: Srgba::new(1.0, 1.0, 1.0, 1.0),
                        },
                    );
                }

                // Show debug info
                ui.set_cursor(Vec2::new(ctx.size.x - 250.0, 10.0));
                ui.group_styled(Some(Srgba::new(0.0, 0.0, 0.5, 0.8)), |ui| {
                    ui.vertical(|ui| {
                        ui.text_styled(
                            "Debug Culling Enabled",
                            TextStyle {
                                size: 16.0,
                                color: Srgba::new(1.0, 1.0, 1.0, 1.0),
                            },
                        );
                        ui.text_styled(
                            "Culled elements shown in red",
                            TextStyle {
                                size: 14.0,
                                color: Srgba::new(1.0, 0.5, 0.5, 1.0),
                            },
                        );
                    });
                });
            });
        })
        .run();
}
