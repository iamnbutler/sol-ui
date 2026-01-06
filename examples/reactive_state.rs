//! Reactive State Demo
//!
//! Demonstrates the entity subscription system for automatic UI updates:
//! - Using `observe()` to read state and subscribe to changes
//! - Using `update_entity()` to modify state (triggers automatic re-render)
//! - Using `derive()` for computed values
//! - Batched updates within a single frame
//!
//! This example shows a counter app where clicking buttons updates entity
//! state, and the UI automatically re-renders without manual invalidation.

use sol_ui::{
    app::app,
    color::colors,
    element::{button, container, row, text, Element},
    entity::{new_entity, observe, update_entity, Entity},
    layer::LayerOptions,
    style::TextStyle,
};

/// State for a counter
struct CounterState {
    value: i32,
}

/// State for tracking total clicks
struct ClickTracker {
    total_clicks: u32,
}

fn main() {
    app()
        .title("Reactive State Demo")
        .size(500.0, 400.0)
        .with_layers(|layers| {
            // Create counter entity (will be cloned into the render closure)
            let counter: Option<Entity<CounterState>> = None;
            let counter = std::cell::RefCell::new(counter);

            // Create click tracker entity
            let tracker: Option<Entity<ClickTracker>> = None;
            let tracker = std::cell::RefCell::new(tracker);

            layers.add_ui_layer(
                0,
                LayerOptions::default().with_input().with_clear(),
                move || {
                    // Initialize entities on first render
                    let counter_entity = {
                        let mut c = counter.borrow_mut();
                        if c.is_none() {
                            *c = Some(new_entity(CounterState { value: 0 }));
                        }
                        c.clone().unwrap()
                    };

                    let tracker_entity = {
                        let mut t = tracker.borrow_mut();
                        if t.is_none() {
                            *t = Some(new_entity(ClickTracker { total_clicks: 0 }));
                        }
                        t.clone().unwrap()
                    };

                    // Use observe() to read state - this registers for reactive updates
                    // When update_entity() is called on an observed entity, the UI
                    // automatically re-renders
                    let count = observe(&counter_entity, |s| s.value).unwrap_or(0);
                    let total = observe(&tracker_entity, |s| s.total_clicks).unwrap_or(0);

                    // Derived/computed value
                    let status_text = if count == 0 {
                        "Counter is at zero".to_string()
                    } else if count > 0 {
                        format!("Counter is positive: +{}", count)
                    } else {
                        format!("Counter is negative: {}", count)
                    };

                    // Clone entities for button handlers
                    let counter_inc = counter_entity.clone();
                    let tracker_inc = tracker_entity.clone();
                    let counter_dec = counter_entity.clone();
                    let tracker_dec = tracker_entity.clone();
                    let counter_reset = counter_entity.clone();
                    let tracker_reset = tracker_entity.clone();
                    let counter_add10 = counter_entity.clone();
                    let tracker_add10 = tracker_entity.clone();

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
                                "Reactive State Demo",
                                TextStyle {
                                    color: colors::BLACK,
                                    size: 28.0,
                                    ..Default::default()
                                },
                            ))
                            // Explanation
                            .child(text(
                                "Uses observe() + update_entity() for automatic re-renders",
                                TextStyle {
                                    color: colors::GRAY_600,
                                    size: 14.0,
                                    ..Default::default()
                                },
                            ))
                            // Counter display
                            .child(
                                container()
                                    .background(colors::WHITE)
                                    .padding(32.0)
                                    .corner_radius(12.0)
                                    .child(text(
                                        count.to_string(),
                                        TextStyle {
                                            color: if count >= 0 {
                                                colors::BLUE_600
                                            } else {
                                                colors::RED_600
                                            },
                                            size: 64.0,
                                            ..Default::default()
                                        },
                                    )),
                            )
                            // Status text (derived value)
                            .child(text(
                                status_text,
                                TextStyle {
                                    color: colors::GRAY_700,
                                    size: 16.0,
                                    ..Default::default()
                                },
                            ))
                            // Button row
                            .child(
                                row()
                                    .gap(12.0)
                                    .child(
                                        button("-")
                                            .padding_xy(16.0, 12.0)
                                            .corner_radius(8.0)
                                            .backgrounds(
                                                colors::RED_500,
                                                colors::RED_400,
                                                colors::RED_600,
                                            )
                                            .text_size(20.0)
                                            .on_click_simple(move || {
                                                // update_entity marks entity dirty
                                                // Since we observe() this entity, UI re-renders
                                                update_entity(&counter_dec, |s| s.value -= 1);
                                                update_entity(&tracker_dec, |s| s.total_clicks += 1);
                                            }),
                                    )
                                    .child(
                                        button("Reset")
                                            .padding_xy(16.0, 12.0)
                                            .corner_radius(8.0)
                                            .backgrounds(
                                                colors::GRAY_500,
                                                colors::GRAY_400,
                                                colors::GRAY_600,
                                            )
                                            .text_size(16.0)
                                            .on_click_simple(move || {
                                                update_entity(&counter_reset, |s| s.value = 0);
                                                update_entity(&tracker_reset, |s| s.total_clicks += 1);
                                            }),
                                    )
                                    .child(
                                        button("+")
                                            .padding_xy(16.0, 12.0)
                                            .corner_radius(8.0)
                                            .backgrounds(
                                                colors::GREEN_500,
                                                colors::GREEN_400,
                                                colors::GREEN_600,
                                            )
                                            .text_size(20.0)
                                            .on_click_simple(move || {
                                                update_entity(&counter_inc, |s| s.value += 1);
                                                update_entity(&tracker_inc, |s| s.total_clicks += 1);
                                            }),
                                    ),
                            )
                            // Add 10 button (demonstrates batched updates)
                            .child(
                                button("+10 (batched)")
                                    .padding_xy(12.0, 8.0)
                                    .corner_radius(6.0)
                                    .backgrounds(
                                        colors::PURPLE_500,
                                        colors::PURPLE_400,
                                        colors::PURPLE_600,
                                    )
                                    .text_size(14.0)
                                    .on_click_simple(move || {
                                        // Multiple updates in same handler are batched
                                        // Only one re-render happens at frame end
                                        for _ in 0..10 {
                                            update_entity(&counter_add10, |s| s.value += 1);
                                        }
                                        update_entity(&tracker_add10, |s| s.total_clicks += 1);
                                    }),
                            )
                            // Total clicks display
                            .child(text(
                                format!("Total button clicks: {}", total),
                                TextStyle {
                                    color: colors::GRAY_500,
                                    size: 12.0,
                                    ..Default::default()
                                },
                            )),
                    ) as Box<dyn Element>
                },
            );
        })
        .run();
}
