use crate::platform::{Window, create_app_menu};
use crate::renderer::MetalRenderer;
use crate::text::TextSystem;
use crate::ui::UiContext;
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
    ui_context: UiContext,
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

        // Create UI context
        let ui_context = UiContext::new(800.0, 600.0);

        // Create text system
        let text_system = TextSystem::new(&device).expect("Failed to create text system");

        // Activate app and bring to front
        let _: () = unsafe { msg_send![ns_app, activateIgnoringOtherApps: YES] };

        Self {
            window,
            device,
            command_queue,
            renderer,
            ui_context,
            text_system,
        }
    }

    pub fn run(mut self) {
        while self.window.handle_events() {
            // TODO: Render frame here
            self.render_frame();
        }
    }

    fn render_frame(&mut self) {
        // Get the next drawable
        let drawable = match self.window.metal_layer().next_drawable() {
            Some(drawable) => drawable,
            None => return, // No drawable available, skip frame
        };

        // Build UI for this frame
        self.ui_context.begin_frame();

        // Example UI usage
        self.ui_context.text("Hello from Toy UI!");

        self.ui_context.space(20.0);

        self.ui_context.group_styled(
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

        self.ui_context.space(20.0);

        self.ui_context.window(
            "Example Window",
            glam::vec2(400.0, 200.0),
            glam::vec2(300.0, 200.0),
            |ui| {
                ui.text("This is a window!");
                ui.text("It has a title bar and content area.");
            },
        );

        // Get screen size before end_frame to avoid borrow conflict
        let screen_size = self.ui_context.screen_size();
        let draw_list = self.ui_context.end_frame();

        // Use the renderer to draw the UI
        let clear_color = metal::MTLClearColor::new(0.8, 0.8, 0.8, 1.0); // Light gray

        self.renderer.render_frame(
            &self.command_queue,
            &drawable,
            clear_color,
            draw_list,
            (screen_size.x, screen_size.y),
            &mut self.text_system,
        );
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
