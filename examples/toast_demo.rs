//! Toast notification demo

use sol_ui::{
    app::app,
    color::colors,
    element::{button, column, container, row, text, toast, ToastPosition, ToastSeverity},
    layer::LayerOptions,
    style::TextStyle,
};
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    // Track which toasts are visible
    let info_visible = Rc::new(RefCell::new(false));
    let success_visible = Rc::new(RefCell::new(false));
    let warning_visible = Rc::new(RefCell::new(false));
    let error_visible = Rc::new(RefCell::new(false));

    app()
        .title("Toast Notification Demo")
        .size(800.0, 600.0)
        .with_layers(move |layers| {
            let info = info_visible.clone();
            let success = success_visible.clone();
            let warning = warning_visible.clone();
            let error = error_visible.clone();

            layers.add_ui_layer(
                0,
                LayerOptions::default().with_input().with_clear(),
                move || {
                    let info_show = info.clone();
                    let success_show = success.clone();
                    let warning_show = warning.clone();
                    let error_show = error.clone();

                    let info_dismiss = info.clone();
                    let success_dismiss = success.clone();
                    let warning_dismiss = warning.clone();
                    let error_dismiss = error.clone();

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
                                "Toast Notification Demo",
                                TextStyle {
                                    color: colors::BLACK,
                                    size: 32.0,
                                },
                            ))
                            .child(text(
                                "Click buttons to show different toast types",
                                TextStyle {
                                    color: colors::GRAY_600,
                                    size: 16.0,
                                },
                            ))
                            .child(
                                row()
                                    .gap(10.0)
                                    .child(
                                        button("Info Toast")
                                            .background(colors::BLUE_500)
                                            .on_click_simple({
                                                let v = info_show.clone();
                                                move || {
                                                    *v.borrow_mut() = true;
                                                }
                                            }),
                                    )
                                    .child(
                                        button("Success Toast")
                                            .background(colors::GREEN_500)
                                            .on_click_simple({
                                                let v = success_show.clone();
                                                move || {
                                                    *v.borrow_mut() = true;
                                                }
                                            }),
                                    )
                                    .child(
                                        button("Warning Toast")
                                            .backgrounds(
                                                colors::GRAY_700,
                                                colors::GRAY_600,
                                                colors::GRAY_800,
                                            )
                                            .on_click_simple({
                                                let v = warning_show.clone();
                                                move || {
                                                    *v.borrow_mut() = true;
                                                }
                                            }),
                                    )
                                    .child(
                                        button("Error Toast")
                                            .background(colors::RED_500)
                                            .on_click_simple({
                                                let v = error_show.clone();
                                                move || {
                                                    *v.borrow_mut() = true;
                                                }
                                            }),
                                    ),
                            )
                            // Toast notifications
                            .child(
                                toast("This is an informational message")
                                    .info()
                                    .position(ToastPosition::TopRight)
                                    .visible(*info.borrow())
                                    .on_dismiss({
                                        let v = info_dismiss.clone();
                                        move || {
                                            *v.borrow_mut() = false;
                                        }
                                    }),
                            )
                            .child(
                                toast("Operation completed successfully!")
                                    .success()
                                    .position(ToastPosition::TopCenter)
                                    .visible(*success.borrow())
                                    .on_dismiss({
                                        let v = success_dismiss.clone();
                                        move || {
                                            *v.borrow_mut() = false;
                                        }
                                    }),
                            )
                            .child(
                                toast("Warning: Low disk space")
                                    .warning()
                                    .position(ToastPosition::BottomRight)
                                    .visible(*warning.borrow())
                                    .on_dismiss({
                                        let v = warning_dismiss.clone();
                                        move || {
                                            *v.borrow_mut() = false;
                                        }
                                    }),
                            )
                            .child(
                                toast("Error: Connection failed")
                                    .error()
                                    .position(ToastPosition::BottomCenter)
                                    .visible(*error.borrow())
                                    .on_dismiss({
                                        let v = error_dismiss.clone();
                                        move || {
                                            *v.borrow_mut() = false;
                                        }
                                    }),
                            ),
                    )
                },
            );
        })
        .run();
}
