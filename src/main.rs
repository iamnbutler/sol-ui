use palette::Srgba;
use toy_ui::{
    app,
    ui::{TextStyle, UiContext, vec2},
};

fn main() {
    app()
        .size(900.0, 800.0)
        .title("Toy UI - Text Rendering Test")
        .layer(|ui: &mut UiContext| {
            // Light background
            ui.rect(vec2(900.0, 800.0), Srgba::new(0.95, 0.95, 0.95, 1.0));
            ui.set_cursor(vec2(40.0, 40.0));

            // Title
            ui.text_styled(
                "Text Rendering Test Suite",
                TextStyle {
                    size: 32.0,
                    color: Srgba::new(0.1, 0.1, 0.1, 1.0),
                },
            );
            ui.space(30.0);

            ui.text_styled(
                "Character Set Test",
                TextStyle {
                    size: 20.0,
                    color: Srgba::new(0.2, 0.2, 0.2, 1.0),
                },
            );
            ui.space(15.0);

            // Uppercase
            ui.text_styled(
                "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
                TextStyle {
                    size: 10.0,
                    color: Srgba::new(0.1, 0.1, 0.1, 1.0),
                },
            );
            ui.space(5.0);

            // Lowercase
            ui.text_styled(
                "abcdefghijklmnopqrstuvwxyz",
                TextStyle {
                    size: 16.0,
                    color: Srgba::new(0.1, 0.1, 0.1, 1.0),
                },
            );
            ui.space(5.0);

            // Numbers
            ui.text_styled(
                "0123456789",
                TextStyle {
                    size: 16.0,
                    color: Srgba::new(0.1, 0.1, 0.1, 1.0),
                },
            );
            ui.space(5.0);

            // Special characters
            ui.text_styled(
                "!@#$%^&*()_+-=[]{}|;':\",./<>?",
                TextStyle {
                    size: 16.0,
                    color: Srgba::new(0.1, 0.1, 0.1, 1.0),
                },
            );

            ui.space(20.0);

            ui.text_styled(
                "The quick brown fox jumps over the lazy dog",
                TextStyle {
                    size: 48.0,
                    color: Srgba::new(0.2, 0.2, 0.2, 1.0),
                },
            );
            ui.space(8.0);
            ui.text_styled(
                "The quick brown fox jumps over the lazy dog",
                TextStyle {
                    size: 32.0,
                    color: Srgba::new(0.2, 0.2, 0.2, 1.0),
                },
            );
            ui.space(8.0);
            ui.text_styled(
                "The quick brown fox jumps over the lazy dog",
                TextStyle {
                    size: 24.0,
                    color: Srgba::new(0.2, 0.2, 0.2, 1.0),
                },
            );
            ui.space(8.0);
            ui.text_styled(
                "The quick brown fox jumps over the lazy dog",
                TextStyle {
                    size: 16.0,
                    color: Srgba::new(0.2, 0.2, 0.2, 1.0),
                },
            );
            ui.space(8.0);
            ui.text_styled(
                "The quick brown fox jumps over the lazy dog",
                TextStyle {
                    size: 12.0,
                    color: Srgba::new(0.2, 0.2, 0.2, 1.0),
                },
            );
            ui.space(8.0);
            ui.text_styled(
                "The quick brown fox jumps over the lazy dog",
                TextStyle {
                    size: 8.0,
                    color: Srgba::new(0.2, 0.2, 0.2, 1.0),
                },
            );
            ui.space(8.0);
            ui.text_styled(
                "The quick brown fox jumps over the lazy dog",
                TextStyle {
                    size: 4.0,
                    color: Srgba::new(0.2, 0.2, 0.2, 1.0),
                },
            );
            ui.space(8.0);
            ui.text_styled(
                "The quick brown fox jumps over the lazy dog",
                TextStyle {
                    size: 2.0,
                    color: Srgba::new(0.2, 0.2, 0.2, 1.0),
                },
            );
            ui.space(8.0);
        })
        .run();
}
