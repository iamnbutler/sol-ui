//! Todo App Example
//!
//! A complete todo application demonstrating sol-ui capabilities:
//! - Text input for adding new todos
//! - Checkboxes for marking complete/incomplete
//! - Delete buttons for removing todos
//! - Filter tabs (All/Active/Completed)
//! - Scroll container for long lists
//! - Entity-based state persistence across frames

use sol_ui::{
    app::app,
    color::{ColorExt, colors},
    element::{
        checkbox, column, container, row, scroll, text, text_input,
        CheckboxInteractable, TextInputInteractable, TextInputState,
    },
    entity::{new_entity, StateCell},
    interaction::Interactable,
    layer::{LayerOptions, MouseButton},
    style::TextStyle,
};

/// A single todo item
#[derive(Debug, Clone)]
struct TodoItem {
    id: u64,
    text: String,
    completed: bool,
}

/// Filter mode for displaying todos
#[derive(Debug, Clone, Copy, PartialEq, Default)]
enum FilterMode {
    #[default]
    All,
    Active,
    Completed,
}

/// Application state
#[derive(Debug, Clone)]
struct TodoAppState {
    todos: Vec<TodoItem>,
    next_id: u64,
    filter: FilterMode,
}

impl Default for TodoAppState {
    fn default() -> Self {
        Self {
            todos: vec![
                TodoItem {
                    id: 1,
                    text: "Learn sol-ui".to_string(),
                    completed: true,
                },
                TodoItem {
                    id: 2,
                    text: "Build a todo app".to_string(),
                    completed: false,
                },
                TodoItem {
                    id: 3,
                    text: "Ship it!".to_string(),
                    completed: false,
                },
            ],
            next_id: 4,
            filter: FilterMode::All,
        }
    }
}

impl TodoAppState {
    fn add_todo(&mut self, text: String) {
        if text.trim().is_empty() {
            return;
        }
        self.todos.push(TodoItem {
            id: self.next_id,
            text: text.trim().to_string(),
            completed: false,
        });
        self.next_id += 1;
    }

    fn toggle_todo(&mut self, id: u64) {
        if let Some(todo) = self.todos.iter_mut().find(|t| t.id == id) {
            todo.completed = !todo.completed;
        }
    }

    fn delete_todo(&mut self, id: u64) {
        self.todos.retain(|t| t.id != id);
    }

    fn set_filter(&mut self, filter: FilterMode) {
        self.filter = filter;
    }

    fn filtered_todos(&self) -> Vec<&TodoItem> {
        self.todos
            .iter()
            .filter(|t| match self.filter {
                FilterMode::All => true,
                FilterMode::Active => !t.completed,
                FilterMode::Completed => t.completed,
            })
            .collect()
    }

    fn active_count(&self) -> usize {
        self.todos.iter().filter(|t| !t.completed).count()
    }

    fn completed_count(&self) -> usize {
        self.todos.iter().filter(|t| t.completed).count()
    }

    fn clear_completed(&mut self) {
        self.todos.retain(|t| !t.completed);
    }
}

