//! Simple scroll test to verify clipping works

use sol_ui::{
    app::app,
    color::colors,
    element::{container, scroll, text},
    layer::LayerOptions,
    style::TextStyle,
};

fn main() {
    app()
        .title("Scroll Clipping Test")
        .size(600.0, 400.0)
        .with_layers(move |layers| {
            layers.add_ui_layer(
                0,
                LayerOptions::default().with_input().with_clear(),
                move || {
                    Box::new(
                        container()
                            .width_full()
                            .height_full()
                            .background(colors::GRAY_100)
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .gap(20.0)
                            .child(text(
                                "Scroll Clipping Test",
                                TextStyle {
                                    color: colors::BLACK,
                                    size: 24.0,
                                },
                            ))
                            .child(text(
                                "Scroll the box below - content should be clipped",
                                TextStyle {
                                    color: colors::GRAY_600,
                                    size: 14.0,
                                },
                            ))
                            .child(
                                // Scroll container with many items
                                scroll()
                                    .width(300.0)
                                    .height(200.0)
                                    .background(colors::WHITE)
                                    .scrollbar(true)
                                    .child(
                                        container()
                                            .width_full()
                                            .flex_col()
                                            .gap(10.0)
                                            .padding(15.0)
                                            .child(text(
                                                "Item 1 - This text should clip at the container edge",
                                                TextStyle {
                                                    color: colors::BLACK,
                                                    size: 16.0,
                                                },
                                            ))
                                            .child(text(
                                                "Item 2 - Scroll down to see more",
                                                TextStyle {
                                                    color: colors::BLACK,
                                                    size: 16.0,
                                                },
                                            ))
                                            .child(text(
                                                "Item 3 - The clipping should work",
                                                TextStyle {
                                                    color: colors::BLUE_500,
                                                    size: 16.0,
                                                },
                                            ))
                                            .child(text(
                                                "Item 4 - More content here",
                                                TextStyle {
                                                    color: colors::BLACK,
                                                    size: 16.0,
                                                },
                                            ))
                                            .child(text(
                                                "Item 5 - Keep scrolling",
                                                TextStyle {
                                                    color: colors::BLACK,
                                                    size: 16.0,
                                                },
                                            ))
                                            .child(text(
                                                "Item 6 - Almost at the bottom",
                                                TextStyle {
                                                    color: colors::GREEN_500,
                                                    size: 16.0,
                                                },
                                            ))
                                            .child(text(
                                                "Item 7 - This is the last item",
                                                TextStyle {
                                                    color: colors::RED_500,
                                                    size: 16.0,
                                                },
                                            ))
                                            .child(text(
                                                "Item 8 - Just kidding, one more",
                                                TextStyle {
                                                    color: colors::PURPLE_500,
                                                    size: 16.0,
                                                },
                                            ))
                                            .child(text(
                                                "Item 9 - Bonus item",
                                                TextStyle {
                                                    color: colors::GRAY_700,
                                                    size: 16.0,
                                                },
                                            ))
                                            .child(text(
                                                "Item 10 - THE END",
                                                TextStyle {
                                                    color: colors::BLACK,
                                                    size: 20.0,
                                                },
                                            )),
                                    ),
                            ),
                    )
                },
            );
        })
        .run();
}
