//! Button Element Demo
//!
//! Demonstrates the Button element with various styles and states.
//! The Button element provides:
//! - Automatic hover and press visual states
//! - Configurable colors for each state
//! - Optional border and corner radius
//! - Disabled state support
//! - Builder-pattern API

use sol_ui::{
    app::app,
    color::colors,
    element::{button, column, container, row, text},
    layer::LayerOptions,
    style::TextStyle,
};
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let click_count = Rc::new(RefCell::new(0));

    app()
        .title("Button Element Demo")
        .size(800.0, 600.0)
        .with_layers(move |layers| {
            let click_count_clone = click_count.clone();

            layers.add_ui_layer(
                0,
                LayerOptions::default().with_input().with_clear(),
                move || {
                    let count = *click_count_clone.borrow();
                    let click_count_inc = click_count_clone.clone();
                    let click_count_dec = click_count_clone.clone();
                    let click_count_reset = click_count_clone.clone();

                    Box::new(
                        container()
                            .width_full()
                            .height_full()
                            .background(colors::GRAY_100)
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .gap(30.0)
                            // Title
                            .child(
                                text(
                                    "Button Element Demo",
                                    TextStyle {
                                        color: colors::BLACK,
                                        size: 28.0,
                                    line_height: 1.2,
                                    },
                                )
                            )
                            // Counter display
                            .child(
                                text(
                                    format!("Count: {}", count),
                                    TextStyle {
                                        color: colors::GRAY_700,
                                        size: 24.0,
                                    line_height: 1.2,
                                    },
                                )
                            )
                            // Default styled buttons
                            .child(
                                column()
                                    .gap(12.0)
                                    .items_center()
                                    .child(
                                        text(
                                            "Default Buttons",
                                            TextStyle {
                                                color: colors::GRAY_600,
                                                size: 14.0,
                                    line_height: 1.2,
                                            },
                                        )
                                    )
                                    .child(
                                        row()
                                            .gap(10.0)
                                            .child(
                                                button("Increment")
                                                    .with_id(1)
                                                    .on_click_simple({
                                                        let count = click_count_inc.clone();
                                                        move || {
                                                            *count.borrow_mut() += 1;
                                                        }
                                                    })
                                            )
                                            .child(
                                                button("Decrement")
                                                    .with_id(2)
                                                    .background(colors::RED_500)
                                                    .hover_background(colors::RED_400)
                                                    .press_background(colors::RED_600)
                                                    .on_click_simple({
                                                        let count = click_count_dec.clone();
                                                        move || {
                                                            *count.borrow_mut() -= 1;
                                                        }
                                                    })
                                            )
                                            .child(
                                                button("Reset")
                                                    .with_id(3)
                                                    .background(colors::GRAY_500)
                                                    .hover_background(colors::GRAY_400)
                                                    .press_background(colors::GRAY_600)
                                                    .on_click_simple({
                                                        let count = click_count_reset.clone();
                                                        move || {
                                                            *count.borrow_mut() = 0;
                                                        }
                                                    })
                                            )
                                    )
                            )
                            // Custom styled buttons
                            .child(
                                column()
                                    .gap(12.0)
                                    .items_center()
                                    .child(
                                        text(
                                            "Custom Styled Buttons",
                                            TextStyle {
                                                color: colors::GRAY_600,
                                                size: 14.0,
                                    line_height: 1.2,
                                            },
                                        )
                                    )
                                    .child(
                                        row()
                                            .gap(10.0)
                                            // Rounded button
                                            .child(
                                                button("Rounded")
                                                    .with_id(4)
                                                    .corner_radius(20.0)
                                                    .padding(20.0, 10.0)
                                                    .background(colors::GREEN_500)
                                                    .hover_background(colors::GREEN_400)
                                                    .press_background(colors::GREEN_600)
                                            )
                                            // Button with border
                                            .child(
                                                button("Outlined")
                                                    .with_id(5)
                                                    .background(colors::TRANSPARENT)
                                                    .hover_background(colors::GRAY_200)
                                                    .press_background(colors::GRAY_300)
                                                    .border(colors::BLUE_500, 2.0)
                                                    .text_color(colors::BLUE_500)
                                            )
                                            // Large button
                                            .child(
                                                button("Large")
                                                    .with_id(6)
                                                    .padding(24.0, 14.0)
                                                    .text_size(18.0)
                                                    .background(colors::PURPLE_500)
                                                    .hover_background(colors::PURPLE_400)
                                                    .press_background(colors::PURPLE_600)
                                            )
                                    )
                            )
                            // Disabled button
                            .child(
                                column()
                                    .gap(12.0)
                                    .items_center()
                                    .child(
                                        text(
                                            "Disabled State",
                                            TextStyle {
                                                color: colors::GRAY_600,
                                                size: 14.0,
                                    line_height: 1.2,
                                            },
                                        )
                                    )
                                    .child(
                                        button("Disabled Button")
                                            .with_id(7)
                                            .disabled(true)
                                            .on_click_simple(|| {
                                                println!("This should never be called!");
                                            })
                                    )
                            )
                            // Help text
                            .child(
                                text(
                                    "Hover and click buttons to see state changes",
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
