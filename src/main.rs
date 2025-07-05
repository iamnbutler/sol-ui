use palette::Srgba;
use toy_ui::{
    app,
    ui::{FrameStyle, UiContext},
};

fn main() {
    app()
        .size(800.0, 600.0)
        .title("Toy UI Frame Demo")
        .layer(|ui: &mut UiContext| {
            ui.text("Frame Styles Demo");

            ui.space(20.0);

            // Demonstrate different frame styles
            ui.vertical(|ui| {
                // Basic frame with rounded corners
                ui.text("Basic frame with rounded corners:");
                ui.frame(
                    glam::vec2(300.0, 60.0),
                    FrameStyle::new()
                        .with_background(Srgba::new(0.2, 0.3, 0.5, 1.0))
                        .with_corner_radius(15.0),
                );

                ui.space(20.0);

                // Frame with border
                ui.text("Frame with border:");
                ui.frame(
                    glam::vec2(300.0, 60.0),
                    FrameStyle::new()
                        .with_background(Srgba::new(0.3, 0.5, 0.3, 1.0))
                        .with_border(3.0, Srgba::new(0.1, 0.3, 0.1, 1.0))
                        .with_corner_radius(10.0),
                );

                ui.space(20.0);

                // Frame with thick border
                ui.text("Frame with thick border:");
                ui.frame(
                    glam::vec2(300.0, 60.0),
                    FrameStyle::new()
                        .with_background(Srgba::new(0.5, 0.3, 0.3, 1.0))
                        .with_border(8.0, Srgba::new(0.8, 0.2, 0.2, 1.0))
                        .with_corner_radius(20.0),
                );

                ui.space(20.0);

                // Frame with asymmetric corners
                ui.text("Frame with different corner radii:");
                ui.frame(
                    glam::vec2(300.0, 60.0),
                    FrameStyle::new()
                        .with_background(Srgba::new(0.4, 0.3, 0.5, 1.0))
                        .with_border(2.0, Srgba::new(0.8, 0.8, 0.8, 1.0))
                        .with_corner_radii(toy_ui::ui::CornerRadii::new(0.0, 25.0, 5.0, 25.0)),
                );
            });
        })
        .run();
}