fn main() {
    // Shared app state - StateCell handles lazy initialization
    let app_state = StateCell::new();

    app()
        .title("Todo App - sol-ui")
        .size(500.0, 600.0)
        .with_layers(move |layers| {
            layers.add_ui_layer(
                0,
                LayerOptions::default().with_input().with_clear(),
                move || {
                    // Initialize entity on first frame (StateCell handles this)
                    let state_entity = app_state.get_or_init(TodoAppState::default);

                    // Read current state using method syntax
                    let (todos, filter, active_count, completed_count) =
                        state_entity.read(|s| {
                            (
                                s.filtered_todos()
                                    .into_iter()
                                    .cloned()
                                    .collect::<Vec<_>>(),
                                s.filter,
                                s.active_count(),
                                s.completed_count(),
                            )
                        })
                        .unwrap_or_default();

                    // Create input state for new todo
                    let input_state = new_entity(TextInputState::default());

                    // Clone state entity for callbacks
                    let state_for_add = state_entity.clone();
                    let input_for_add = input_state.clone();

                    Box::new(
                        container()
                            .width_full()
                            .height_full()
                            .background(colors::GRAY_100)
                            .flex_col()
                            .padding(24.0)
                            .gap(16.0)
                            // Header
                            .child(
                                container()
                                    .flex()
                                    .justify_center()
                                    .child(text(
                                        "todos",
                                        TextStyle {
                                            color: colors::RED_400.with_alpha(0.3),
                                            size: 64.0,
                                            line_height: 1.2,
                                        },
                                    )),
                            )
                            // Input area
                            .child(
                                container()
                                    .width_full()
                                    .background(colors::WHITE)
                                    .border(colors::GRAY_200, 1.0)
                                    .corner_radius(4.0)
                                    .padding(8.0)
                                    .child(
                                        row()
                                            .width_full()
                                            .items_center()
                                            .gap(8.0)
                                            .child(
                                                text_input(input_state.clone())
                                                    .width(380.0)
                                                    .height(40.0)
                                                    .placeholder("What needs to be done?")
                                                    .text_size(18.0)
                                                    .on_submit({
                                                        let state = state_for_add.clone();
                                                        let input = input_for_add.clone();
                                                        move |text| {
                                                            if !text.trim().is_empty() {
                                                                state.update(|s| {
                                                                    s.add_todo(text.to_string());
                                                                });
                                                                // Clear the input
                                                                input.update(|s| {
                                                                    s.text.clear();
                                                                    s.cursor = 0;
                                                                });
                                                            }
                                                        }
                                                    })
                                                    .interactive_input(),
                                            ),
                                    ),
                            )
                            // Todo list
                            .child(
                                scroll()
                                    .width_full()
                                    .height(300.0)
                                    .child({
                                        let mut list = column().width_full().gap(1.0);

                                        for todo in todos.iter() {
                                            let todo_id = todo.id;
                                            let todo_text = todo.text.clone();
                                            let todo_completed = todo.completed;
                                            let state_for_toggle = state_entity.clone();
                                            let state_for_delete = state_entity.clone();

                                            list = list.child(
                                                container()
                                                    .width_full()
                                                    .background(colors::WHITE)
                                                    .padding(12.0)
                                                    .child(
                                                        row()
                                                            .width_full()
                                                            .items_center()
                                                            .gap(12.0)
                                                            // Checkbox
                                                            .child(
                                                                checkbox(todo_completed)
                                                                    .size(24.0)
                                                                    .on_change({
                                                                        let state = state_for_toggle.clone();
                                                                        move |_| {
                                                                            state.update(|s| {
                                                                                s.toggle_todo(todo_id);
                                                                            });
                                                                        }
                                                                    })
                                                                    .interactive_checkbox(),
                                                            )
                                                            // Todo text
                                                            .child(
                                                                container()
                                                                    .width(320.0)
                                                                    .child(text(
                                                                        todo_text,
                                                                        TextStyle {
                                                                            color: if todo_completed {
                                                                                colors::GRAY_400
                                                                            } else {
                                                                                colors::BLACK
                                                                            },
                                                                            size: 16.0,
                                                                            line_height: 1.2,
                                                                        },
                                                                    )),
                                                            )
                                                            // Delete button
                                                            .child(
                                                                container()
                                                                    .width(24.0)
                                                                    .height(24.0)
                                                                    .flex()
                                                                    .items_center()
                                                                    .justify_center()
                                                                    .child(text(
                                                                        "×",
                                                                        TextStyle {
                                                                            color: colors::RED_500,
                                                                            size: 20.0,
                                                                            line_height: 1.2,
                                                                        },
                                                                    ))
                                                                    .interactive()
                                                                    .with_id(1000 + todo_id as i32)
                                                                    .hover_overlay(colors::RED_500.with_alpha(0.1))
                                                                    .on_click(move |btn, _, _, _, _| {
                                                                        if btn == MouseButton::Left {
                                                                            state_for_delete.update(|s| {
                                                                                s.delete_todo(todo_id);
                                                                            });
                                                                        }
                                                                    }),
                                                            ),
                                                    ),
                                            );
                                        }

                                        if todos.is_empty() {
                                            list = list.child(
                                                container()
                                                    .width_full()
                                                    .padding(24.0)
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .child(text(
                                                        match filter {
                                                            FilterMode::All => "No todos yet. Add one above!",
                                                            FilterMode::Active => "No active todos",
                                                            FilterMode::Completed => "No completed todos",
                                                        },
                                                        TextStyle {
                                                            color: colors::GRAY_400,
                                                            size: 16.0,
                                                            line_height: 1.2,
                                                        },
                                                    )),
                                            );
                                        }

                                        list
                                    }),
                            )
                            // Footer with filters
                            .child(
                                container()
                                    .width_full()
                                    .background(colors::WHITE)
                                    .border(colors::GRAY_200, 1.0)
                                    .corner_radius(4.0)
                                    .padding(12.0)
                                    .child(
                                        row()
                                            .width_full()
                                            .items_center()
                                            .gap(16.0)
                                            // Item count
                                            .child(text(
                                                format!(
                                                    "{} item{} left",
                                                    active_count,
                                                    if active_count == 1 { "" } else { "s" }
                                                ),
                                                TextStyle {
                                                    color: colors::GRAY_500,
                                                    size: 14.0,
                                                    line_height: 1.2,
                                                },
                                            ))
                                            // Filter buttons
                                            .child(
                                                row()
                                                    .gap(4.0)
                                                    .child(
                                                        filter_button("All", filter == FilterMode::All, {
                                                            let state = state_entity.clone();
                                                            move || {
                                                                state.update(|s| {
                                                                    s.set_filter(FilterMode::All);
                                                                });
                                                            }
                                                        }),
                                                    )
                                                    .child(
                                                        filter_button("Active", filter == FilterMode::Active, {
                                                            let state = state_entity.clone();
                                                            move || {
                                                                state.update(|s| {
                                                                    s.set_filter(FilterMode::Active);
                                                                });
                                                            }
                                                        }),
                                                    )
                                                    .child(
                                                        filter_button("Completed", filter == FilterMode::Completed, {
                                                            let state = state_entity.clone();
                                                            move || {
                                                                state.update(|s| {
                                                                    s.set_filter(FilterMode::Completed);
                                                                });
                                                            }
                                                        }),
                                                    ),
                                            )
                                            // Clear completed (only show if there are completed items)
                                            .child(
                                                if completed_count > 0 {
                                                    container().child(
                                                        container()
                                                            .padding(8.0)
                                                            .child(text(
                                                                "Clear completed",
                                                                TextStyle {
                                                                    color: colors::GRAY_500,
                                                                    size: 14.0,
                                                                    line_height: 1.2,
                                                                },
                                                            ))
                                                            .interactive()
                                                            .with_id(9999)
                                                            .hover_overlay(colors::BLACK.with_alpha(0.05))
                                                            .on_click({
                                                                let state = state_entity.clone();
                                                                move |btn, _, _, _, _| {
                                                                    if btn == MouseButton::Left {
                                                                        state.update(|s| {
                                                                            s.clear_completed();
                                                                        });
                                                                    }
                                                                }
                                                            }),
                                                    )
                                                } else {
                                                    container()
                                                },
                                            ),
                                    ),
                            ),
                    )
                },
            );
        })
        .run();
}

/// Create a filter button
fn filter_button(
    label: &str,
    selected: bool,
    on_click: impl FnMut() + 'static,
) -> impl sol_ui::element::Element {
    let label = label.to_string();
    let mut on_click = on_click;

    container()
        .padding(8.0)
        .border(
            if selected {
                colors::GRAY_400
            } else {
                colors::TRANSPARENT
            },
            1.0,
        )
        .corner_radius(3.0)
        .child(text(
            label.clone(),
            TextStyle {
                color: if selected {
                    colors::BLACK
                } else {
                    colors::GRAY_500
                },
                size: 14.0,
                line_height: 1.2,
            },
        ))
        .interactive()
        .hover_overlay(colors::BLACK.with_alpha(0.05))
        .on_click(move |btn, _, _, _, _| {
            if btn == MouseButton::Left {
                on_click();
            }
        })
}
