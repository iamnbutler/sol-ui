//! Keyboard input example demonstrating focus management and keyboard events
//!
//! This example shows:
//! - Focusable elements with visual focus indicators
//! - Tab navigation between focusable elements
//! - Keyboard event handling (key down/up)
//! - Character input handling

use sol_ui::{
    app::app,
    color::{ColorExt, colors},
    element::{column, container, text},
    interaction::Interactable,
    layer::{Key, LayerOptions, MouseButton},
};
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    // Shared state
    let last_key = Rc::new(RefCell::new(String::from("None")));
    let typed_text = Rc::new(RefCell::new(String::new()));
    let focused_box = Rc::new(RefCell::new(String::from("None")));

    app()
        .title("Keyboard Input Example")
        .size(800.0, 600.0)
        .with_layers(move |layers| {
            let last_key_clone = last_key.clone();
            let typed_text_clone = typed_text.clone();
            let focused_box_clone = focused_box.clone();

            layers.add_ui_layer(
                0,
                LayerOptions::default().with_input().with_clear(),
                move || {
                    let last_key_val = last_key_clone.borrow().clone();
                    let typed_text_val = typed_text_clone.borrow().clone();
                    let focused_box_val = focused_box_clone.borrow().clone();

                    let last_key_clone2 = last_key_clone.clone();
                    let typed_text_clone2 = typed_text_clone.clone();
                    let focused_box_clone2 = focused_box_clone.clone();

                    container()
                        .width_full()
                        .height_full()
                        .background(colors::GRAY_100)
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .gap(20.0)
                        .child(text("Keyboard Input Demo")
                            .size(32.0)
                            .color(colors::BLACK))
                        .child(text("Press Tab to navigate between boxes, type to see input")
                            .size(16.0)
                            .color(colors::GRAY_600))
                            // Status display
                            .child(
                                column()
                                    .gap(8.0)
                                    .child(text(format!("Last key: {}", last_key_val))
                                        .size(18.0)
                                        .color(colors::GRAY_700))
                                    .child(text(format!("Focused: {}", focused_box_val))
                                        .size(18.0)
                                        .color(colors::GRAY_700))
                                    .child(text(format!("Typed: {}", if typed_text_val.is_empty() { "(empty)" } else { &typed_text_val }))
                                        .size(18.0)
                                        .color(colors::GRAY_700))
                            )
                            // Focusable input boxes
                            .child(
                                column()
                                    .gap(16.0)
                                    .child(
                                        // Box 1 - Blue input area
                                        container()
                                            .width(300.0)
                                            .height(60.0)
                                            .background(colors::BLUE_400.with_alpha(0.3))
                                            .border(colors::BLUE_500, 2.0)
                                            .corner_radius(8.0)
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(text("Box 1 (Blue)")
                                                .size(16.0)
                                                .color(colors::BLUE_600))
                                            .interactive()
                                            .with_id(1)
                                            .focusable_with_overlay(colors::BLUE_500.with_alpha(0.3))
                                            .hover_overlay(colors::BLUE_500.with_alpha(0.1))
                                            .on_focus_in({
                                                let focused = focused_box_clone2.clone();
                                                move || {
                                                    *focused.borrow_mut() = "Box 1 (Blue)".to_string();
                                                    println!("Box 1 gained focus");
                                                }
                                            })
                                            .on_focus_out(|| {
                                                println!("Box 1 lost focus");
                                            })
                                            .on_key_down({
                                                let last_key = last_key_clone2.clone();
                                                let typed_text = typed_text_clone2.clone();
                                                move |key, modifiers, character, is_repeat| {
                                                    // Update last key display
                                                    let key_str = format!("{:?}{}", key, if is_repeat { " (repeat)" } else { "" });
                                                    *last_key.borrow_mut() = key_str;

                                                    // Handle backspace
                                                    if key == Key::Backspace {
                                                        typed_text.borrow_mut().pop();
                                                    }
                                                    // Handle character input
                                                    else if let Some(c) = character {
                                                        if !modifiers.cmd && !modifiers.ctrl {
                                                            typed_text.borrow_mut().push(c);
                                                        }
                                                    }

                                                    println!("Box 1: KeyDown {:?} char={:?} mods={:?}", key, character, modifiers);
                                                }
                                            })
                                            .on_click({
                                                let focused = focused_box_clone2.clone();
                                                move |button, _, _| {
                                                    if button == MouseButton::Left {
                                                        *focused.borrow_mut() = "Box 1 (Blue)".to_string();
                                                        println!("Box 1 clicked");
                                                    }
                                                }
                                            })
                                    )
                                    .child(
                                        // Box 2 - Green input area
                                        container()
                                            .width(300.0)
                                            .height(60.0)
                                            .background(colors::GREEN_400.with_alpha(0.3))
                                            .border(colors::GREEN_500, 2.0)
                                            .corner_radius(8.0)
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(text("Box 2 (Green)")
                                                .size(16.0)
                                                .color(colors::GREEN_600))
                                            .interactive()
                                            .with_id(2)
                                            .focusable_with_overlay(colors::GREEN_500.with_alpha(0.3))
                                            .hover_overlay(colors::GREEN_500.with_alpha(0.1))
                                            .on_focus_in({
                                                let focused = focused_box_clone2.clone();
                                                move || {
                                                    *focused.borrow_mut() = "Box 2 (Green)".to_string();
                                                    println!("Box 2 gained focus");
                                                }
                                            })
                                            .on_focus_out(|| {
                                                println!("Box 2 lost focus");
                                            })
                                            .on_key_down({
                                                let last_key = last_key_clone2.clone();
                                                let typed_text = typed_text_clone2.clone();
                                                move |key, modifiers, character, is_repeat| {
                                                    let key_str = format!("{:?}{}", key, if is_repeat { " (repeat)" } else { "" });
                                                    *last_key.borrow_mut() = key_str;

                                                    if key == Key::Backspace {
                                                        typed_text.borrow_mut().pop();
                                                    } else if let Some(c) = character {
                                                        if !modifiers.cmd && !modifiers.ctrl {
                                                            typed_text.borrow_mut().push(c);
                                                        }
                                                    }

                                                    println!("Box 2: KeyDown {:?} char={:?} mods={:?}", key, character, modifiers);
                                                }
                                            })
                                            .on_click({
                                                let focused = focused_box_clone2.clone();
                                                move |button, _, _| {
                                                    if button == MouseButton::Left {
                                                        *focused.borrow_mut() = "Box 2 (Green)".to_string();
                                                        println!("Box 2 clicked");
                                                    }
                                                }
                                            })
                                    )
                                    .child(
                                        // Box 3 - Purple input area
                                        container()
                                            .width(300.0)
                                            .height(60.0)
                                            .background(colors::PURPLE_400.with_alpha(0.3))
                                            .border(colors::PURPLE_500, 2.0)
                                            .corner_radius(8.0)
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(text("Box 3 (Purple)")
                                                .size(16.0)
                                                .color(colors::PURPLE_600))
                                            .interactive()
                                            .with_id(3)
                                            .focusable_with_overlay(colors::PURPLE_500.with_alpha(0.3))
                                            .hover_overlay(colors::PURPLE_500.with_alpha(0.1))
                                            .on_focus_in({
                                                let focused = focused_box_clone2.clone();
                                                move || {
                                                    *focused.borrow_mut() = "Box 3 (Purple)".to_string();
                                                    println!("Box 3 gained focus");
                                                }
                                            })
                                            .on_focus_out(|| {
                                                println!("Box 3 lost focus");
                                            })
                                            .on_key_down({
                                                let last_key = last_key_clone2.clone();
                                                let typed_text = typed_text_clone2.clone();
                                                move |key, modifiers, character, is_repeat| {
                                                    let key_str = format!("{:?}{}", key, if is_repeat { " (repeat)" } else { "" });
                                                    *last_key.borrow_mut() = key_str;

                                                    if key == Key::Backspace {
                                                        typed_text.borrow_mut().pop();
                                                    } else if let Some(c) = character {
                                                        if !modifiers.cmd && !modifiers.ctrl {
                                                            typed_text.borrow_mut().push(c);
                                                        }
                                                    }

                                                    println!("Box 3: KeyDown {:?} char={:?} mods={:?}", key, character, modifiers);
                                                }
                                            })
                                            .on_click({
                                                let focused = focused_box_clone2.clone();
                                                move |button, _, _| {
                                                    if button == MouseButton::Left {
                                                        *focused.borrow_mut() = "Box 3 (Purple)".to_string();
                                                        println!("Box 3 clicked");
                                                    }
                                                }
                                            })
                                    )
                            )
                            .child(text("Tips: Tab/Shift+Tab to navigate | Escape to clear | Type any key")
                                .size(14.0)
                                .color(colors::GRAY_500))
                },
            );
        })
        .run();
}
