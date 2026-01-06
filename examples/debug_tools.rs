//! Debug tools example demonstrating the debug overlay system
//!
//! Keyboard shortcuts:
//! - F12: Toggle debug overlay
//! - F1: Toggle bounds overlay
//! - F2: Toggle layout inspector
//! - F3: Toggle hit test visualization
//! - F4: Toggle performance metrics
//! - F6: Toggle debug console

use sol_ui::{
    app::app,
    color::{ColorExt, colors},
    debug::{DebugOverlay, DebugPanel},
    element::{column, container, row, text},
    interaction::Interactable,
    layer::{LayerOptions, MouseButton},
    style::TextStyle,
};
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    // Shared debug overlay state
    let debug = Rc::new(RefCell::new(DebugOverlay::new()));

    // Enable debug mode by default for this example
    debug.borrow_mut().state_mut().enable();
    debug.borrow_mut().state_mut().enable_panel(DebugPanel::Bounds);

    // Counter for the demo UI
    let counter = Rc::new(RefCell::new(0));

    app()
        .title("Debug Tools Example")
        .size(900.0, 700.0)
        .with_layers(move |layers| {
            let debug_clone = debug.clone();
            let counter_clone = counter.clone();

            // Main UI layer
            layers.add_ui_layer(
                0,
                LayerOptions::default().with_input().with_clear(),
                move || {
                    let count = *counter_clone.borrow();
                    let counter_clone2 = counter_clone.clone();
                    let debug_ref = debug_clone.borrow();

                    // Register some element bounds for the debug overlay
                    // In a real app, elements would self-register during paint

                    Box::new(
                        container()
                            .width_full()
                            .height_full()
                            .background(colors::GRAY_100)
                            .flex_col()
                            .padding(20.0)
                            .gap(16.0)
                            .child(
                                // Header
                                container()
                                    .width_full()
                                    .background(colors::BLUE_500)
                                    .corner_radius(8.0)
                                    .padding(16.0)
                                    .child(
                                        column()
                                            .gap(8.0)
                                            .child(
                                                text(
                                                    "Debug Tools Demo",
                                                    TextStyle {
                                                        color: colors::WHITE,
                                                        size: 24.0,
                                                    },
                                                )
                                            )
                                            .child(
                                                text(
                                                    "Press F12 to toggle debug overlay",
                                                    TextStyle {
                                                        color: colors::WHITE.with_alpha(0.8),
                                                        size: 14.0,
                                                    },
                                                )
                                            )
                                    )
                            )
                            .child(
                                // Help panel
                                container()
                                    .background(colors::WHITE)
                                    .corner_radius(8.0)
                                    .padding(16.0)
                                    .border(colors::GRAY_300, 1.0)
                                    .child(
                                        column()
                                            .gap(8.0)
                                            .child(
                                                text(
                                                    "Keyboard Shortcuts",
                                                    TextStyle {
                                                        color: colors::GRAY_800,
                                                        size: 18.0,
                                                    },
                                                )
                                            )
                                            .child(text_line("F12 - Toggle debug overlay"))
                                            .child(text_line("F1 - Toggle bounds overlay"))
                                            .child(text_line("F2 - Toggle layout inspector"))
                                            .child(text_line("F3 - Toggle hit test visualization"))
                                            .child(text_line("F4 - Toggle performance metrics"))
                                            .child(text_line("F6 - Toggle debug console"))
                                    )
                            )
                            .child(
                                // Counter demo
                                row()
                                    .gap(16.0)
                                    .child(
                                        // Counter display
                                        container()
                                            .width(200.0)
                                            .height(100.0)
                                            .background(colors::GREEN_400)
                                            .corner_radius(12.0)
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(
                                                text(
                                                    format!("Count: {}", count),
                                                    TextStyle {
                                                        color: colors::WHITE,
                                                        size: 28.0,
                                                    },
                                                )
                                            )
                                    )
                                    .child(
                                        column()
                                            .gap(8.0)
                                            .child(
                                                // Increment button
                                                container()
                                                    .width(120.0)
                                                    .height(44.0)
                                                    .background(colors::BLUE_500)
                                                    .corner_radius(8.0)
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .child(
                                                        text(
                                                            "+1",
                                                            TextStyle {
                                                                color: colors::WHITE,
                                                                size: 18.0,
                                                            },
                                                        )
                                                    )
                                                    .interactive()
                                                    .with_id(1)
                                                    .hover_overlay(colors::BLACK.with_alpha(0.1))
                                                    .press_overlay(colors::BLACK.with_alpha(0.2))
                                                    .on_click({
                                                        let counter = counter_clone2.clone();
                                                        move |button, _, _| {
                                                            if button == MouseButton::Left {
                                                                *counter.borrow_mut() += 1;
                                                            }
                                                        }
                                                    })
                                            )
                                            .child(
                                                // Decrement button
                                                container()
                                                    .width(120.0)
                                                    .height(44.0)
                                                    .background(colors::RED_500)
                                                    .corner_radius(8.0)
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .child(
                                                        text(
                                                            "-1",
                                                            TextStyle {
                                                                color: colors::WHITE,
                                                                size: 18.0,
                                                            },
                                                        )
                                                    )
                                                    .interactive()
                                                    .with_id(2)
                                                    .hover_overlay(colors::BLACK.with_alpha(0.1))
                                                    .press_overlay(colors::BLACK.with_alpha(0.2))
                                                    .on_click({
                                                        let counter = counter_clone2.clone();
                                                        move |button, _, _| {
                                                            if button == MouseButton::Left {
                                                                *counter.borrow_mut() -= 1;
                                                            }
                                                        }
                                                    })
                                            )
                                    )
                            )
                            .child(
                                // Nested layout demo
                                container()
                                    .background(colors::PURPLE_400)
                                    .corner_radius(12.0)
                                    .padding(16.0)
                                    .child(
                                        row()
                                            .gap(12.0)
                                            .child(
                                                container()
                                                    .size(80.0, 80.0)
                                                    .background(colors::WHITE.with_alpha(0.3))
                                                    .corner_radius(8.0)
                                            )
                                            .child(
                                                column()
                                                    .gap(8.0)
                                                    .child(
                                                        container()
                                                            .size(100.0, 32.0)
                                                            .background(colors::WHITE.with_alpha(0.3))
                                                            .corner_radius(4.0)
                                                    )
                                                    .child(
                                                        container()
                                                            .size(80.0, 32.0)
                                                            .background(colors::WHITE.with_alpha(0.3))
                                                            .corner_radius(4.0)
                                                    )
                                            )
                                            .child(
                                                container()
                                                    .size(60.0, 80.0)
                                                    .background(colors::WHITE.with_alpha(0.3))
                                                    .corner_radius(8.0)
                                            )
                                    )
                            )
                            .child(
                                // Status bar
                                container()
                                    .width_full()
                                    .background(colors::GRAY_800)
                                    .corner_radius(6.0)
                                    .padding(12.0)
                                    .child(
                                        text(
                                            format!(
                                                "Debug: {} | Panels: {}",
                                                if debug_ref.is_enabled() { "ON" } else { "OFF" },
                                                debug_ref.state().active_panels().count()
                                            ),
                                            TextStyle {
                                                color: colors::GREEN_400,
                                                size: 12.0,
                                            },
                                        )
                                    )
                            )
                    )
                },
            );

            // Debug overlay layer (highest z-index)
            // Note: In a real integration, the debug overlay would be integrated
            // into the layer system more deeply to access element bounds and hit test data
        })
        .run();
}

/// Helper function to create consistent text lines
fn text_line(content: &str) -> impl sol_ui::element::Element {
    text(
        content,
        TextStyle {
            color: colors::GRAY_600,
            size: 13.0,
        },
    )
}
