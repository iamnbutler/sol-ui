//! Box model example demonstrating border, padding, and content sizing
//!
//! This example creates a visual test of the CSS box model implementation,
//! ensuring that borders, padding, and content are rendered correctly.

use sol_ui::{
    app::app,
    color::{Color, ColorExt},
    element::{Container, Text},
    geometry::{Rect, WorldPoint, LocalPoint, local_to_world, world_to_screen, screen_to_world},
    layer::{LayerManager, LayerOptions},
    style::TextStyle,
};


/// Expected dimensions for our test element
const CONTENT_WIDTH: f32 = 300.0;
const CONTENT_HEIGHT: f32 = 200.0;
const BORDER_WIDTH: f32 = 5.0;
const PADDING: f32 = 20.0;

/// Calculate expected total dimensions
const EXPECTED_INNER_WIDTH: f32 = CONTENT_WIDTH + (PADDING * 2.0);
const EXPECTED_INNER_HEIGHT: f32 = CONTENT_HEIGHT + (PADDING * 2.0);
const EXPECTED_OUTER_WIDTH: f32 = EXPECTED_INNER_WIDTH + (BORDER_WIDTH * 2.0);
const EXPECTED_OUTER_HEIGHT: f32 = EXPECTED_INNER_HEIGHT + (BORDER_WIDTH * 2.0);

fn main() {
    println!("Box Model Test");
    println!("==============");
    println!("Expected dimensions:");
    println!("  Content: {}x{}", CONTENT_WIDTH, CONTENT_HEIGHT);
    println!("  With padding: {}x{}", EXPECTED_INNER_WIDTH, EXPECTED_INNER_HEIGHT);
    println!("  With border: {}x{}", EXPECTED_OUTER_WIDTH, EXPECTED_OUTER_HEIGHT);
    println!();

    // Run tests
    test_box_model_math();
    test_coordinate_conversions();
    test_border_positioning();
    println!("\nAll tests passed! ✓");

    // Create the visual example
    app()
        .title("Box Model Test")
        .size(800.0, 600.0)
        .with_layers(|layer_manager: &mut LayerManager| {
            // Add a UI layer with our test elements
            layer_manager.add_ui_layer(
                0,
                LayerOptions::default()
                    .with_clear()
                    .with_clear_color(0.95, 0.95, 0.95, 1.0),
                || Box::new(build_test_ui()),
            );

            // Add overlay with measurements
            layer_manager.add_ui_layer(
                1,
                LayerOptions::default(),
                || Box::new(build_overlay()),
            );
        })
        .run();
}

fn build_test_ui() -> Container {
    // Create a centered container
    Container::new()
        .flex_col()
        .justify_center()
        .items_center()
        .width_full()
        .height_full()
        .child(
            // Create the test box with specific dimensions
            Container::new()
                .width(EXPECTED_OUTER_WIDTH)
                .height(EXPECTED_OUTER_HEIGHT)
                .background(Color::rgb(0.2, 0.4, 0.8))
                .border(Color::rgb(0.8, 0.2, 0.2), BORDER_WIDTH)
                .padding(PADDING)
                .child(
                    // Add content area (should be CONTENT_WIDTH x CONTENT_HEIGHT)
                    Container::new()
                        .width_full()
                        .height_full()
                        .background(Color::rgba(1.0, 1.0, 1.0, 0.5))
                        .flex_col()
                        .justify_center()
                        .items_center()
                        .gap(10.0)
                        .child(
                            Text::new("Content Area", TextStyle {
                                size: 18.0,
                                color: Color::rgb(0.0, 0.0, 0.0),
                                ..Default::default()
                            })
                        )
                        .child(
                            Text::new(
                                format!("{}x{}", CONTENT_WIDTH, CONTENT_HEIGHT),
                                TextStyle {
                                    size: 14.0,
                                    color: Color::rgb(0.3, 0.3, 0.3),
                                    ..Default::default()
                                }
                            )
                        )
                )
        )
}

