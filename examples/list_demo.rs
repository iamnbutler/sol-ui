//! List element demo
//!
//! Demonstrates the List element with various configurations:
//! - Basic list with items
//! - Single and multi-selection modes
//! - Hover-reveal action buttons
//! - Custom styling
//! - Empty state

use sol_ui::{
    app::app,
    color::{colors, ColorExt},
    element::{column, container, list, row, text, ListAction, ListItemData, SelectionMode},
    layer::LayerOptions,
    style::TextStyle,
};
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    // Sample data for the lists
    let contacts: Rc<RefCell<Vec<(&str, &str)>>> = Rc::new(RefCell::new(vec![
        ("Alice Johnson", "alice@example.com"),
        ("Bob Smith", "bob@example.com"),
        ("Carol Williams", "carol@example.com"),
        ("David Brown", "david@example.com"),
        ("Eve Davis", "eve@example.com"),
    ]));

    let tasks: Rc<RefCell<Vec<(&str, &str)>>> = Rc::new(RefCell::new(vec![
        ("Review pull requests", "3 pending"),
        ("Update documentation", "In progress"),
        ("Fix login bug", "High priority"),
        ("Deploy to staging", "Blocked"),
        ("Write unit tests", "Low priority"),
    ]));

    // Track deleted items for demo
    let deleted_count = Rc::new(RefCell::new(0usize));

    app()
        .title("List Element Demo")
        .size(900.0, 700.0)
        .with_layers(move |layers| {
            let contacts_clone = contacts.clone();
            let tasks_clone = tasks.clone();
            let deleted_count_clone = deleted_count.clone();

            layers.add_ui_layer(
                0,
                LayerOptions::default().with_input().with_clear(),
                move || {
                    let contacts_data: Vec<ListItemData> = contacts_clone
                        .borrow()
                        .iter()
                        .map(|(name, email)| ListItemData::new(*name).subtitle(*email))
                        .collect();

                    let tasks_data: Vec<ListItemData> = tasks_clone
                        .borrow()
                        .iter()
                        .map(|(title, status)| ListItemData::new(*title).subtitle(*status))
                        .collect();

                    let deleted = *deleted_count_clone.borrow();

                    // Callbacks for actions
                    let contacts_for_delete = contacts_clone.clone();
                    let deleted_for_update = deleted_count_clone.clone();

                    let tasks_for_delete = tasks_clone.clone();
                    let deleted_for_tasks = deleted_count_clone.clone();

                    Box::new(
                        container()
                            .width_full()
                            .height_full()
                            .background(colors::GRAY_100)
                            .flex_col()
                            .padding(32.0)
                            .gap(24.0)
                            // Title
                            .child(
                                text(
                                    "List Element Demo",
                                    TextStyle {
                                        color: colors::GRAY_900,
                                        size: 28.0,
                                        ..Default::default()
                                    },
                                )
                            )
                            // Status bar
                            .child(
                                container()
                                    .background(colors::BLUE_400.with_alpha(0.2))
                                    .padding(12.0)
                                    .corner_radius(8.0)
                                    .child(
                                        text(
                                            format!("Items deleted this session: {}", deleted),
                                            TextStyle {
                                                color: colors::BLUE_600,
                                                size: 14.0,
                                                ..Default::default()
                                            },
                                        )
                                    )
                            )
                            // Two-column layout
                            .child(
                                row()
                                    .gap(24.0)
                                    // Left column: Contacts with single selection
                                    .child(
                                        column()
                                            .gap(12.0)
                                            .child(
                                                text(
                                                    "Contacts (Single Select)",
                                                    TextStyle {
                                                        color: colors::GRAY_700,
                                                        size: 18.0,
                                                        ..Default::default()
                                                    },
                                                )
                                            )
                                            .child(
                                                container()
                                                    .background(colors::WHITE)
                                                    .corner_radius(12.0)
                                                    .border(colors::GRAY_200, 1.0)
                                                    .width(380.0)
                                                    .height(300.0)
                                                    .child(
                                                        list(contacts_data)
                                                            .selection_mode(SelectionMode::Single)
                                                            .item_height(56.0)
                                                            .gap(1.0)
                                                            .selected_background(colors::BLUE_400.with_alpha(0.2))
                                                            .action(ListAction::new(
                                                                "Call",
                                                                colors::GREEN_500,
                                                                |idx| println!("Calling contact {}", idx),
                                                            ))
                                                            .action(ListAction::delete(move |idx| {
                                                                println!("Deleting contact {}", idx);
                                                                let mut contacts = contacts_for_delete.borrow_mut();
                                                                if idx < contacts.len() {
                                                                    contacts.remove(idx);
                                                                    *deleted_for_update.borrow_mut() += 1;
                                                                }
                                                            }))
                                                            .on_item_click(|idx| {
                                                                println!("Clicked contact {}", idx);
                                                            })
                                                            .on_selection_change(|selected| {
                                                                println!("Contact selection: {:?}", selected);
                                                            })
                                                    )
                                            )
                                    )
                                    // Right column: Tasks with multi selection
                                    .child(
                                        column()
                                            .gap(12.0)
                                            .child(
                                                text(
                                                    "Tasks (Multi Select)",
                                                    TextStyle {
                                                        color: colors::GRAY_700,
                                                        size: 18.0,
                                                        ..Default::default()
                                                    },
                                                )
                                            )
                                            .child(
                                                container()
                                                    .background(colors::WHITE)
                                                    .corner_radius(12.0)
                                                    .border(colors::GRAY_200, 1.0)
                                                    .width(380.0)
                                                    .height(300.0)
                                                    .child(
                                                        list(tasks_data)
                                                            .selection_mode(SelectionMode::Multi)
                                                            .item_height(56.0)
                                                            .gap(1.0)
                                                            .selected_background(colors::PURPLE_400.with_alpha(0.2))
                                                            .action(ListAction::edit(|idx| {
                                                                println!("Editing task {}", idx);
                                                            }))
                                                            .action(ListAction::delete(move |idx| {
                                                                println!("Deleting task {}", idx);
                                                                let mut tasks = tasks_for_delete.borrow_mut();
                                                                if idx < tasks.len() {
                                                                    tasks.remove(idx);
                                                                    *deleted_for_tasks.borrow_mut() += 1;
                                                                }
                                                            }))
                                                            .on_selection_change(|selected| {
                                                                println!("Task selection: {:?}", selected);
                                                            })
                                                    )
                                            )
                                    )
                            )
                            // Bottom row: Empty state demo
                            .child(
                                column()
                                    .gap(12.0)
                                    .child(
                                        text(
                                            "Empty State Demo",
                                            TextStyle {
                                                color: colors::GRAY_700,
                                                size: 18.0,
                                                ..Default::default()
                                            },
                                        )
                                    )
                                    .child(
                                        container()
                                            .background(colors::WHITE)
                                            .corner_radius(12.0)
                                            .border(colors::GRAY_200, 1.0)
                                            .height(100.0)
                                            .width_full()
                                            .child(
                                                list::<ListItemData>(vec![])
                                                    .empty_state(
                                                        container()
                                                            .width_full()
                                                            .height_full()
                                                            .justify_center()
                                                            .items_center()
                                                            .child(
                                                                text(
                                                                    "No items yet. Add some to get started!",
                                                                    TextStyle {
                                                                        color: colors::GRAY_400,
                                                                        size: 14.0,
                                                                        ..Default::default()
                                                                    },
                                                                )
                                                            )
                                                    )
                                            )
                                    )
                            )
                    )
                },
            );
        })
        .run();
}
