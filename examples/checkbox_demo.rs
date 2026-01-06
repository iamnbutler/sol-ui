//! Checkbox element demo
//!
//! Demonstrates the checkbox element with various configurations:
//! - Basic checked/unchecked states
//! - Labels
//! - Disabled state
//! - Custom colors
//! - Interactive toggling

use sol_ui::{
    app::app,
    color::colors,
    element::{checkbox, column, container, row, text, CheckboxInteractable},
    layer::LayerOptions,
    style::TextStyle,
};
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    // Shared state for checkboxes
    let option1 = Rc::new(RefCell::new(false));
    let option2 = Rc::new(RefCell::new(true));
    let option3 = Rc::new(RefCell::new(false));
    let notifications = Rc::new(RefCell::new(true));
    let dark_mode = Rc::new(RefCell::new(false));

    app()
        .title("Checkbox Demo")
        .size(600.0, 500.0)
        .with_layers(move |layers| {
            let option1_clone = option1.clone();
            let option2_clone = option2.clone();
            let option3_clone = option3.clone();
            let notifications_clone = notifications.clone();
            let dark_mode_clone = dark_mode.clone();

            layers.add_ui_layer(
                0,
                LayerOptions::default().with_input().with_clear(),
                move || {
                    let opt1 = *option1_clone.borrow();
                    let opt2 = *option2_clone.borrow();
                    let opt3 = *option3_clone.borrow();
                    let notif = *notifications_clone.borrow();
                    let dark = *dark_mode_clone.borrow();

                    let option1_update = option1_clone.clone();
                    let option2_update = option2_clone.clone();
                    let option3_update = option3_clone.clone();
                    let notifications_update = notifications_clone.clone();
                    let dark_mode_update = dark_mode_clone.clone();

                    Box::new(
                        container()
                            .width_full()
                            .height_full()
                            .background(colors::GRAY_100)
                            .flex_col()
                            .padding(40.0)
                            .gap(30.0)
                            .child(
                                text(
                                    "Checkbox Demo",
                                    TextStyle {
                                        color: colors::BLACK,
                                        size: 28.0,
                                        ..Default::default()
                                    },
                                )
                            )
                            // Basic checkboxes section
                            .child(
                                column()
                                    .gap(12.0)
                                    .child(
                                        text(
                                            "Basic Checkboxes",
                                            TextStyle {
                                                color: colors::GRAY_700,
                                                size: 18.0,
                                                ..Default::default()
                                            },
                                        )
                                    )
                                    .child(
                                        checkbox(opt1)
                                            .label("Option 1 (unchecked by default)")
                                            .with_id(1)
                                            .on_change(move |new_state| {
                                                *option1_update.borrow_mut() = new_state;
                                                println!("Option 1: {}", new_state);
                                            })
                                            .interactive_checkbox()
                                    )
                                    .child(
                                        checkbox(opt2)
                                            .label("Option 2 (checked by default)")
                                            .with_id(2)
                                            .on_change(move |new_state| {
                                                *option2_update.borrow_mut() = new_state;
                                                println!("Option 2: {}", new_state);
                                            })
                                            .interactive_checkbox()
                                    )
                                    .child(
                                        checkbox(opt3)
                                            .label("Option 3")
                                            .with_id(3)
                                            .on_change(move |new_state| {
                                                *option3_update.borrow_mut() = new_state;
                                                println!("Option 3: {}", new_state);
                                            })
                                            .interactive_checkbox()
                                    )
                            )
                            // Settings section
                            .child(
                                column()
                                    .gap(12.0)
                                    .child(
                                        text(
                                            "Settings",
                                            TextStyle {
                                                color: colors::GRAY_700,
                                                size: 18.0,
                                                ..Default::default()
                                            },
                                        )
                                    )
                                    .child(
                                        checkbox(notif)
                                            .label("Enable notifications")
                                            .with_id(4)
                                            .checked_background(colors::GREEN_500)
                                            .on_change(move |new_state| {
                                                *notifications_update.borrow_mut() = new_state;
                                                println!("Notifications: {}", new_state);
                                            })
                                            .interactive_checkbox()
                                    )
                                    .child(
                                        checkbox(dark)
                                            .label("Dark mode")
                                            .with_id(5)
                                            .checked_background(colors::PURPLE_500)
                                            .on_change(move |new_state| {
                                                *dark_mode_update.borrow_mut() = new_state;
                                                println!("Dark mode: {}", new_state);
                                            })
                                            .interactive_checkbox()
                                    )
                            )
                            // Disabled checkbox section
                            .child(
                                column()
                                    .gap(12.0)
                                    .child(
                                        text(
                                            "Disabled States",
                                            TextStyle {
                                                color: colors::GRAY_700,
                                                size: 18.0,
                                                ..Default::default()
                                            },
                                        )
                                    )
                                    .child(
                                        checkbox(false)
                                            .label("Disabled unchecked")
                                            .disabled(true)
                                            .with_id(6)
                                            .interactive_checkbox()
                                    )
                                    .child(
                                        checkbox(true)
                                            .label("Disabled checked")
                                            .disabled(true)
                                            .with_id(7)
                                            .interactive_checkbox()
                                    )
                            )
                            // Size variations
                            .child(
                                column()
                                    .gap(12.0)
                                    .child(
                                        text(
                                            "Size Variations",
                                            TextStyle {
                                                color: colors::GRAY_700,
                                                size: 18.0,
                                                ..Default::default()
                                            },
                                        )
                                    )
                                    .child(
                                        row()
                                            .gap(20.0)
                                            .items_center()
                                            .child(
                                                checkbox(true)
                                                    .size(14.0)
                                                    .label("Small")
                                                    .label_style(TextStyle {
                                                        color: colors::BLACK,
                                                        size: 12.0,
                                                        ..Default::default()
                                                    })
                                                    .with_id(8)
                                                    .interactive_checkbox()
                                            )
                                            .child(
                                                checkbox(true)
                                                    .size(20.0)
                                                    .label("Medium")
                                                    .with_id(9)
                                                    .interactive_checkbox()
                                            )
                                            .child(
                                                checkbox(true)
                                                    .size(28.0)
                                                    .label("Large")
                                                    .label_style(TextStyle {
                                                        color: colors::BLACK,
                                                        size: 18.0,
                                                        ..Default::default()
                                                    })
                                                    .with_id(10)
                                                    .interactive_checkbox()
                                            )
                                    )
                            )
                            // Status display
                            .child(
                                container()
                                    .background(colors::WHITE)
                                    .padding(16.0)
                                    .corner_radius(8.0)
                                    .child(
                                        text(
                                            format!(
                                                "Current state: Option1={}, Option2={}, Option3={}, Notifications={}, DarkMode={}",
                                                opt1, opt2, opt3, notif, dark
                                            ),
                                            TextStyle {
                                                color: colors::GRAY_600,
                                                size: 14.0,
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
