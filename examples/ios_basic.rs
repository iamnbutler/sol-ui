//! Basic iOS example for sol-ui
//!
//! This example demonstrates:
//! - Creating a window on iOS
//! - Basic Metal rendering
//! - Touch input handling
//! - Simple UI elements

use sol_ui::{
    app::App,
    color::Color,
    element::{Container, Element, Text},
    geometry::{Point, Size},
    layer::{InputEvent, Layer, LayerOptions, UILayer},
    platform::Window,
    style::{ElementStyle, Fill},
};
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

fn main() {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    info!("Starting iOS basic example");

    // Create window - on iOS this will use UIWindow
    let mut window = Window::new("Sol UI iOS Example", 375.0, 812.0); // iPhone X/11 dimensions

    // Create the app
    let mut app = App::new();

    // Create a simple UI layer
    let mut ui_layer = UILayer::new(
        Size::new(375.0, 812.0),
        LayerOptions::default()
            .with_input()
            .with_clear()
            .with_clear_color(0.95, 0.95, 0.97, 1.0), // iOS-style light gray background
    );

    // Build a simple UI
    let root = build_ui();
    ui_layer.set_root(root);

    // Add the layer to the app
    app.add_layer(Box::new(ui_layer));

    // State for touch tracking
    let mut touch_position = None;
    let mut touch_active = false;

    // Run the event loop
    window.run_event_loop(move |window| {
        // Process input events
        let events = window.process_events();
        for event in events {
            match event {
                InputEvent::TouchDown { position, id } => {
                    info!("Touch down at {:?} with id {}", position, id);
                    touch_position = Some(position);
                    touch_active = true;
                    app.handle_input(event);
                }
                InputEvent::TouchMove { position, id } => {
                    info!("Touch move to {:?} with id {}", position, id);
                    touch_position = Some(position);
                    app.handle_input(event);
                }
                InputEvent::TouchUp { position, id } => {
                    info!("Touch up at {:?} with id {}", position, id);
                    touch_position = None;
                    touch_active = false;
                    app.handle_input(event);
                }
                InputEvent::TouchCancel { id } => {
                    info!("Touch cancelled for id {}", id);
                    touch_position = None;
                    touch_active = false;
                    app.handle_input(event);
                }
                _ => {
                    app.handle_input(event);
                }
            }
        }

        // Update the app
        app.update(16.67); // ~60 FPS

        // Render
        if let Some(drawable) = window.metal_layer().next_drawable() {
            let command_buffer = window.command_queue().new_command_buffer();

            // Render all layers
            app.render(command_buffer, &drawable);

            command_buffer.present_drawable(&drawable);
            command_buffer.commit();
        }

        true // Continue running
    });
}

fn build_ui() -> Arc<dyn Element> {
    // Create a container with padding
    let mut container = Container::new();
    container.style_mut().padding = [20.0, 20.0, 20.0, 20.0];
    container.style_mut().background = Some(Fill::Solid(Color::white()));
    container.style_mut().corner_radius = 16.0;
    container.style_mut().shadow_offset = Point::new(0.0, 2.0);
    container.style_mut().shadow_blur = 8.0;
    container.style_mut().shadow_color = Color::rgba(0.0, 0.0, 0.0, 0.1);

    // Add a title
    let mut title = Text::new("Sol UI on iOS");
    title.style_mut().font_size = 32.0;
    title.style_mut().color = Color::rgb(0.0, 0.0, 0.0);
    title.style_mut().margin = [0.0, 0.0, 20.0, 0.0];
    container.add_child(Arc::new(title));

    // Add description text
    let mut description = Text::new("This is a basic example running on iOS with touch support.");
    description.style_mut().font_size = 16.0;
    description.style_mut().color = Color::rgb(0.3, 0.3, 0.3);
    description.style_mut().margin = [0.0, 0.0, 20.0, 0.0];
    container.add_child(Arc::new(description));

    // Create a button-like element
    let mut button_container = Container::new();
    button_container.style_mut().padding = [12.0, 24.0, 12.0, 24.0];
    button_container.style_mut().background = Some(Fill::Solid(Color::rgb(0.0, 0.478, 1.0))); // iOS blue
    button_container.style_mut().corner_radius = 8.0;
    button_container.style_mut().margin = [20.0, 0.0, 0.0, 0.0];

    let mut button_text = Text::new("Touch Me");
    button_text.style_mut().font_size = 18.0;
    button_text.style_mut().color = Color::white();
    button_container.add_child(Arc::new(button_text));

    container.add_child(Arc::new(button_container));

    // Create some status indicators
    let mut status_container = Container::new();
    status_container.style_mut().margin = [40.0, 0.0, 0.0, 0.0];
    status_container.style_mut().padding = [16.0, 16.0, 16.0, 16.0];
    status_container.style_mut().background = Some(Fill::Solid(Color::rgba(0.9, 0.9, 0.9, 1.0)));
    status_container.style_mut().corner_radius = 8.0;

    let mut status_title = Text::new("System Info");
    status_title.style_mut().font_size = 14.0;
    status_title.style_mut().color = Color::rgb(0.5, 0.5, 0.5);
    status_title.style_mut().margin = [0.0, 0.0, 8.0, 0.0];
    status_container.add_child(Arc::new(status_title));

    let mut platform_text = Text::new("Platform: iOS");
    platform_text.style_mut().font_size = 16.0;
    platform_text.style_mut().color = Color::rgb(0.2, 0.2, 0.2);
    platform_text.style_mut().margin = [0.0, 0.0, 4.0, 0.0];
    status_container.add_child(Arc::new(platform_text));

    let mut renderer_text = Text::new("Renderer: Metal");
    renderer_text.style_mut().font_size = 16.0;
    renderer_text.style_mut().color = Color::rgb(0.2, 0.2, 0.2);
    renderer_text.style_mut().margin = [0.0, 0.0, 4.0, 0.0];
    status_container.add_child(Arc::new(renderer_text));

    let mut input_text = Text::new("Input: Touch");
    input_text.style_mut().font_size = 16.0;
    input_text.style_mut().color = Color::rgb(0.2, 0.2, 0.2);
    status_container.add_child(Arc::new(input_text));

    container.add_child(Arc::new(status_container));

    // Add instructions
    let mut instructions = Text::new("Touch anywhere to interact with the UI");
    instructions.style_mut().font_size = 14.0;
    instructions.style_mut().color = Color::rgb(0.6, 0.6, 0.6);
    instructions.style_mut().margin = [40.0, 0.0, 0.0, 0.0];
    container.add_child(Arc::new(instructions));

    Arc::new(container)
}
