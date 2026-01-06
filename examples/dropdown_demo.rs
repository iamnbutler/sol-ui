//! Dropdown element demo
//!
//! Run with: cargo run --example dropdown_demo

use palette::Srgba;
use sol_ui::{
    app::app,
    color::colors,
    element::{column, container, dropdown, row, text, DropdownOption},
    layer::{LayerManager, LayerOptions},
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

    // State for displaying selection
    let selected_fruit = Rc::new(RefCell::new(String::from("(none)")));
    let selected_country = Rc::new(RefCell::new(String::from("(none)")));

    app()
        .title("Dropdown Demo")
        .size(600.0, 500.0)
        .with_layers(move |layer_manager: &mut LayerManager| {
            let selected_fruit_display = selected_fruit.clone();
            let selected_country_display = selected_country.clone();

            layer_manager.add_ui_layer(0, LayerOptions::default(), move || {
                let selected_fruit_inner = selected_fruit_display.clone();
                let selected_country_inner = selected_country_display.clone();

                Box::new(
                    container()
                        .width_full()
                        .height_full()
                        .padding(40.0)
                        .background(Srgba::new(0.97, 0.97, 0.97, 1.0))
                        .child(
                            column()
                                .gap(30.0)
                                // Title
                                .child(text(
                                    "Dropdown Element Demo",
                                    TextStyle {
                                        size: 28.0,
                                        color: colors::BLACK,
                                    },
                                ))
                                // Basic dropdown
                                .child(
                                    column()
                                        .gap(8.0)
                                        .child(text(
                                            "Basic Dropdown",
                                            TextStyle {
                                                size: 16.0,
                                                color: colors::GRAY_700,
                                            },
                                        ))
                                        .child(
                                            row()
                                                .gap(20.0)
                                                .child(
                                                    dropdown(vec![
                                                        "Apple",
                                                        "Banana",
                                                        "Cherry",
                                                        "Date",
                                                        "Elderberry",
                                                        "Fig",
                                                        "Grape",
                                                    ])
                                                    .placeholder("Select a fruit...")
                                                    .width(200.0)
                                                    .on_change({
                                                        let selected = selected_fruit_inner.clone();
                                                        move |_idx, value: &&str| {
                                                            *selected.borrow_mut() = value.to_string();
                                                        }
                                                    }),
                                                )
                                                .child(text(
                                                    format!("Selected: {}", selected_fruit_inner.borrow()),
                                                    TextStyle {
                                                        size: 14.0,
                                                        color: colors::GRAY_600,
                                                    },
                                                )),
                                        ),
                                )
                                // Dropdown with disabled options
                                .child(
                                    column()
                                        .gap(8.0)
                                        .child(text(
                                            "With Disabled Options",
                                            TextStyle {
                                                size: 16.0,
                                                color: colors::GRAY_700,
                                            },
                                        ))
                                        .child(
                                            dropdown(vec![
                                                "Small",
                                                "Medium",
                                                "Large",
                                                "Extra Large",
                                            ])
                                            .placeholder("Select size...")
                                            .width(200.0)
                                            .selected(1), // Pre-select "Medium"
                                        ),
                                )
                                // Dropdown with rich options
                                .child(
                                    column()
                                        .gap(8.0)
                                        .child(text(
                                            "With Labels",
                                            TextStyle {
                                                size: 16.0,
                                                color: colors::GRAY_700,
                                            },
                                        ))
                                        .child(
                                            row()
                                                .gap(20.0)
                                                .child({
                                                    let options = vec![
                                                        DropdownOption::new("us").label("United States"),
                                                        DropdownOption::new("uk").label("United Kingdom"),
                                                        DropdownOption::new("ca").label("Canada"),
                                                        DropdownOption::new("au").label("Australia"),
                                                        DropdownOption::new("de").label("Germany"),
                                                        DropdownOption::new("fr").label("France"),
                                                        DropdownOption::new("jp").label("Japan"),
                                                    ];
                                                    sol_ui::element::Dropdown::with_options(options)
                                                        .placeholder("Select country...")
                                                        .width(200.0)
                                                        .on_change({
                                                            let selected = selected_country_inner.clone();
                                                            move |_idx, value: &&str| {
                                                                *selected.borrow_mut() = value.to_string();
                                                            }
                                                        })
                                                })
                                                .child(text(
                                                    format!("Code: {}", selected_country_inner.borrow()),
                                                    TextStyle {
                                                        size: 14.0,
                                                        color: colors::GRAY_600,
                                                    },
                                                )),
                                        ),
                                )
                                // Disabled dropdown
                                .child(
                                    column()
                                        .gap(8.0)
                                        .child(text(
                                            "Disabled Dropdown",
                                            TextStyle {
                                                size: 16.0,
                                                color: colors::GRAY_700,
                                            },
                                        ))
                                        .child(
                                            dropdown(vec!["Option 1", "Option 2", "Option 3"])
                                                .placeholder("Disabled...")
                                                .width(200.0)
                                                .disabled(true),
                                        ),
                                )
                                // Usage instructions
                                .child(
                                    container()
                                        .background(Srgba::new(0.9, 0.95, 1.0, 1.0))
                                        .padding(16.0)
                                        .corner_radius(8.0)
                                        .child(
                                            column()
                                                .gap(4.0)
                                                .child(text(
                                                    "Usage:",
                                                    TextStyle {
                                                        size: 14.0,
                                                        color: colors::BLUE_600,
                                                    },
                                                ))
                                                .child(text(
                                                    "• Click to open/close",
                                                    TextStyle {
                                                        size: 13.0,
                                                        color: colors::GRAY_600,
                                                    },
                                                ))
                                                .child(text(
                                                    "• Keyboard: Arrow keys, Enter, Escape (when focused)",
                                                    TextStyle {
                                                        size: 13.0,
                                                        color: colors::GRAY_600,
                                                    },
                                                ))
                                                .child(text(
                                                    "• Type to search (when open and focused)",
                                                    TextStyle {
                                                        size: 13.0,
                                                        color: colors::GRAY_600,
                                                    },
                                                )),
                                        ),
                                ),
                        ),
                )
            });
        })
        .run();
}
