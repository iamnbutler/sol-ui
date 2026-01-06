//! Text input demo demonstrating the TextInput element
//!
//! This example shows:
//! - Single-line text input with cursor
//! - Placeholder text
//! - Selection support (Shift+Arrow, Cmd+A)
//! - Keyboard navigation (Home, End, Arrow keys)
//! - Text editing (typing, backspace, delete)
//! - Focus management with Tab navigation
//! - On-change and on-submit callbacks

use sol_ui::{
    app::app,
    color::{ColorExt, colors},
    element::{column, container, text, text_input, TextInputInteractable, TextInputState},
    entity::new_entity,
    layer::LayerOptions,
    style::TextStyle,
};
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    // Status display
    let status_text = Rc::new(RefCell::new(String::from("Type in the inputs below")));
    let submitted_text = Rc::new(RefCell::new(String::new()));

    app()
        .title("Text Input Demo")
        .size(600.0, 500.0)
        .with_layers(move |layers| {
            let status_clone = status_text.clone();
            let submitted_clone = submitted_text.clone();

            layers.add_ui_layer(
                0,
                LayerOptions::default().with_input().with_clear(),
                move || {
                    let status = status_clone.borrow().clone();
                    let submitted = submitted_clone.borrow().clone();

                    // Create entity for text input state - must be inside render context
                    let input1_state = new_entity(TextInputState::default());
                    let input2_state = new_entity(TextInputState::with_text("Pre-filled text"));
                    let input3_state = new_entity(TextInputState::default());

                    let status_for_change = status_clone.clone();
                    let submitted_for_submit = submitted_clone.clone();

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
                            .child(text(
                                "Text Input Demo",
                                TextStyle {
                                    color: colors::BLACK,
                                    size: 28.0,
                                    ..Default::default()
                                },
                            ))
                            // Instructions
                            .child(text(
                                "Tab to navigate | Type to edit | Shift+Arrow to select | Cmd+A to select all",
                                TextStyle {
                                    color: colors::GRAY_600,
                                    size: 14.0,
                                    ..Default::default()
                                },
                            ))
                            // Status display
                            .child(
                                container()
                                    .padding(12.0)
                                    .background(colors::GRAY_200)
                                    .corner_radius(4.0)
                                    .child(text(
                                        status,
                                        TextStyle {
                                            color: colors::GRAY_700,
                                            size: 14.0,
                                            ..Default::default()
                                        },
                                    )),
                            )
                            // Input fields
                            .child(
                                column()
                                    .gap(16.0)
                                    .width(350.0)
                                    // Input 1 - Basic with placeholder
                                    .child(
                                        column()
                                            .gap(4.0)
                                            .child(text(
                                                "Username",
                                                TextStyle {
                                                    color: colors::GRAY_700,
                                                    size: 12.0,
                                                    ..Default::default()
                                                },
                                            ))
                                            .child(
                                                text_input(input1_state)
                                                    .width(350.0)
                                                    .placeholder("Enter your username...")
                                                    .on_change({
                                                        let status = status_for_change.clone();
                                                        move |text| {
                                                            *status.borrow_mut() =
                                                                format!("Username: {}", text);
                                                        }
                                                    })
                                                    .interactive_input(),
                                            ),
                                    )
                                    // Input 2 - Pre-filled
                                    .child(
                                        column()
                                            .gap(4.0)
                                            .child(text(
                                                "Email",
                                                TextStyle {
                                                    color: colors::GRAY_700,
                                                    size: 12.0,
                                                    ..Default::default()
                                                },
                                            ))
                                            .child(
                                                text_input(input2_state)
                                                    .width(350.0)
                                                    .placeholder("Enter your email...")
                                                    .on_change({
                                                        let status = status_for_change.clone();
                                                        move |text| {
                                                            *status.borrow_mut() =
                                                                format!("Email: {}", text);
                                                        }
                                                    })
                                                    .interactive_input(),
                                            ),
                                    )
                                    // Input 3 - With submit handler
                                    .child(
                                        column()
                                            .gap(4.0)
                                            .child(text(
                                                "Search (press Enter to submit)",
                                                TextStyle {
                                                    color: colors::GRAY_700,
                                                    size: 12.0,
                                                    ..Default::default()
                                                },
                                            ))
                                            .child(
                                                text_input(input3_state)
                                                    .width(350.0)
                                                    .placeholder("Search...")
                                                    .focus_border_color(colors::GREEN_500)
                                                    .on_submit({
                                                        let submitted = submitted_for_submit.clone();
                                                        move |text| {
                                                            *submitted.borrow_mut() =
                                                                format!("Searched for: {}", text);
                                                            println!("Search submitted: {}", text);
                                                        }
                                                    })
                                                    .interactive_input(),
                                            ),
                                    ),
                            )
                            // Submitted text display
                            .child(
                                if !submitted.is_empty() {
                                    container()
                                        .padding(12.0)
                                        .background(colors::GREEN_400.with_alpha(0.2))
                                        .border(colors::GREEN_500, 1.0)
                                        .corner_radius(4.0)
                                        .child(text(
                                            submitted,
                                            TextStyle {
                                                color: colors::GREEN_600,
                                                size: 14.0,
                                                ..Default::default()
                                            },
                                        ))
                                } else {
                                    container()
                                },
                            )
                            // Tips
                            .child(text(
                                "Tips: Arrow keys move cursor | Home/End jump to start/end",
                                TextStyle {
                                    color: colors::GRAY_500,
                                    size: 12.0,
                                    ..Default::default()
                                },
                            )),
                    )
                },
            );
        })
        .run();
}
