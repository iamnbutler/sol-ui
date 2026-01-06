//! Modal dialog demo

use sol_ui::{
    app::app,
    color::colors,
    element::{button, column, container, modal, text},
    layer::LayerOptions,
    style::TextStyle,
};
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    // Track whether modal is open
    let is_open = Rc::new(RefCell::new(false));

    app()
        .title("Modal Dialog Demo")
        .size(800.0, 600.0)
        .with_layers(move |layers| {
            let is_open_clone = is_open.clone();

            layers.add_ui_layer(
                0,
                LayerOptions::default().with_input().with_clear(),
                move || {
                    let modal_open = *is_open_clone.borrow();
                    let is_open_for_open = is_open_clone.clone();
                    let is_open_for_close = is_open_clone.clone();

                    Box::new(
                        container()
                            .width_full()
                            .height_full()
                            .background(colors::GRAY_100)
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .gap(20.0)
                            .child(text(
                                "Modal Dialog Demo",
                                TextStyle {
                                    color: colors::BLACK,
                                    size: 32.0,
                                },
                            ))
                            .child(text(
                                "Click the button below to open a modal",
                                TextStyle {
                                    color: colors::GRAY_600,
                                    size: 16.0,
                                },
                            ))
                            .child(
                                button("Open Modal")
                                    .on_click_simple({
                                        let is_open = is_open_for_open.clone();
                                        move || {
                                            *is_open.borrow_mut() = true;
                                            println!("Modal opened");
                                        }
                                    }),
                            )
                            // Modal overlay (only visible when open)
                            .child(
                                modal()
                                    .open(modal_open)
                                    .on_close({
                                        let is_open = is_open_for_close.clone();
                                        move || {
                                            *is_open.borrow_mut() = false;
                                            println!("Modal closed");
                                        }
                                    })
                                    .child(
                                        column()
                                            .gap(16.0)
                                            .child(text(
                                                "Modal Title",
                                                TextStyle {
                                                    color: colors::BLACK,
                                                    size: 24.0,
                                                },
                                            ))
                                            .child(text(
                                                "This is a modal dialog. Click the backdrop",
                                                TextStyle {
                                                    color: colors::GRAY_600,
                                                    size: 14.0,
                                                },
                                            ))
                                            .child(text(
                                                "or press Escape to close it.",
                                                TextStyle {
                                                    color: colors::GRAY_600,
                                                    size: 14.0,
                                                },
                                            ))
                                            .child(
                                                button("Close")
                                                    .on_click_simple({
                                                        let is_open = is_open_for_close.clone();
                                                        move || {
                                                            *is_open.borrow_mut() = false;
                                                            println!("Close button clicked");
                                                        }
                                                    }),
                                            ),
                                    ),
                            ),
                    )
                },
            );
        })
        .run();
}
