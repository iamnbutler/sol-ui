use crate::platform::{Window, create_app_menu};
use cocoa::base::{YES, id};
use metal::{CommandQueue, Device};
use objc::{class, msg_send, sel, sel_impl};
use std::sync::Arc;

pub struct App {
    window: Arc<Window>,
    device: Device,
    command_queue: CommandQueue,
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

        // Activate app and bring to front
        let _: () = unsafe { msg_send![ns_app, activateIgnoringOtherApps: YES] };

        Self {
            window,
            device,
            command_queue,
        }
    }

    pub fn run(self) {
        while self.window.handle_events() {
            // TODO: Render frame here
            self.render_frame();
        }
    }

    fn render_frame(&self) {
        // Get the next drawable
        let drawable = match self.window.metal_layer().next_drawable() {
            Some(drawable) => drawable,
            None => return, // No drawable available, skip frame
        };

        // Create command buffer
        let command_buffer = self.command_queue.new_command_buffer();

        // Create render pass descriptor
        let render_pass_descriptor = metal::RenderPassDescriptor::new();

        // Configure color attachment to clear to blue
        let color_attachment = render_pass_descriptor
            .color_attachments()
            .object_at(0)
            .unwrap();
        color_attachment.set_texture(Some(drawable.texture()));
        color_attachment.set_load_action(metal::MTLLoadAction::Clear);
        color_attachment.set_clear_color(metal::MTLClearColor::new(0.0, 0.2, 0.4, 1.0)); // Dark blue
        color_attachment.set_store_action(metal::MTLStoreAction::Store);

        // Create render encoder (even though we're just clearing)
        let render_encoder = command_buffer.new_render_command_encoder(&render_pass_descriptor);
        render_encoder.end_encoding();

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
