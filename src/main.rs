use palette::Srgba;
use toy_ui::{
    app,
    ui::{FrameStyle, UiContext, vec2},
};

fn main() {
    app()
        .size(800.0, 600.0)
        .title("Toy UI Frames Debug")
        .layer(|ui: &mut UiContext| {
            // Add a light background to see shadows better
            ui.rect(vec2(800.0, 600.0), Srgba::new(0.95, 0.95, 0.95, 1.0));
            ui.set_cursor(vec2(20.0, 20.0));

            ui.text("Frame Debug Test");
            ui.space(20.0);

            // Simple gradient test
            ui.text("Linear gradient:");
            ui.frame(
                vec2(200.0, 80.0),
                FrameStyle::new()
                    .with_linear_gradient(
                        Srgba::new(1.0, 0.0, 0.0, 1.0),
                        Srgba::new(0.0, 0.0, 1.0, 1.0),
                        0.0, // horizontal
                    )
                    .with_corner_radius(15.0)
                    .with_shadow(vec2(0.0, 4.0), 12.0, Srgba::new(0.5, 0.0, 0.5, 0.3)),
            );

            ui.space(30.0);

            // Add a more dramatic shadow example
            ui.text("Dramatic shadow (elevated card):");
            ui.frame(
                vec2(200.0, 80.0),
                FrameStyle::new()
                    .with_background(Srgba::new(1.0, 1.0, 1.0, 1.0))
                    .with_corner_radius(8.0)
                    .with_shadow(vec2(0.0, 8.0), 20.0, Srgba::new(0.0, 0.0, 0.0, 0.25)),
            );

            ui.space(30.0);

            // Test 0 blur shadows (sharp shadows)
            ui.text("Sharp shadow (0 blur):");
            ui.frame(
                vec2(200.0, 80.0),
                FrameStyle::new()
                    .with_background(Srgba::new(1.0, 1.0, 1.0, 1.0))
                    .with_corner_radius(10.0)
                    .with_shadow(vec2(5.0, 5.0), 0.0, Srgba::new(0.0, 0.0, 0.0, 0.5)),
            );

            ui.space(30.0);

            // Multiple sharp shadows for layered effect
            ui.text("Colored sharp shadow:");
            ui.frame(
                vec2(200.0, 80.0),
                FrameStyle::new()
                    .with_linear_gradient(
                        Srgba::new(0.9, 0.4, 0.1, 1.0),
                        Srgba::new(0.9, 0.7, 0.1, 1.0),
                        std::f32::consts::PI / 2.0, // vertical
                    )
                    .with_corner_radius(12.0)
                    .with_shadow(vec2(3.0, 3.0), 0.0, Srgba::new(0.6, 0.2, 0.0, 0.6)),
            );
        })
        .run();
}
