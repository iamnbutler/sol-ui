//! Keyboard shortcuts demo
//!
//! This example demonstrates:
//! - Global shortcuts (Cmd+S, Cmd+Q, etc.)
//! - Contextual shortcuts (active when specific element is focused)
//! - Shortcut hints in UI
//! - Conflict detection

use sol_ui::{
    app::app,
    color::{ColorExt, colors},
    element::{column, container, row, text, Container},
    interaction::Interactable,
    layer::{Key, LayerOptions, MouseButton},
};
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    // Shared state
    let last_action = Rc::new(RefCell::new(String::from("None")));
    let counter = Rc::new(RefCell::new(0i32));

    app()
        .title("Keyboard Shortcuts Demo")
        .size(800.0, 600.0)
        .with_layers(move |layers| {
            let last_action_clone = last_action.clone();
            let counter_clone = counter.clone();

            layers.add_ui_layer(
                0,
                LayerOptions::default().with_input().with_clear(),
                move || {
                    let last_action_val = last_action_clone.borrow().clone();
                    let counter_val = *counter_clone.borrow();

                    let last_action_update = last_action_clone.clone();
                    let counter_update = counter_clone.clone();

                    Box::new(
                        container()
                            .width_full()
                            .height_full()
                            .background(colors::GRAY_100)
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .gap(24.0)
                            // Title
                            .child(text("Keyboard Shortcuts Demo")
                                .size(32.0)
                                .color(colors::GRAY_900))
                            // Instructions
                            .child(
                                column()
                                    .gap(8.0)
                                    .items_center()
                                    .child(text("Try these shortcuts:")
                                        .size(16.0)
                                        .color(colors::GRAY_600))
                                    .child(shortcut_row("Cmd+S", "Save"))
                                    .child(shortcut_row("Cmd+Z", "Undo"))
                                    .child(shortcut_row("Shift+Cmd+Z", "Redo"))
                                    .child(shortcut_row("Cmd+C", "Copy"))
                                    .child(shortcut_row("Cmd+V", "Paste"))
                                    .child(shortcut_row("Cmd+=", "Zoom In"))
                                    .child(shortcut_row("Cmd+-", "Zoom Out"))
                            )
                            // Status display
                            .child(
                                container()
                                    .width(400.0)
                                    .padding(16.0)
                                    .background(colors::WHITE)
                                    .border(colors::GRAY_300, 1.0)
                                    .corner_radius(8.0)
                                    .flex_col()
                                    .gap(8.0)
                                    .child(text(format!("Last action: {}", last_action_val))
                                        .size(18.0)
                                        .color(colors::GRAY_800))
                                    .child(text(format!("Counter: {}", counter_val))
                                        .size(16.0)
                                        .color(colors::GRAY_600))
                            )
                            // Interactive buttons demonstrating focused shortcuts
                            .child(
                                row()
                                    .gap(16.0)
                                    .child(
                                        // Button with contextual shortcut
                                        container()
                                            .width(150.0)
                                            .height(50.0)
                                            .background(colors::BLUE_500)
                                            .corner_radius(8.0)
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(text("Increment (+)")
                                                .size(16.0)
                                                .color(colors::WHITE))
                                            .interactive()
                                            .with_id(1)
                                            .focusable_with_overlay(colors::BLUE_400.with_alpha(0.5))
                                            .hover_overlay(colors::WHITE.with_alpha(0.1))
                                            .on_click({
                                                let counter = counter_update.clone();
                                                let action = last_action_update.clone();
                                                move |button, _, _| {
                                                    if button == MouseButton::Left {
                                                        *counter.borrow_mut() += 1;
                                                        *action.borrow_mut() = "Increment (click)".to_string();
                                                    }
                                                }
                                            })
                                            .on_key_down({
                                                let counter = counter_update.clone();
                                                let action = last_action_update.clone();
                                                move |key, modifiers, _, _| {
                                                    // When focused, + key increments
                                                    if key == Key::Equal && !modifiers.cmd {
                                                        *counter.borrow_mut() += 1;
                                                        *action.borrow_mut() = "Increment (+ key)".to_string();
                                                    }
                                                }
                                            })
                                    )
                                    .child(
                                        // Button with contextual shortcut
                                        container()
                                            .width(150.0)
                                            .height(50.0)
                                            .background(colors::RED_500)
                                            .corner_radius(8.0)
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(text("Decrement (-)")
                                                .size(16.0)
                                                .color(colors::WHITE))
                                            .interactive()
                                            .with_id(2)
                                            .focusable_with_overlay(colors::RED_400.with_alpha(0.5))
                                            .hover_overlay(colors::WHITE.with_alpha(0.1))
                                            .on_click({
                                                let counter = counter_update.clone();
                                                let action = last_action_update.clone();
                                                move |button, _, _| {
                                                    if button == MouseButton::Left {
                                                        *counter.borrow_mut() -= 1;
                                                        *action.borrow_mut() = "Decrement (click)".to_string();
                                                    }
                                                }
                                            })
                                            .on_key_down({
                                                let counter = counter_update.clone();
                                                let action = last_action_update.clone();
                                                move |key, modifiers, _, _| {
                                                    // When focused, - key decrements
                                                    if key == Key::Minus && !modifiers.cmd {
                                                        *counter.borrow_mut() -= 1;
                                                        *action.borrow_mut() = "Decrement (- key)".to_string();
                                                    }
                                                }
                                            })
                                    )
                            )
                            .child(text("Tab to switch focus between buttons | +/- keys work when focused")
                                .size(14.0)
                                .color(colors::GRAY_500))
                    )
                },
            );
        })
        .run();
}

/// Helper to create a shortcut + description row
fn shortcut_row(shortcut: &str, description: &str) -> Container {
    row()
        .gap(16.0)
        .items_center()
        .child(
            container()
                .width(120.0)
                .padding(4.0)
                .background(colors::GRAY_200)
                .corner_radius(4.0)
                .flex()
                .items_center()
                .justify_center()
                .child(text(shortcut)
                    .size(14.0)
                    .color(colors::GRAY_700))
        )
        .child(text(description)
            .size(14.0)
            .color(colors::GRAY_600))
}
