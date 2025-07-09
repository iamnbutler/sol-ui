use palette::Srgba;
use toy_ui::{
    app,
    ui::{FrameStyle, UiContext, vec2},
};

fn main() {
    app()
        .size(800.0, 600.0)
        .title("Toy UI - Nested Frames Demo")
        .layer(|ui: &mut UiContext| {
            // Light background
            ui.rect(vec2(800.0, 600.0), Srgba::new(0.54, 0.54, 0.54, 1.0));
            ui.set_cursor(vec2(30.0, 30.0));

            ui.text("Nested Frames Demo");
            ui.space(20.0);

            // Example 1: Simple nested frame
            ui.frame_container_padded(
                FrameStyle::new()
                    .with_background(Srgba::new(1.0, 1.0, 1.0, 1.0))
                    .with_corner_radius(12.0)
                    .with_shadow(vec2(0.0, 4.0), 10.0, Srgba::new(0.0, 0.0, 0.0, 0.15)),
                20.0,
                |ui| {
                    ui.text("Container Frame");
                    ui.space(10.0);

                    ui.frame_container_padded(
                        FrameStyle::new()
                            .with_linear_gradient(
                                Srgba::new(0.9, 0.95, 1.0, 1.0),
                                Srgba::new(0.8, 0.85, 0.95, 1.0),
                                std::f32::consts::PI / 2.0,
                            )
                            .with_border(1.0, Srgba::new(0.6, 0.7, 0.9, 1.0))
                            .with_corner_radius(8.0),
                        15.0,
                        |ui| {
                            ui.text("Nested content");
                            ui.text("With gradient background");
                        },
                    );
                },
            );

            ui.space(30.0);

            // Example 2: Card with sections
            ui.frame_container(
                FrameStyle::new()
                    .with_background(Srgba::new(1.0, 1.0, 1.0, 1.0))
                    .with_corner_radius(10.0)
                    .with_shadow(vec2(0.0, 2.0), 8.0, Srgba::new(0.0, 0.0, 0.0, 0.1)),
                |ui| {
                    // Header section
                    ui.frame_container_padded(
                        FrameStyle::new()
                            .with_background(Srgba::new(0.2, 0.4, 0.8, 1.0))
                            .with_corner_radii(toy_ui::ui::CornerRadii::new(10.0, 10.0, 0.0, 0.0)),
                        15.0,
                        |ui| {
                            ui.text_styled(
                                "Feature Card",
                                toy_ui::ui::TextStyle {
                                    size: 18.0,
                                    color: Srgba::new(1.0, 1.0, 1.0, 1.0),
                                },
                            );
                        },
                    );

                    // Content section
                    ui.frame_container_padded(
                        FrameStyle::new()
                            .with_background(Srgba::new(1.0, 1.0, 1.0, 1.0))
                            .with_corner_radii(toy_ui::ui::CornerRadii::new(0.0, 0.0, 10.0, 10.0)),
                        15.0,
                        |ui| {
                            ui.text("Clean design");
                            ui.text("Proper nesting");
                            ui.text("Beautiful shadows");
                        },
                    );
                },
            );

            ui.space(30.0);

            // Example 3: Notification style with icon
            ui.frame_container_padded(
                FrameStyle::new()
                    .with_radial_gradient(
                        Srgba::new(0.3, 0.8, 0.5, 1.0),
                        Srgba::new(0.2, 0.7, 0.4, 1.0),
                    )
                    .with_corner_radius(8.0)
                    .with_shadow(vec2(0.0, 3.0), 8.0, Srgba::new(0.0, 0.0, 0.0, 0.2)),
                15.0,
                |ui| {
                    ui.text_styled(
                        "✓ Success",
                        toy_ui::ui::TextStyle {
                            size: 16.0,
                            color: Srgba::new(1.0, 1.0, 1.0, 1.0),
                        },
                    );
                    ui.text_styled(
                        "Frame nesting is working!",
                        toy_ui::ui::TextStyle {
                            size: 14.0,
                            color: Srgba::new(1.0, 1.0, 1.0, 0.9),
                        },
                    );
                },
            );
        })
        .run();
}
