//! Tooltip demo

use sol_ui::{
    app::app,
    color::colors,
    element::{button, container, row, text, tooltip},
    layer::LayerOptions,
    style::TextStyle,
};

fn main() {
    app()
        .title("Tooltip Demo")
        .size(800.0, 600.0)
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
                            .gap(40.0)
                            .child(text(
                                "Tooltip Demo",
                                TextStyle {
                                    color: colors::BLACK,
                                    size: 32.0,
                                },
                            ))
                            .child(text(
                                "Hover over the buttons to see tooltips",
                                TextStyle {
                                    color: colors::GRAY_600,
                                    size: 16.0,
                                },
                            ))
                            .child(
                                row()
                                    .gap(20.0)
                                    .child(
                                        tooltip("This is a helpful tooltip!")
                                            .top()
                                            .child(button("Hover me (top)")),
                                    )
                                    .child(
                                        tooltip("Tooltip appears below")
                                            .bottom()
                                            .child(button("Hover me (bottom)")),
                                    )
                                    .child(
                                        tooltip("Left side tooltip")
                                            .left()
                                            .child(button("Hover me (left)")),
                                    )
                                    .child(
                                        tooltip("Right side tooltip")
                                            .right()
                                            .child(button("Hover me (right)")),
                                    ),
                            ),
                    )
                },
            );
        })
        .run();
}
