use palette::Srgba;
use toy_ui::{app, ui::UiContext};

fn main() {
    app()
        .size(800.0, 600.0)
        .title("Toy UI Demo")
        .layer(|ui: &mut UiContext| {
            ui.text("Hello from Toy UI!");

            ui.space(20.0);

            ui.horizontal(|ui| {
                ui.rect(
                    glam::vec2(50.0, 50.0),
                    Srgba::new(0.863, 0.078, 0.235, 1.0), // Red
                );
                ui.rect(
                    glam::vec2(50.0, 50.0),
                    Srgba::new(0.196, 0.804, 0.196, 1.0), // Green
                );
                ui.rect(
                    glam::vec2(50.0, 50.0),
                    Srgba::new(0.118, 0.565, 1.0, 1.0), // Blue
                );
            });
        })
        .layer(|ui: &mut UiContext| {
            ui.space(40.0);

            // Create 20 rows of text with a distinct background
            ui.group_styled(
                Some(Srgba::new(0.282, 0.239, 0.545, 1.0)), // Dark purple
                |ui| {
                    ui.vertical(|ui| {
                        ui.text("Second Layer - Text Rows");
                        ui.space(10.0);

                        for i in 0..20 {
                            ui.vertical(|ui| {
                                ui.text(&format!("Row {}: ", i + 1));
                            });
                        }
                    });
                },
            );
        })
        .run();
}
