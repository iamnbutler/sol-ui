//! Modal/Dialog element demo
//!
//! Demonstrates modal and dialog elements:
//! - Basic modal with custom content
//! - Confirm dialog pattern
//! - Destructive action dialog
//! - Escape key and backdrop click to close

use sol_ui::{
    app::app,
    color::colors,
    element::{button, column, confirm_dialog, container, modal, row, text},
    layer::LayerOptions,
    style::TextStyle,
};
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    // State for modal visibility
    let basic_modal_open = Rc::new(RefCell::new(false));
    let confirm_dialog_open = Rc::new(RefCell::new(false));
    let delete_dialog_open = Rc::new(RefCell::new(false));

    // State for showing results
    let last_action = Rc::new(RefCell::new(String::from("No action yet")));

    app()
        .title("Modal/Dialog Demo")
        .size(700.0, 600.0)
        .with_layers(move |layers| {
            let basic_modal_open_clone = basic_modal_open.clone();
            let confirm_dialog_open_clone = confirm_dialog_open.clone();
            let delete_dialog_open_clone = delete_dialog_open.clone();
            let last_action_clone = last_action.clone();

            layers.add_ui_layer(
                0,
                LayerOptions::default().with_input().with_clear(),
                move || {
                    let basic_open = *basic_modal_open_clone.borrow();
                    let confirm_open = *confirm_dialog_open_clone.borrow();
                    let delete_open = *delete_dialog_open_clone.borrow();
                    let action_text = last_action_clone.borrow().clone();

                    // Clones for button handlers
                    let open_basic = basic_modal_open_clone.clone();
                    let open_confirm = confirm_dialog_open_clone.clone();
                    let open_delete = delete_dialog_open_clone.clone();

                    // Clones for modal close handlers
                    let close_basic = basic_modal_open_clone.clone();
                    let close_confirm = confirm_dialog_open_clone.clone();
                    let close_delete = delete_dialog_open_clone.clone();

                    // Action result updaters
                    let confirm_action = last_action_clone.clone();
                    let confirm_cancel = last_action_clone.clone();
                    let delete_action = last_action_clone.clone();
                    let delete_cancel = last_action_clone.clone();

                    Box::new(
                        container()
                            .width_full()
                            .height_full()
                            .background(colors::GRAY_100)
                            .flex_col()
                            .padding(40.0)
                            .gap(24.0)
                            // Title
                            .child(
                                text(
                                    "Modal/Dialog Demo",
                                    TextStyle {
                                        color: colors::BLACK,
                                        size: 28.0,
                                    },
                                )
                            )
                            // Instructions
                            .child(
                                text(
                                    "Click the buttons below to open different modal types. Close with Escape key or click outside.",
                                    TextStyle {
                                        color: colors::GRAY_600,
                                        size: 14.0,
                                    },
                                )
                            )
                            // Buttons row
                            .child(
                                row()
                                    .gap(16.0)
                                    .child(
                                        button("Open Basic Modal")
                                            .background(colors::BLUE_500)
                                            .hover_background(colors::BLUE_400)
                                            .press_background(colors::BLUE_600)
                                            .on_click_simple(move || {
                                                *open_basic.borrow_mut() = true;
                                            })
                                    )
                                    .child(
                                        button("Open Confirm Dialog")
                                            .background(colors::GREEN_500)
                                            .hover_background(colors::GREEN_400)
                                            .press_background(colors::GREEN_600)
                                            .on_click_simple(move || {
                                                *open_confirm.borrow_mut() = true;
                                            })
                                    )
                                    .child(
                                        button("Open Delete Dialog")
                                            .background(colors::RED_500)
                                            .hover_background(colors::RED_400)
                                            .press_background(colors::RED_600)
                                            .on_click_simple(move || {
                                                *open_delete.borrow_mut() = true;
                                            })
                                    )
                            )
                            // Last action display
                            .child(
                                container()
                                    .background(colors::WHITE)
                                    .padding(16.0)
                                    .corner_radius(8.0)
                                    .child(
                                        text(
                                            format!("Last action: {}", action_text),
                                            TextStyle {
                                                color: colors::GRAY_700,
                                                size: 14.0,
                                            },
                                        )
                                    )
                            )
                            // Basic Modal
                            .child(
                                modal(basic_open, move || {
                                    *close_basic.borrow_mut() = false;
                                })
                                .content(
                                    column()
                                        .gap(16.0)
                                        .child(
                                            text(
                                                "Basic Modal",
                                                TextStyle {
                                                    color: colors::GRAY_900,
                                                    size: 18.0,
                                                },
                                            )
                                        )
                                        .child(
                                            text(
                                                "This is a basic modal with custom content. Press Escape or click the backdrop to close.",
                                                TextStyle {
                                                    color: colors::GRAY_600,
                                                    size: 14.0,
                                                },
                                            )
                                        )
                                )
                            )
                            // Confirm Dialog
                            .child(
                                confirm_dialog(
                                    confirm_open,
                                    "Save Changes?",
                                    "Do you want to save your changes before closing?",
                                    move || {
                                        *confirm_action.borrow_mut() = "Confirmed: Saved changes".to_string();
                                        *close_confirm.borrow_mut() = false;
                                    },
                                    move || {
                                        *confirm_cancel.borrow_mut() = "Cancelled: Changes discarded".to_string();
                                    },
                                )
                                .confirm_label("Save")
                                .cancel_label("Discard")
                            )
                            // Delete Dialog (destructive)
                            .child(
                                confirm_dialog(
                                    delete_open,
                                    "Delete Item?",
                                    "This action cannot be undone. Are you sure you want to delete this item?",
                                    move || {
                                        *delete_action.borrow_mut() = "Confirmed: Item deleted!".to_string();
                                        *close_delete.borrow_mut() = false;
                                    },
                                    move || {
                                        *delete_cancel.borrow_mut() = "Cancelled: Item not deleted".to_string();
                                    },
                                )
                                .confirm_label("Delete")
                                .cancel_label("Keep")
                                .destructive()
                            )
                    )
                },
            );
        })
        .run();
}
