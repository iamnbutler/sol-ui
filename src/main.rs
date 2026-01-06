use palette::Srgba;
use sol_ui::{
    app::app,
    element::{column, container, row, text},
    layer::{LayerManager, LayerOptions},
    style::TextStyle,
};
use tracing::{info, info_span};
use tracing_subscriber::{EnvFilter, fmt};

fn main() {
    // Initialize tracing
    let _subscriber = fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("sol=trace,info")),
        )
        .with_target(true)
        .with_thread_ids(true)
        .with_timer(fmt::time::uptime())
        .init();

    info!("Starting Toy UI application");
    let _main_span = info_span!("main").entered();

    app()
        .title("Toy UI - Taffy Layout Demo")
        .size(800.0, 600.0)
        .with_layers(|layer_manager: &mut LayerManager| {
            let _layers_span = info_span!("layer_setup").entered();

            // Layer 1: Basic text layout
            info!("Setting up Layer 1: Basic text layout");
            layer_manager.add_ui_layer(0, LayerOptions::default(), || {
                Box::new(
                    column()
                        .padding(20.0)
                        .gap(10.0)
                        .child(text(
                            "Hello from Taffy!",
                            TextStyle {
                                size: 24.0,
                                color: Srgba::new(0.0, 0.0, 0.0, 1.0),
                                ..Default::default()
                            },
                        ))
                        .child(text(
                            "This is a column layout with padding and gap.",
                            TextStyle {
                                size: 16.0,
                                color: Srgba::new(0.3, 0.3, 0.3, 1.0),
                                ..Default::default()
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
                                ..Default::default()
                                    },
                                ))
                                .child(text(
                                    "Row item 2",
                                    TextStyle {
                                        size: 14.0,
                                        color: Srgba::new(0.0, 0.0, 0.5, 1.0),
                                ..Default::default()
                                    },
                                ))
                                .child(text(
                                    "Row item 3",
                                    TextStyle {
                                        size: 14.0,
                                        color: Srgba::new(0.5, 0.0, 0.0, 1.0),
                                ..Default::default()
                                    },
                                )),
                        ),
                )
            });

            // Layer 2: Centered content with background
            info!("Setting up Layer 2: Centered content with background");
            layer_manager.add_ui_layer(1, LayerOptions::default(), || {
                Box::new(
                    container()
                        .width_full()
                        .height_full()
                        .justify_center()
                        .items_center()
                        .child(
                            container()
                                .background(Srgba::new(0.9, 0.9, 0.9, 0.95))
                                .padding(30.0)
                                .child(
                                    column()
                                        .gap(20.0)
                                        .child(text(
                                            "Centered Content",
                                            TextStyle {
                                                size: 28.0,
                                                color: Srgba::new(0.2, 0.2, 0.2, 1.0),
                                ..Default::default()
                                            },
                                        ))
                                        .child(text(
                                            "This div is centered in the window",
                                            TextStyle {
                                                size: 16.0,
                                                color: Srgba::new(0.4, 0.4, 0.4, 1.0),
                                ..Default::default()
                                            },
                                        ))
                                        .child(
                                            container()
                                                .background(Srgba::new(0.2, 0.3, 0.8, 1.0))
                                                .size(200.0, 50.0)
                                                .justify_center()
                                                .items_center()
                                                .child(text(
                                                    "Button-like div",
                                                    TextStyle {
                                                        size: 16.0,
                                                        color: Srgba::new(1.0, 1.0, 1.0, 1.0),
                                ..Default::default()
                                                    },
                                                )),
                                        ),
                                ),
                        ),
                )
            });

            // Layer 3: Complex layout example
            info!("Setting up Layer 3: Complex layout example");
            layer_manager.add_ui_layer(2, LayerOptions::default(), || {
                Box::new(
                    container()
                        .flex_col()
                        .padding(20.0)
                        .gap(20.0)
                        .child(
                            container()
                                .background(Srgba::new(0.95, 0.95, 0.95, 0.9))
                                .padding(20.0)
                                .width(600.0)
                                .child(
                                    column()
                                        .gap(15.0)
                                        .child(text(
                                            "Complex Layout Example",
                                            TextStyle {
                                                size: 24.0,
                                                color: Srgba::new(0.1, 0.1, 0.1, 1.0),
                                ..Default::default()
                                            },
                                        ))
                                        .child(text(
                                            "This demonstrates nested layouts with the builder pattern.",
                                            TextStyle {
                                                size: 16.0,
                                                color: Srgba::new(0.3, 0.3, 0.3, 1.0),
                                ..Default::default()
                                            },
                                        ))
                                        .child(
                                            row()
                                                .gap(10.0)
                                                .child(
                                                    container()
                                                        .background(Srgba::new(1.0, 0.8, 0.8, 1.0))
                                                        .padding(10.0)
                                                        .child(text(
                                                            "Card 1",
                                                            TextStyle {
                                                                size: 14.0,
                                                                color: Srgba::new(0.5, 0.0, 0.0, 1.0),
                                ..Default::default()
                                                            },
                                                        )),
                                                )
                                                .child(
                                                    container()
                                                        .background(Srgba::new(0.8, 1.0, 0.8, 1.0))
                                                        .padding(10.0)
                                                        .child(text(
                                                            "Card 2",
                                                            TextStyle {
                                                                size: 14.0,
                                                                color: Srgba::new(0.0, 0.5, 0.0, 1.0),
                                ..Default::default()
                                                            },
                                                        )),
                                                )
                                                .child(
                                                    container()
                                                        .background(Srgba::new(0.8, 0.8, 1.0, 1.0))
                                                        .padding(10.0)
                                                        .child(text(
                                                            "Card 3",
                                                            TextStyle {
                                                                size: 14.0,
                                                                color: Srgba::new(0.0, 0.0, 0.5, 1.0),
                                ..Default::default()
                                                            },
                                                        )),
                                                ),
                                        ),
                                ),
                        ),
                )
            });

            // Layer 4: Performance test with many elements
            info!("Setting up Layer 4: Performance test with many elements");
            layer_manager.add_ui_layer(3, LayerOptions::default(), || {
                let mut root_container = container()
                    .flex_col()
                    .padding(20.0)
                    .gap(5.0)
                    .height(400.0)
                    .margin(50.0)
                    .background(Srgba::new(0.0, 0.0, 0.0, 0.1));

                // Add header
                root_container = root_container.child(text(
                    "Performance Test - 50 Items",
                    TextStyle {
                        size: 20.0,
                        color: Srgba::new(0.0, 0.0, 0.0, 1.0),
                                ..Default::default()
                    },
                ));

                // Create many items
                for i in 0..50 {
                    let hue = i as f32 / 50.0;
                    let color = Srgba::new(hue, 0.8, 0.9, 1.0);

                    root_container = root_container.child(
                        row()
                            .gap(10.0)
                            .child(
                                container()
                                    .background(color)
                                    .size(30.0, 20.0),
                            )
                            .child(text(
                                format!("Item {}", i + 1),
                                TextStyle {
                                    size: 14.0,
                                    color: Srgba::new(0.3, 0.3, 0.3, 1.0),
                                ..Default::default()
                                },
                            )),
                    );
                }

                Box::new(root_container)
            });
            info!("All layers setup complete");
        })
        .run();
}