fn build_overlay() -> Container {
    Container::new()
        .padding(15.0)
        .background(Color::rgba(0.0, 0.0, 0.0, 0.85))
        .corner_radius(8.0)
        .gap(5.0)
        .flex_col()
        .child(
            Text::new("Box Model Test", TextStyle {
                size: 16.0,
                color: Color::rgb(1.0, 1.0, 1.0),
                ..Default::default()
            })
        )
        .child(
            Text::new(
                format!("Content: {}x{}", CONTENT_WIDTH, CONTENT_HEIGHT),
                TextStyle {
                    size: 12.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                    ..Default::default()
                }
            )
        )
        .child(
            Text::new(
                format!("Padding: {} (all sides)", PADDING),
                TextStyle {
                    size: 12.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                    ..Default::default()
                }
            )
        )
        .child(
            Text::new(
                format!("Border: {} (all sides)", BORDER_WIDTH),
                TextStyle {
                    size: 12.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                    ..Default::default()
                }
            )
        )
        .child(
            Text::new(
                format!("Inner: {}x{}", EXPECTED_INNER_WIDTH, EXPECTED_INNER_HEIGHT),
                TextStyle {
                    size: 12.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                    ..Default::default()
                }
            )
        )
        .child(
            Text::new(
                format!("Total: {}x{}", EXPECTED_OUTER_WIDTH, EXPECTED_OUTER_HEIGHT),
                TextStyle {
                    size: 12.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                    ..Default::default()
                }
            )
        )
}

fn test_box_model_math() {
    println!("Testing box model calculations...");

    assert_eq!(EXPECTED_INNER_WIDTH, 340.0, "Inner width calculation");
    assert_eq!(EXPECTED_INNER_HEIGHT, 240.0, "Inner height calculation");
    assert_eq!(EXPECTED_OUTER_WIDTH, 350.0, "Outer width calculation");
    assert_eq!(EXPECTED_OUTER_HEIGHT, 250.0, "Outer height calculation");

    println!("  ✓ Box model math correct");
}

fn test_coordinate_conversions() {
    println!("Testing coordinate space conversions...");

    // Test local to world conversion
    let local = LocalPoint::new(10.0, 20.0);
    let parent = WorldPoint::new(100.0, 100.0);
    let world = local_to_world(local, parent);

    assert_eq!(world.x(), 110.0, "World X coordinate");
    assert_eq!(world.y(), 120.0, "World Y coordinate");
    println!("  ✓ Local to world conversion");

    // Test world to screen (currently identity)
    let screen = world_to_screen(world);
    assert_eq!(screen.x(), 110.0, "Screen X coordinate");
    assert_eq!(screen.y(), 120.0, "Screen Y coordinate");
    println!("  ✓ World to screen conversion");

    // Test round trip
    let back = screen_to_world(screen);
    assert_eq!(back.x(), world.x(), "Round trip X");
    assert_eq!(back.y(), world.y(), "Round trip Y");
    println!("  ✓ Round trip conversion");
}

fn test_border_positioning() {
    println!("Testing border positioning...");

    // Create a test rectangle
    let bounds = Rect::new(100.0, 100.0, EXPECTED_OUTER_WIDTH, EXPECTED_OUTER_HEIGHT);

    // Borders should be inside the outer bounds
    // Top border starts at top edge
    assert_eq!(bounds.pos.y(), 100.0, "Top border Y position");

    // Left border starts at left edge
    assert_eq!(bounds.pos.x(), 100.0, "Left border X position");

    // Content area should be inset by border + padding
    let content_x = bounds.pos.x() + BORDER_WIDTH + PADDING;
    let content_y = bounds.pos.y() + BORDER_WIDTH + PADDING;
    assert_eq!(content_x, 125.0, "Content X position");
    assert_eq!(content_y, 125.0, "Content Y position");

    // Content dimensions should match original
    let content_width = bounds.size.width() - (BORDER_WIDTH * 2.0) - (PADDING * 2.0);
    let content_height = bounds.size.height() - (BORDER_WIDTH * 2.0) - (PADDING * 2.0);
    assert_eq!(content_width, CONTENT_WIDTH, "Content width");
    assert_eq!(content_height, CONTENT_HEIGHT, "Content height");

    println!("  ✓ Border positioning correct");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimensions() {
        assert_eq!(EXPECTED_INNER_WIDTH, 340.0);
        assert_eq!(EXPECTED_INNER_HEIGHT, 240.0);
        assert_eq!(EXPECTED_OUTER_WIDTH, 350.0);
        assert_eq!(EXPECTED_OUTER_HEIGHT, 250.0);
    }

    #[test]
    fn test_coordinate_types() {
        let local = LocalPoint::new(10.0, 10.0);
        let world = WorldPoint::new(10.0, 10.0);

        // Z should default to 1.0 for 2D points
        assert_eq!(local.z(), 1.0);
        assert_eq!(world.z(), 1.0);
    }

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(100.0, 100.0, 350.0, 250.0);

        // Test points inside
        assert!(rect.contains(WorldPoint::new(200.0, 200.0)));
        assert!(rect.contains(WorldPoint::new(100.0, 100.0)));
        assert!(rect.contains(WorldPoint::new(450.0, 350.0)));

        // Test points outside
        assert!(!rect.contains(WorldPoint::new(99.0, 200.0)));
        assert!(!rect.contains(WorldPoint::new(451.0, 200.0)));
    }
}
