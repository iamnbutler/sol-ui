//! Async Tasks Example
//!
//! This example demonstrates the non-blocking async task system.
//! It shows how to spawn background tasks that:
//! - Run expensive computations off the main thread
//! - Deliver results back to the UI thread safely
//! - Keep the render loop responsive

use sol_ui::{
    app::app,
    color::{ColorExt, colors},
    element::{column, container, row, text},
    interaction::Interactable,
    layer::{LayerOptions, MouseButton},
    style::TextStyle,
    task::spawn_task,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

fn main() {
    // Shared state for the UI
    let is_running = Rc::new(RefCell::new(false));
    let task_count = Rc::new(RefCell::new(0));
    let completed_count = Rc::new(RefCell::new(0));
    let last_result = Rc::new(RefCell::new(String::from("None yet")));

    app()
        .title("Async Tasks Example")
        .size(800.0, 600.0)
        .with_layers(move |layers| {
            let is_running_clone = is_running.clone();
            let task_count_clone = task_count.clone();
            let completed_count_clone = completed_count.clone();
            let last_result_clone = last_result.clone();

            layers.add_ui_layer(
                0,
                LayerOptions::default().with_input().with_clear(),
                move || {
                    let running = *is_running_clone.borrow();
                    let tasks = *task_count_clone.borrow();
                    let completed = *completed_count_clone.borrow();
                    let last = last_result_clone.borrow().clone();

                    // Clones for callbacks
                    let is_running_long = is_running_clone.clone();
                    let task_count_long = task_count_clone.clone();
                    let completed_count_long = completed_count_clone.clone();
                    let last_result_long = last_result_clone.clone();

                    let is_running_quick = is_running_clone.clone();
                    let task_count_quick = task_count_clone.clone();
                    let completed_count_quick = completed_count_clone.clone();
                    let last_result_quick = last_result_clone.clone();

                    let is_running_many = is_running_clone.clone();
                    let task_count_many = task_count_clone.clone();
                    let completed_count_many = completed_count_clone.clone();

                    // Status indicator color
                    let status_color = if running {
                        colors::BLUE_500
                    } else {
                        colors::GRAY_500
                    };

                    let status_text = if running {
                        format!("Running... ({} pending)", tasks - completed)
                    } else {
                        "Idle".to_string()
                    };

                    Box::new(
                        container()
                            .width_full()
                            .height_full()
                            .background(colors::GRAY_100)
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .gap(20.0)
                            // Title
                            .child(
                                text(
                                    "Async Task Demo",
                                    TextStyle {
                                        color: colors::BLACK,
                                        size: 32.0,
                                    line_height: 1.2,
                                    },
                                )
                            )
                            // Description
                            .child(
                                text(
                                    "Background tasks run without blocking the UI",
                                    TextStyle {
                                        color: colors::GRAY_600,
                                        size: 16.0,
                                    line_height: 1.2,
                                    },
                                )
                            )
                            // Stats row
                            .child(
                                row()
                                    .gap(40.0)
                                    .child(
                                        column()
                                            .items_center()
                                            .child(
                                                text(
                                                    format!("{}", tasks),
                                                    TextStyle {
                                                        color: colors::BLUE_600,
                                                        size: 36.0,
                                    line_height: 1.2,
                                                    },
                                                )
                                            )
                                            .child(
                                                text(
                                                    "Spawned",
                                                    TextStyle {
                                                        color: colors::GRAY_500,
                                                        size: 14.0,
                                    line_height: 1.2,
                                                    },
                                                )
                                            )
                                    )
                                    .child(
                                        column()
                                            .items_center()
                                            .child(
                                                text(
                                                    format!("{}", completed),
                                                    TextStyle {
                                                        color: colors::GREEN_600,
                                                        size: 36.0,
                                    line_height: 1.2,
                                                    },
                                                )
                                            )
                                            .child(
                                                text(
                                                    "Completed",
                                                    TextStyle {
                                                        color: colors::GRAY_500,
                                                        size: 14.0,
                                    line_height: 1.2,
                                                    },
                                                )
                                            )
                                    )
                            )
                            // Status indicator
                            .child(
                                container()
                                    .width(300.0)
                                    .height(50.0)
                                    .background(status_color.with_alpha(0.1))
                                    .corner_radius(8.0)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(
                                        text(
                                            status_text,
                                            TextStyle {
                                                color: status_color,
                                                size: 16.0,
                                    line_height: 1.2,
                                            },
                                        )
                                    )
                            )
                            // Last result
                            .child(
                                text(
                                    format!("Last result: {}", last),
                                    TextStyle {
                                        color: colors::GRAY_700,
                                        size: 14.0,
                                    line_height: 1.2,
                                    },
                                )
                            )
                            // Buttons
                            .child(
                                row()
                                    .gap(10.0)
                                    // Long task button (2 seconds)
                                    .child(
                                        container()
                                            .width(150.0)
                                            .height(50.0)
                                            .background(colors::BLUE_500)
                                            .corner_radius(8.0)
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(
                                                text(
                                                    "Long Task (2s)",
                                                    TextStyle {
                                                        color: colors::WHITE,
                                                        size: 14.0,
                                    line_height: 1.2,
                                                    },
                                                )
                                            )
                                            .interactive()
                                            .with_id(1)
                                            .hover_overlay(colors::BLACK.with_alpha(0.1))
                                            .press_overlay(colors::BLACK.with_alpha(0.2))
                                            .on_click({
                                                move |button, _, _| {
                                                    if button == MouseButton::Left {
                                                        *is_running_long.borrow_mut() = true;
                                                        *task_count_long.borrow_mut() += 1;
                                                        let task_num = *task_count_long.borrow();

                                                        let is_running_cb = is_running_long.clone();
                                                        let completed_cb = completed_count_long.clone();
                                                        let last_result_cb = last_result_long.clone();
                                                        let task_count_cb = task_count_long.clone();

                                                        spawn_task(
                                                            move || {
                                                                println!("[BG] Task {} started (2s)", task_num);
                                                                std::thread::sleep(Duration::from_secs(2));
                                                                println!("[BG] Task {} completed", task_num);
                                                                format!("Long #{} done!", task_num)
                                                            },
                                                            move |result| {
                                                                println!("[UI] Received: {}", result);
                                                                *last_result_cb.borrow_mut() = result;
                                                                *completed_cb.borrow_mut() += 1;
                                                                let pending = *task_count_cb.borrow() - *completed_cb.borrow();
                                                                if pending == 0 {
                                                                    *is_running_cb.borrow_mut() = false;
                                                                }
                                                            },
                                                        );
                                                    }
                                                }
                                            })
                                    )
                                    // Quick task button (100ms)
                                    .child(
                                        container()
                                            .width(150.0)
                                            .height(50.0)
                                            .background(colors::GREEN_500)
                                            .corner_radius(8.0)
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(
                                                text(
                                                    "Quick (100ms)",
                                                    TextStyle {
                                                        color: colors::WHITE,
                                                        size: 14.0,
                                    line_height: 1.2,
                                                    },
                                                )
                                            )
                                            .interactive()
                                            .with_id(2)
                                            .hover_overlay(colors::BLACK.with_alpha(0.1))
                                            .press_overlay(colors::BLACK.with_alpha(0.2))
                                            .on_click({
                                                move |button, _, _| {
                                                    if button == MouseButton::Left {
                                                        *is_running_quick.borrow_mut() = true;
                                                        *task_count_quick.borrow_mut() += 1;
                                                        let task_num = *task_count_quick.borrow();

                                                        let is_running_cb = is_running_quick.clone();
                                                        let completed_cb = completed_count_quick.clone();
                                                        let last_result_cb = last_result_quick.clone();
                                                        let task_count_cb = task_count_quick.clone();

                                                        spawn_task(
                                                            move || {
                                                                println!("[BG] Quick task {} started", task_num);
                                                                std::thread::sleep(Duration::from_millis(100));
                                                                format!("Quick #{}", task_num)
                                                            },
                                                            move |result| {
                                                                println!("[UI] Received: {}", result);
                                                                *last_result_cb.borrow_mut() = result;
                                                                *completed_cb.borrow_mut() += 1;
                                                                let pending = *task_count_cb.borrow() - *completed_cb.borrow();
                                                                if pending == 0 {
                                                                    *is_running_cb.borrow_mut() = false;
                                                                }
                                                            },
                                                        );
                                                    }
                                                }
                                            })
                                    )
                                    // Spawn many tasks button
                                    .child(
                                        container()
                                            .width(150.0)
                                            .height(50.0)
                                            .background(colors::PURPLE_500)
                                            .corner_radius(8.0)
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(
                                                text(
                                                    "Spawn 10 Tasks",
                                                    TextStyle {
                                                        color: colors::WHITE,
                                                        size: 14.0,
                                    line_height: 1.2,
                                                    },
                                                )
                                            )
                                            .interactive()
                                            .with_id(3)
                                            .hover_overlay(colors::BLACK.with_alpha(0.1))
                                            .press_overlay(colors::BLACK.with_alpha(0.2))
                                            .on_click({
                                                move |button, _, _| {
                                                    if button == MouseButton::Left {
                                                        *is_running_many.borrow_mut() = true;

                                                        for _ in 0..10 {
                                                            *task_count_many.borrow_mut() += 1;
                                                            let task_num = *task_count_many.borrow();

                                                            let is_running_cb = is_running_many.clone();
                                                            let completed_cb = completed_count_many.clone();
                                                            let task_count_cb = task_count_many.clone();

                                                            spawn_task(
                                                                move || {
                                                                    // Random-ish delay between 50-500ms
                                                                    let delay = 50 + (task_num * 37) % 450;
                                                                    std::thread::sleep(Duration::from_millis(delay as u64));
                                                                    task_num
                                                                },
                                                                move |result| {
                                                                    println!("[UI] Batch task {} done", result);
                                                                    *completed_cb.borrow_mut() += 1;
                                                                    let pending = *task_count_cb.borrow() - *completed_cb.borrow();
                                                                    if pending == 0 {
                                                                        *is_running_cb.borrow_mut() = false;
                                                                    }
                                                                },
                                                            );
                                                        }
                                                    }
                                                }
                                            })
                                    )
                            )
                            // Help text
                            .child(
                                text(
                                    "UI remains responsive - try clicking during long tasks!",
                                    TextStyle {
                                        color: colors::GRAY_500,
                                        size: 14.0,
                                    line_height: 1.2,
                                    },
                                )
                            )
                    )
                },
            );
        })
        .run();
}
