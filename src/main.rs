use palette::Srgba;
use toy_ui::{
    app,
    layer::{LayerManager, LayerOptions},
    taffy::UiTaffyContext,
    ui::TextStyle,
};

fn main() {
    app()
        .size(900.0, 800.0)
        .title("Toy UI - Text Rendering Test")
        .with_layers(|layer_manager: &mut LayerManager| {
            // Add a Taffy UI layer with proper layout
            layer_manager.add_taffy_ui_layer(
                0,
                LayerOptions::default().with_input(),
                |ui: &mut UiTaffyContext| {
                    ui.padded_column(0.0, |ui| {
                        ui.text(
                            "Text Rendering Test Suite",
                            TextStyle {
                                size: 32.0,
                                color: Srgba::new(0.1, 0.1, 0.1, 1.0),
                            },
                        );

                        ui.space(30.0);


                        // ui.text(
                        //     "Type Scale",
                        //     TextStyle {
                        //         size: 20.0,
                        //         color: Srgba::new(0.2, 0.2, 0.2, 1.0),
                        //     },
                        // );

                        // ui.space(15.0);

                        // let sizes = [48.0, 36.0, 28.0, 24.0, 20.0, 18.0, 16.0, 14.0, 12.0, 10.0];
                        // for size in sizes.iter() {
                        //     ui.text(
                        //         format!("{}px - The quick brown fox jumps over the lazy dog", size),
                        //         TextStyle {
                        //             size: *size,
                        //             color: Srgba::new(0.1, 0.1, 0.1, 1.0),
                        //         },
                        //     );
                        //     ui.space(8.0);
                        // }

                        // ui.space(30.0);


                        // ui.text(
                        //     "Character Set Test",
                        //     TextStyle {
                        //         size: 20.0,
                        //         color: Srgba::new(0.2, 0.2, 0.2, 1.0),
                        //     },
                        // );

                        // ui.space(15.0);

                        // // Uppercase
                        // ui.text(
                        //     "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
                        //     TextStyle {
                        //         size: 16.0,
                        //         color: Srgba::new(0.1, 0.1, 0.1, 1.0),
                        //     },
                        // );

                        // ui.space(5.0);

                        // // Lowercase
                        // ui.text(
                        //     "abcdefghijklmnopqrstuvwxyz",
                        //     TextStyle {
                        //         size: 16.0,
                        //         color: Srgba::new(0.1, 0.1, 0.1, 1.0),
                        //     },
                        // );

                        // ui.space(5.0);

                        // // Numbers
                        // ui.text(
                        //     "0123456789",
                        //     TextStyle {
                        //         size: 16.0,
                        //         color: Srgba::new(0.1, 0.1, 0.1, 1.0),
                        //     },
                        // );

                        // ui.space(5.0);

                        // // Special characters
                        // ui.text(
                        //     "!@#$%^&*()_+-=[]{}|;':\",./<>?",
                        //     TextStyle {
                        //         size: 16.0,
                        //         color: Srgba::new(0.1, 0.1, 0.1, 1.0),
                        //     },
                        // );

                        // ui.space(30.0);

                        // Paragraph Test
                        ui.begin_container(taffy::prelude::Style {
                            display: taffy::prelude::Display::Flex,
                            flex_direction: taffy::prelude::FlexDirection::Column,
                            padding: taffy::prelude::Rect {
                                left: taffy::prelude::LengthPercentage::length(20.0),
                                right: taffy::prelude::LengthPercentage::length(20.0),
                                top: taffy::prelude::LengthPercentage::length(20.0),
                                bottom: taffy::prelude::LengthPercentage::length(20.0),
                            },
                            size: taffy::prelude::Size {
                                width: taffy::prelude::Dimension::length(600.0),
                                height: taffy::prelude::Dimension::auto(),
                            },
                            ..Default::default()
                        });

                        ui.text(
                            "Paragraph Test",
                            TextStyle {
                                size: 20.0,
                                color: Srgba::new(0.2, 0.2, 0.2, 1.0),
                            },
                        );

                        ui.space(15.0);

                        // Body text
                        ui.text(
                            "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.",
                            TextStyle {
                                size: 16.0,
                                color: Srgba::new(0.1, 0.1, 0.1, 1.0),
                            },
                        );

                        ui.space(10.0);

                        ui.text(
                            "The quick brown fox jumps over the lazy dog. Pack my box with five dozen liquor jugs. How vexingly quick daft zebras jump! The five boxing wizards jump quickly.",
                            TextStyle {
                                size: 16.0,
                                color: Srgba::new(0.1, 0.1, 0.1, 1.0),
                            },
                        );

                        ui.end_container();

                        ui.space(30.0);

                        // Color and Style Test
                        ui.begin_container(taffy::prelude::Style {
                            display: taffy::prelude::Display::Flex,
                            flex_direction: taffy::prelude::FlexDirection::Column,
                            padding: taffy::prelude::Rect {
                                left: taffy::prelude::LengthPercentage::length(20.0),
                                right: taffy::prelude::LengthPercentage::length(20.0),
                                top: taffy::prelude::LengthPercentage::length(20.0),
                                bottom: taffy::prelude::LengthPercentage::length(20.0),
                            },
                            ..Default::default()
                        });

                        ui.text(
                            "Color and Style Test",
                            TextStyle {
                                size: 20.0,
                                color: Srgba::new(0.2, 0.2, 0.2, 1.0),
                            },
                        );

                        ui.space(15.0);

                        // Different colors
                        ui.text(
                            "Red text color test",
                            TextStyle {
                                size: 16.0,
                                color: Srgba::new(0.8, 0.1, 0.1, 1.0),
                            },
                        );
                        ui.text(
                            "Green text color test",
                            TextStyle {
                                size: 16.0,
                                color: Srgba::new(0.1, 0.7, 0.1, 1.0),
                            },
                        );
                        ui.text(
                            "Blue text color test",
                            TextStyle {
                                size: 16.0,
                                color: Srgba::new(0.1, 0.1, 0.8, 1.0),
                            },
                        );

                        ui.space(10.0);

                        // Different opacities
                        ui.text(
                            "100% opacity",
                            TextStyle {
                                size: 16.0,
                                color: Srgba::new(0.1, 0.1, 0.1, 1.0),
                            },
                        );
                        ui.text(
                            "75% opacity",
                            TextStyle {
                                size: 16.0,
                                color: Srgba::new(0.1, 0.1, 0.1, 0.75),
                            },
                        );
                        ui.text(
                            "50% opacity",
                            TextStyle {
                                size: 16.0,
                                color: Srgba::new(0.1, 0.1, 0.1, 0.5),
                            },
                        );
                        ui.text(
                            "25% opacity",
                            TextStyle {
                                size: 16.0,
                                color: Srgba::new(0.1, 0.1, 0.1, 0.25),
                            },
                        );

                        ui.end_container();
                    });
                },
            );
        })
        .run();
}
