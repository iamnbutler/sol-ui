use crate::platform::Window;
use cocoa::base::id;
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

        // Create window
        let window = Window::new(800.0, 600.0, "Toy UI");

        // Create Metal device and command queue
        let device = Device::system_default().expect("No Metal device found");
        let command_queue = device.new_command_queue();

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
        // TODO: Implement actual rendering
        // For now, just clear to a color to verify Metal is working
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
