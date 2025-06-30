use crate::layer::{LayerManager, LayerOptions};
use crate::platform::{Window, create_app_menu};
use crate::renderer::MetalRenderer;
use crate::text::TextSystem;

use cocoa::base::{YES, id};
use metal::{CommandQueue, Device};
use objc::{class, msg_send, sel, sel_impl};
use palette::{Srgba, named};
use std::sync::Arc;

pub struct App {
    window: Arc<Window>,
    device: Device,
    command_queue: CommandQueue,
    renderer: MetalRenderer,
    layer_manager: LayerManager,
    text_system: TextSystem,
}

impl App {
    pub fn new() -> Self {
        // Initialize NSApplication
        let ns_app: id = unsafe { msg_send![class!(NSApplication), sharedApplication] };
        let _: () = unsafe { msg_send![ns_app, setActivationPolicy: 0] }; // NSApplicationActivationPolicyRegular

        // Create app menu
        create_app_menu();

        // Create Metal device and command queue
        let device = Device::system_default().expect("No Metal device found");
        let command_queue = device.new_command_queue();

        // Create window
        let window = Window::new(800.0, 600.0, "Toy UI", &device);

        // Create and initialize renderer
        let mut renderer = MetalRenderer::new(device.clone());
        if let Err(e) = renderer.initialize() {
            panic!("Failed to initialize renderer: {}", e);
        }

        // Create layer manager
        let layer_manager = LayerManager::new();

        // Create text system
        let text_system = TextSystem::new(&device).expect("Failed to create text system");

        // Activate app and bring to front
        let _: () = unsafe { msg_send![ns_app, activateIgnoringOtherApps: YES] };

        Self {
            window,
            device,
            command_queue,
            renderer,
            layer_manager,
            text_system,
        }
    }

    pub fn run(mut self) {
        while self.window.handle_events() {
            self.render_frame();
        }
    }

    fn render_frame(&mut self) {
        // Get the next drawable from the Metal layer
        let drawable = match self.window.metal_layer().next_drawable() {
            Some(drawable) => drawable,
            None => {
                return; // No drawable available, skip frame
            }
        };

        // Get window size
        let size = self.window.size();

        // Clear layers from previous frame
        self.layer_manager.clear();

        // Add main UI layer
        self.layer_manager.add_ui_layer(
            0, // z-index
            LayerOptions::default()
                .with_clear()
                .with_clear_color(0.8, 0.8, 0.8, 1.0), // Light gray background
            |ui| {
                // Example UI
                ui.text("Hello from Toy UI!");

                ui.space(20.0);

                ui.group_styled(
                    Some(Srgba::from(named::DARKSLATEGRAY).into_format()),
                    |ui| {
                        ui.text("This is a group container");
                        ui.text("With multiple lines of text");

                        ui.space(10.0);

                        ui.horizontal(|ui| {
                            // Use palette's named colors
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
                    },
                );

                ui.space(20.0);

                ui.window(
                    "Example Window",
                    glam::vec2(400.0, 200.0),
                    glam::vec2(300.0, 200.0),
                    |ui| {
                        ui.text("This is a window!");
                        ui.text("It has a title bar and content area.");
                    },
                );
            },
        );

        // Create command buffer
        let command_buffer = self.command_queue.new_command_buffer();

        // Render all layers
        self.layer_manager.render(
            &mut self.renderer,
            &command_buffer,
            &drawable,
            glam::vec2(size.0, size.1),
            &mut self.text_system,
        );

        // Present drawable and commit
        command_buffer.present_drawable(&drawable);
        command_buffer.commit();
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn command_queue(&self) -> &CommandQueue {
        &self.command_queue
    }

    pub fn window(&self) -> &Window {
        &self.window
    }
}
