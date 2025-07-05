use palette::{Srgba, named};
use toy_ui::{app, ui::UiContext};

fn main() {
    app()
        .size(800.0, 600.0)
        .title("Toy UI Demo")
        .layer(|ui: &mut UiContext| {
            ui.space(20.0);

            ui.text("Hello from Toy UI!");

            ui.space(20.0);

            ui.horizontal(|ui| {
                ui.rect(
                    glam::vec2(50.0, 50.0),
                    Srgba::from(named::CRIMSON).into_format(),
                );
                ui.rect(
                    glam::vec2(50.0, 50.0),
                    Srgba::from(named::LIMEGREEN).into_format(),
                );
                ui.rect(
                    glam::vec2(50.0, 50.0),
                    Srgba::from(named::DODGERBLUE).into_format(),
                );
            });
        })
        .run();
}
