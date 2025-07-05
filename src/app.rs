use crate::layer::{LayerManager, LayerOptions};
use crate::platform::{Window, create_app_menu};
use crate::renderer::MetalRenderer;
use crate::text::TextSystem;
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
    layer_manager: LayerManager,
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
        let layer_manager = LayerManager::new();

        // Create text system
        let text_system = TextSystem::new(&device).expect("Failed to create text system");

        // Activate app and bring to front
        let _: () = unsafe { msg_send![ns_app, activateIgnoringOtherApps: YES] };

        App {
            window,
            device,
            command_queue,
            renderer,
            layer_manager,
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

        // Get window size
        let size = self.window.size();

        // Clear layers from previous frame
        self.layer_manager.clear();

        // For now, we'll combine all layers into one UI context
        // TODO: In the future, each layer should have its own context and render pass
        let mut ui_context = crate::ui::UiContext::new(glam::vec2(size.0, size.1));

        // Execute all layer render functions
        for layer in layers.iter_mut() {
            layer(&mut ui_context);
        }

        // Create command buffer
        let command_buffer = self.command_queue.new_command_buffer();

        // For now, render directly without the layer system
        // TODO: Fix layer system to support borrowed closures or redesign
        let draw_list = ui_context.draw_list();

        // Render with clear
        self.renderer.render_draw_list(
            draw_list,
            &command_buffer,
            &drawable,
            size,
            &mut self.text_system,
            metal::MTLLoadAction::Clear,
            metal::MTLClearColor::new(0.8, 0.8, 0.8, 1.0),
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
