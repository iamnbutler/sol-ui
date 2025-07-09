use crate::layer::LayerManager;
use crate::metal_renderer::MetalRenderer;
use crate::platform::{Window, create_app_menu};
use crate::text_system::TextSystem;

use cocoa::base::{YES, id};
use metal::{CommandQueue, Device};
use objc::{class, msg_send, sel, sel_impl};

use std::sync::Arc;

pub struct App {
    window: Arc<Window>,
    device: Device,
    command_queue: CommandQueue,
    renderer: MetalRenderer,
    _layer_manager: LayerManager,
    text_system: TextSystem,
}

pub struct AppBuilder {
    width: f64,
    height: f64,
    title: String,
    layer_setup: Box<dyn FnOnce(&mut LayerManager)>,
}

pub fn app() -> AppBuilder {
    AppBuilder::new()
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            width: 800.0,
            height: 600.0,
            title: "Toy UI App".to_string(),
            layer_setup: Box::new(|_| {}),
        }
    }

    pub fn size(mut self, width: f64, height: f64) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_layers<F>(mut self, setup: F) -> Self
    where
        F: FnOnce(&mut LayerManager) + 'static,
    {
        self.layer_setup = Box::new(setup);
        self
    }

    pub fn run(mut self) {
        let layer_setup = std::mem::replace(&mut self.layer_setup, Box::new(|_| {}));
        let app = self.build();
        app.run(layer_setup);
    }

    fn build(self) -> App {
        // Initialize NSApplication
        let ns_app: id = unsafe { msg_send![class!(NSApplication), sharedApplication] };
        let _: () = unsafe { msg_send![ns_app, setActivationPolicy: 0] }; // NSApplicationActivationPolicyRegular

        // Create app menu
        create_app_menu();

        // Create Metal device and command queue
        let device = Device::system_default().expect("No Metal device found");
        let command_queue = device.new_command_queue();

        // Create window
        let window = Window::new(self.width, self.height, &self.title, &device);

        // Create and initialize renderer
        let mut renderer = MetalRenderer::new(device.clone());
        if let Err(e) = renderer.initialize() {
            panic!("Failed to initialize renderer: {}", e);
        }

        // Create layer manager
        let _layer_manager = LayerManager::new();

        // Create text system
        let text_system = TextSystem::new(&device).expect("Failed to create text system");

        // Activate app and bring to front
        let _: () = unsafe { msg_send![ns_app, activateIgnoringOtherApps: YES] };

        App {
            window,
            device,
            command_queue,
            renderer,
            _layer_manager,
            text_system,
        }
    }
}

impl App {
    fn run(mut self, layer_setup: Box<dyn FnOnce(&mut LayerManager)>) {
        // Set up layers
        layer_setup(&mut self._layer_manager);

        // Main render loop
        while self.window.handle_events() {
            self.render_frame();
        }
    }

    fn render_frame(&mut self) {
        // Get the next drawable from the Metal layer
        let drawable = match self.window.metal_layer().next_drawable() {
            Some(drawable) => drawable,
            None => {
                eprintln!("Failed to get next drawable");
                return;
            }
        };

        // Get window size and scale factor
        let size = self.window.size();
        let scale_factor = self.window.scale_factor();

        // Create command buffer
        let command_buffer = self.command_queue.new_command_buffer();

        // Render all layers using the layer manager
        self._layer_manager.render(
            &mut self.renderer,
            &command_buffer,
            &drawable,
            glam::vec2(size.0, size.1),
            &mut self.text_system,
            scale_factor,
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
