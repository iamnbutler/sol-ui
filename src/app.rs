use crate::layer::LayerManager;
use crate::metal_renderer::MetalRenderer;
use crate::platform::{Window, create_app_menu};
use crate::text_system::TextSystem;
use crate::ui::UiContext;

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
    layers: Vec<Box<dyn FnMut(&mut UiContext)>>,
}

pub fn app() -> AppBuilder {
    AppBuilder::new()
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            width: 800.0,
            height: 600.0,
            title: "Toy UI".to_string(),
            layers: Vec::new(),
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

    pub fn layer<F>(mut self, render_fn: F) -> Self
    where
        F: FnMut(&mut UiContext) + 'static,
    {
        self.layers.push(Box::new(render_fn));
        self
    }

    pub fn run(mut self) {
        let layers = std::mem::take(&mut self.layers);
        let app = self.build();
        app.run(layers);
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
    fn run(mut self, mut layers: Vec<Box<dyn FnMut(&mut UiContext)>>) {
        while self.window.handle_events() {
            self.render_frame(&mut layers);
        }
    }

    fn render_frame(&mut self, layers: &mut Vec<Box<dyn FnMut(&mut UiContext)>>) {
        // Get the next drawable from the Metal layer
        let drawable = match self.window.metal_layer().next_drawable() {
            Some(drawable) => drawable,
            None => {
                return; // No drawable available, skip frame
            }
        };

        // Get window size and scale factor
        let size = self.window.size();
        let scale_factor = self.window.scale_factor();

        // Create command buffer
        let command_buffer = self.command_queue.new_command_buffer();

        // Render each layer directly
        for (index, layer_fn) in layers.iter_mut().enumerate() {
            // Create a new UI context for this layer
            let mut ui_context = crate::ui::UiContext::new(glam::vec2(size.0, size.1));

            // Execute the layer's render function
            layer_fn(&mut ui_context);

            // Get the draw list
            let draw_list = ui_context.draw_list();

            // Determine load action and clear color
            // First layer clears, others load existing content
            let (load_action, clear_color) = if index == 0 {
                (
                    metal::MTLLoadAction::Clear,
                    metal::MTLClearColor::new(0.0, 0.0, 0.0, 1.0),
                )
            } else {
                (
                    metal::MTLLoadAction::Load,
                    metal::MTLClearColor::new(0.0, 0.0, 0.0, 0.0),
                )
            };

            // Render this layer's draw list
            self.renderer.render_draw_list(
                draw_list,
                &command_buffer,
                &drawable,
                size,
                scale_factor,
                &mut self.text_system,
                load_action,
                clear_color,
            );
        }

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
