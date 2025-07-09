use palette::Srgba;
use toy_ui::{
    app,
    draw::TextStyle,
    layer::{LayerManager, LayerOptions},
    layout::{col, group, row, text},
};

fn main() {
    app()
        .title("Toy UI - Taffy Layout Demo")
        .size(800.0, 600.0)
        .with_layers(|layer_manager: &mut LayerManager| {

            // Layer 1: Basic text layout
            layer_manager.add_taffy_ui_layer(0, LayerOptions::default(), || {
                Box::new(
                    col()
                        .p(20.0)
                        .gap(10.0)
                        .child(text(
                            "Hello from Taffy!",
                            TextStyle {
                                size: 24.0,
                                color: Srgba::new(0.0, 0.0, 0.0, 1.0),
                            },
                        ))
                        .child(text(
                            "This is a column layout with padding and gap.",
                            TextStyle {
                                size: 16.0,
                                color: Srgba::new(0.3, 0.3, 0.3, 1.0),
                            },
                        ))
                        .child(
                            row()
                                .gap(15.0)
                                .child(text(
                                    "Row item 1",
                                    TextStyle {
                                        size: 14.0,
                                        color: Srgba::new(0.0, 0.5, 0.0, 1.0),
                                    },
                                ))
                                .child(text(
                                    "Row item 2",
                                    TextStyle {
                                        size: 14.0,
                                        color: Srgba::new(0.0, 0.0, 0.5, 1.0),
                                    },
                                ))
                                .child(text(
                                    "Row item 3",
                                    TextStyle {
                                        size: 14.0,
                                        color: Srgba::new(0.5, 0.0, 0.0, 1.0),
                                    },
                                )),
                        ),
                )
            });

            // Layer 2: Centered content with background
            layer_manager.add_taffy_ui_layer(1, LayerOptions::default(), || {
                Box::new(
                    group()
                        .size_full()
                        .justify_center()
                        .items_center()
                        .child(
                            group()
                                .bg(Srgba::new(0.9, 0.9, 0.9, 0.95))
                                .p(30.0)
                                .child(
                                    col()
                                        .gap(20.0)
                                        .child(text(
                                            "Centered Content",
                                            TextStyle {
                                                size: 28.0,
                                                color: Srgba::new(0.2, 0.2, 0.2, 1.0),
                                            },
                                        ))
                                        .child(text(
                                            "This div is centered in the window",
                                            TextStyle {
                                                size: 16.0,
                                                color: Srgba::new(0.4, 0.4, 0.4, 1.0),
                                            },
                                        ))
                                        .child(
                                            group()
                                                .bg(Srgba::new(0.2, 0.3, 0.8, 1.0))
                                                .size(200.0, 50.0)
                                                .justify_center()
                                                .items_center()
                                                .child(text(
                                                    "Button-like div",
                                                    TextStyle {
                                                        size: 16.0,
                                                        color: Srgba::new(1.0, 1.0, 1.0, 1.0),
                                                    },
                                                )),
                                        ),
                                ),
                        ),
                )
            });

            // Layer 3: Complex layout example
            layer_manager.add_taffy_ui_layer(2, LayerOptions::default(), || {
                Box::new(
                    group()
                        .flex_col()
                        .p(20.0)
                        .gap(20.0)
                        .child(
                            group()
                                .bg(Srgba::new(0.95, 0.95, 0.95, 0.9))
                                .p(20.0)
                                .w(600.0)
                                .child(
                                    col()
                                        .gap(15.0)
                                        .child(text(
                                            "Complex Layout Example",
                                            TextStyle {
                                                size: 24.0,
                                                color: Srgba::new(0.1, 0.1, 0.1, 1.0),
                                            },
                                        ))
                                        .child(text(
                                            "This demonstrates nested layouts with the builder pattern.",
                                            TextStyle {
                                                size: 16.0,
                                                color: Srgba::new(0.3, 0.3, 0.3, 1.0),
                                            },
                                        ))
                                        .child(
                                            row()
                                                .gap(10.0)
                                                .child(
                                                    group()
                                                        .bg(Srgba::new(1.0, 0.8, 0.8, 1.0))
                                                        .p(10.0)
                                                        .child(text(
                                                            "Card 1",
                                                            TextStyle {
                                                                size: 14.0,
                                                                color: Srgba::new(0.5, 0.0, 0.0, 1.0),
                                                            },
                                                        )),
                                                )
                                                .child(
                                                    group()
                                                        .bg(Srgba::new(0.8, 1.0, 0.8, 1.0))
                                                        .p(10.0)
                                                        .child(text(
                                                            "Card 2",
                                                            TextStyle {
                                                                size: 14.0,
                                                                color: Srgba::new(0.0, 0.5, 0.0, 1.0),
                                                            },
                                                        )),
                                                )
                                                .child(
                                                    group()
                                                        .bg(Srgba::new(0.8, 0.8, 1.0, 1.0))
                                                        .p(10.0)
                                                        .child(text(
                                                            "Card 3",
                                                            TextStyle {
                                                                size: 14.0,
                                                                color: Srgba::new(0.0, 0.0, 0.5, 1.0),
                                                            },
                                                        )),
                                                ),
                                        ),
                                ),
                        ),
                )
            });

            // Layer 4: Performance test with many elements
            layer_manager.add_taffy_ui_layer(3, LayerOptions::default(), || {
                let mut container = group()
                    .flex_col()
                    .p(20.0)
                    .gap(5.0)
                    .h(400.0)
                    .m(50.0)
                    .bg(Srgba::new(0.0, 0.0, 0.0, 0.1));

                // Add header
                container = container.child(text(
                    "Performance Test - 50 Items",
                    TextStyle {
                        size: 20.0,
                        color: Srgba::new(0.0, 0.0, 0.0, 1.0),
                    },
                ));

                // Create many items
                for i in 0..50 {
                    let hue = i as f32 / 50.0;
                    let color = Srgba::new(hue, 0.8, 0.9, 1.0);

                    container = container.child(
                        row()
                            .gap(10.0)
                            .child(
                                group()
                                    .bg(color)
                                    .size(30.0, 20.0),
                            )
                            .child(text(
                                format!("Item {}", i + 1),
                                TextStyle {
                                    size: 14.0,
                                    color: Srgba::new(0.3, 0.3, 0.3, 1.0),
                                },
                            )),
                    );
                }

                Box::new(container)
            });
        })
        .run();
}
