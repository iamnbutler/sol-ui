//! Interactive UI example demonstrating click and hover interactions

use sol_ui::{
    app::app,
    color::{ColorExt, colors},
    element::{column, container, row, text},
    interaction::Interactable,
    layer::{LayerOptions, MouseButton},
    style::TextStyle,
};
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    // Shared state for the counter
    let counter = Rc::new(RefCell::new(0));
    let hover_count = Rc::new(RefCell::new(0));

    app()
        .title("Interactive UI Example")
        .size(800.0, 600.0)
        .with_layers(move |layers| {
            // Main UI layer
            let counter_clone = counter.clone();
            let hover_count_clone = hover_count.clone();

            layers.add_ui_layer(
                0,
                LayerOptions::default().with_input().with_clear(),
                move || {
                    let count = *counter_clone.borrow();
                    let hover = *hover_count_clone.borrow();
                    let counter_clone2 = counter_clone.clone();
                    let hover_count_clone2 = hover_count_clone.clone();

                    Box::new(
                        container()
                            .width_full()
                            .height_full()
                            .background(colors::GRAY_100)
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .gap(20.0)
                            .child(
                                text(
                                    "Interactive UI Demo",
                                    TextStyle {
                                        color: colors::BLACK,
                                        size: 32.0,
                                    line_height: 1.2,
                                    },
                                )
                            )
                            .child(
                                text(
                                    format!("Click count: {}", count),
                                    TextStyle {
                                        color: colors::GRAY_700,
                                        size: 20.0,
                                    line_height: 1.2,
                                    },
                                )
                            )
                            .child(
                                text(
                                    format!("Hover count: {}", hover),
                                    TextStyle {
                                        color: colors::GRAY_600,
                                        size: 16.0,
                                    line_height: 1.2,
                                    },
                                )
                            )
                            .child(
                                row()
                                    .gap(10.0)
                                    .child(
                                        // Increment button
                                        container()
                                            .width(120.0)
                                            .height(40.0)
                                            .background(colors::BLUE_500)
                                            .corner_radius(8.0)
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(
                                                text(
                                                    "Increment",
                                                    TextStyle {
                                                        color: colors::WHITE,
                                                        size: 16.0,
                                    line_height: 1.2,
                                                    },
                                                )
                                            )
                                            .interactive()
                                            .with_id(1) // Stable ID for increment button
                                            .focusable_with_overlay(colors::BLUE_400.with_alpha(0.4))
                                            .hover_overlay(colors::BLACK.with_alpha(0.1))
                                            .press_overlay(colors::BLACK.with_alpha(0.2))
                                            .on_click({
                                                let counter = counter_clone2.clone();
                                                move |button, _, _| {
                                                    if button == MouseButton::Left {
                                                        *counter.borrow_mut() += 1;
                                                        println!("Increment clicked! New count: {}", *counter.borrow());
                                                    }
                                                }
                                            })
                                            .on_mouse_enter({
                                                let hover_count = hover_count_clone2.clone();
                                                move || {
                                                    *hover_count.borrow_mut() += 1;
                                                    println!("Mouse entered increment button");
                                                }
                                            })
                                            .on_mouse_leave(|| {
                                                println!("Mouse left increment button");
                                            })
                                    )
                                    .child(
                                        // Decrement button
                                        container()
                                            .width(120.0)
                                            .height(40.0)
                                            .background(colors::RED_500)
                                            .corner_radius(8.0)
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(
                                                text(
                                                    "Decrement",
                                                    TextStyle {
                                                        color: colors::WHITE,
                                                        size: 16.0,
                                    line_height: 1.2,
                                                    },
                                                )
                                            )
                                            .interactive()
                                            .with_id(2) // Stable ID for decrement button
                                            .focusable_with_overlay(colors::RED_400.with_alpha(0.4))
                                            .hover_overlay(colors::BLACK.with_alpha(0.1))
                                            .press_overlay(colors::BLACK.with_alpha(0.2))
                                            .on_click({
                                                let counter = counter_clone2.clone();
                                                move |button, _, _| {
                                                    if button == MouseButton::Left {
                                                        *counter.borrow_mut() -= 1;
                                                        println!("Decrement clicked! New count: {}", *counter.borrow());
                                                    }
                                                }
                                            })
                                            .on_mouse_enter({
                                                let hover_count = hover_count_clone2.clone();
                                                move || {
                                                    *hover_count.borrow_mut() += 1;
                                                    println!("Mouse entered decrement button");
                                                }
                                            })
                                            .on_mouse_leave(|| {
                                                println!("Mouse left decrement button");
                                            })
                                    )
                                    .child(
                                        // Reset button
                                        container()
                                            .width(100.0)
                                            .height(40.0)
                                            .background(colors::GRAY_500)
                                            .corner_radius(8.0)
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(
                                                text(
                                                    "Reset",
                                                    TextStyle {
                                                        color: colors::WHITE,
                                                        size: 16.0,
                                    line_height: 1.2,
                                                    },
                                                )
                                            )
                                            .interactive()
                                            .with_id(3) // Stable ID for reset button
                                            .focusable_with_overlay(colors::GRAY_400.with_alpha(0.4))
                                            .hover_overlay(colors::BLACK.with_alpha(0.1))
                                            .press_overlay(colors::BLACK.with_alpha(0.2))
                                            .on_click({
                                                let counter = counter_clone2.clone();
                                                move |button, _, _| {
                                                    if button == MouseButton::Left {
                                                        *counter.borrow_mut() = 0;
                                                        println!("Reset clicked! Count reset to 0");
                                                    }
                                                }
                                            })
                                    )
                            )
                            .child(
                                // Interactive grid demonstrating z-order
                                column()
                                    .gap(10.0)
                                    .child(
                                        text(
                                            "Z-Order Demo (overlapping elements)",
                                            TextStyle {
                                                color: colors::GRAY_700,
                                                size: 18.0,
                                    line_height: 1.2,
                                            },
                                        )
                                    )
                                    .child(
                                        container()
                                            .width(300.0)
                                            .height(200.0)
                                            .child(
                                                // Bottom layer (z-index 0)
                                                container()
                                                    .width(200.0)
                                                    .height(150.0)
                                                    .background(colors::GREEN_400)
                                                    .corner_radius(10.0)
                                                    .margin(10.0)
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .child(
                                                        text(
                                                            "Bottom Layer",
                                                            TextStyle {
                                                                color: colors::WHITE,
                                                                size: 14.0,
                                    line_height: 1.2,
                                                            },
                                                        )
                                                    )
                                                    .interactive()
                                                    .with_id(4) // Stable ID for bottom layer
                                                    .z_index(0)
                                                    .focusable_with_overlay(colors::GREEN_400.with_alpha(0.4))
                                                    .hover_overlay(colors::BLACK.with_alpha(0.1))
                                                    .on_click(|_, _, _| {
                                                        println!("Bottom layer clicked!");
                                                    })
                                            )
                                            .child(
                                                // Top layer (z-index 10) - overlapping
                                                container()
                                                    .width(150.0)
                                                    .height(100.0)
                                                    .background(colors::PURPLE_500)
                                                    .corner_radius(10.0)
                                                    .margin(50.0)
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .child(
                                                        text(
                                                            "Top Layer",
                                                            TextStyle {
                                                                color: colors::WHITE,
                                                                size: 14.0,
                                    line_height: 1.2,
                                                            },
                                                        )
                                                    )
                                                    .interactive()
                                                    .with_id(5) // Stable ID for top layer
                                                    .z_index(10)
                                                    .focusable_with_overlay(colors::PURPLE_400.with_alpha(0.4))
                                                    .hover_overlay(colors::WHITE.with_alpha(0.2))
                                                    .on_click(|_, _, _| {
                                                        println!("Top layer clicked! (This should take precedence)");
                                                    })
                                            )
                                    )
                            )
                            .child(
                                // Disabled button example
                                container()
                                    .width(150.0)
                                    .height(40.0)
                                    .background(colors::GRAY_300)
                                    .corner_radius(8.0)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(
                                        text(
                                            "Disabled Button",
                                            TextStyle {
                                                color: colors::GRAY_500,
                                                size: 16.0,
                                    line_height: 1.2,
                                            },
                                        )
                                    )
                                    .interactive()
                                    .with_id(6) // Stable ID for disabled button
                                    .enabled(false) // This disables interaction
                                    .on_click(|_, _, _| {
                                        println!("This should never be called!");
                                    })
                            )
                            .child(
                                text(
                                    "Press Tab to navigate between focusable elements",
                                    TextStyle {
                                        color: colors::GRAY_500,
                                        size: 14.0,
                                    line_height: 1.2,
                                    },
                                )
                            )
                    )
                },
            );
        })
        .run();
}
