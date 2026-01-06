//! Drag and drop demo
//!
//! Run with: cargo run --example drag_drop_demo

use palette::Srgba;
use sol_ui::{
    app::app,
    color::colors,
    element::{column, container, row, text},
    interaction::Interactable,
    layer::{LayerOptions, MouseButton},
    style::TextStyle,
};
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // State for tracking drops
    let drop_count = Rc::new(RefCell::new(0));
    let last_dropped = Rc::new(RefCell::new(String::from("(none)")));

    app()
        .title("Drag and Drop Demo")
        .size(800.0, 600.0)
        .with_layers(move |layers| {
            let drop_count_clone = drop_count.clone();
            let last_dropped_clone = last_dropped.clone();

            layers.add_ui_layer(
                0,
                LayerOptions::default().with_input().with_clear(),
                move || {
                    let drop_count_inner = drop_count_clone.clone();
                    let last_dropped_inner = last_dropped_clone.clone();

                    Box::new(
                        container()
                            .width_full()
                            .height_full()
                            .background(colors::GRAY_100)
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .gap(30.0)
                            .child(
                                text(
                                    "Drag and Drop Demo",
                                    TextStyle {
                                        size: 28.0,
                                        color: colors::BLACK,
                                        line_height: 1.2,
                                        ..Default::default()
                                    },
                                )
                            )
                            .child(
                                text(
                                    "Note: Full drag and drop interaction requires element integration.\nThis demo shows the infrastructure is in place.",
                                    TextStyle {
                                        size: 14.0,
                                        color: colors::GRAY_600,
                                        line_height: 1.4,
                                        ..Default::default()
                                    },
                                )
                            )
                            .child(
                                row()
                                    .gap(40.0)
                                    // Draggable items column
                                    .child(
                                        column()
                                            .gap(10.0)
                                            .child(
                                                text(
                                                    "Draggable Items",
                                                    TextStyle {
                                                        size: 16.0,
                                                        color: colors::GRAY_700,
                                                        line_height: 1.2,
                                                        ..Default::default()
                                                    },
                                                )
                                            )
                                            .child(draggable_item("Item A", colors::BLUE_400))
                                            .child(draggable_item("Item B", colors::GREEN_400))
                                            .child(draggable_item("Item C", colors::PURPLE_400))
                                    )
                                    // Drop zone column
                                    .child(
                                        column()
                                            .gap(10.0)
                                            .child(
                                                text(
                                                    "Drop Zone",
                                                    TextStyle {
                                                        size: 16.0,
                                                        color: colors::GRAY_700,
                                                        line_height: 1.2,
                                                        ..Default::default()
                                                    },
                                                )
                                            )
                                            .child(
                                                container()
                                                    .width(200.0)
                                                    .height(200.0)
                                                    .background(Srgba::new(0.9, 0.95, 1.0, 1.0))
                                                    .border(colors::BLUE_400, 2.0)
                                                    .corner_radius(8.0)
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .child(
                                                        text(
                                                            "Drop items here",
                                                            TextStyle {
                                                                size: 14.0,
                                                                color: colors::GRAY_500,
                                                                line_height: 1.2,
                                                                ..Default::default()
                                                            },
                                                        )
                                                    )
                                                    .interactive()
                                                    .with_id(100)
                                            )
                                    )
                            )
                            .child(
                                column()
                                    .gap(8.0)
                                    .child(
                                        text(
                                            format!("Drop count: {}", drop_count_inner.borrow()),
                                            TextStyle {
                                                size: 14.0,
                                                color: colors::GRAY_600,
                                                line_height: 1.2,
                                                ..Default::default()
                                            },
                                        )
                                    )
                                    .child(
                                        text(
                                            format!("Last dropped: {}", last_dropped_inner.borrow()),
                                            TextStyle {
                                                size: 14.0,
                                                color: colors::GRAY_600,
                                                line_height: 1.2,
                                                ..Default::default()
                                            },
                                        )
                                    )
                            )
                    )
                },
            );
        })
        .run();
}

/// Create a draggable item container
fn draggable_item(label: &str, color: Srgba<f32>) -> impl sol_ui::element::Element {
    let label_owned = label.to_string();
    let label_for_handler = label.to_string();

    container()
        .width(150.0)
        .height(50.0)
        .background(color)
        .corner_radius(6.0)
        .flex()
        .items_center()
        .justify_center()
        .child(
            text(
                &label_owned,
                TextStyle {
                    size: 14.0,
                    color: colors::WHITE,
                    line_height: 1.2,
                    ..Default::default()
                },
            )
        )
        .interactive()
        .with_key(format!("draggable-{}", label_owned))
        .hover_overlay(Srgba::new(1.0, 1.0, 1.0, 0.2))
        .press_overlay(Srgba::new(0.0, 0.0, 0.0, 0.2))
        .on_mouse_down(move |button, pos, _, _, _| {
            if button == MouseButton::Left {
                println!("Started drag on {} at {:?}", label_for_handler, pos);
            }
        })
}
