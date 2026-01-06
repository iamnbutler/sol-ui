use crate::{
    entity::EntityStore,
    layer::LayerManager,
    platform::{create_app_menu, mac::metal_renderer::MetalRenderer, MenuBar, Window},
    task::{TaskRunner, clear_task_runner, set_task_runner},
    text_system::TextSystem,
};
use std::time::Instant;
use tracing::{debug, info, info_span};

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
    entity_store: EntityStore,
    task_runner: TaskRunner,
    last_window_size: Option<(f32, f32)>,
    animation_frame_requested: bool,
    start_time: Instant,
}

pub struct AppBuilder {
    width: f64,
    height: f64,
    title: String,
    layer_setup: Box<dyn FnOnce(&mut LayerManager)>,
    menu_setup: Option<Box<dyn FnOnce(&str) -> MenuBar>>,
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
            menu_setup: None,
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

    /// Configure a custom menu bar for the application.
    ///
    /// The closure receives the app title and should return a configured MenuBar.
    /// If not called, a default app menu with just Quit is created.
    ///
    /// # Example
    /// ```ignore
    /// app()
    ///     .title("My App")
    ///     .with_menu_bar(|title| {
    ///         MenuBar::new(title)
    ///             .with_app_menu()
    ///             .menu(Menu::new("File")
    ///                 .item(MenuItem::action("New")
    ///                     .shortcut(KeyboardShortcut::cmd("n"))
    ///                     .build()))
    ///             .with_edit_menu()
    ///     })
    ///     .run();
    /// ```
    pub fn with_menu_bar<F>(mut self, setup: F) -> Self
    where
        F: FnOnce(&str) -> MenuBar + 'static,
    {
        self.menu_setup = Some(Box::new(setup));
        self
    }

    pub fn run(mut self) {
        let layer_setup = std::mem::replace(&mut self.layer_setup, Box::new(|_| {}));
        let app = self.build();
        app.run(layer_setup);
    }

    fn build(self) -> App {
        let _build_span = info_span!("app_build").entered();
        let build_start = Instant::now();

        // Initialize NSApplication
        let start = Instant::now();
        info!("Initializing NSApplication");
        let ns_app: id = unsafe { msg_send![class!(NSApplication), sharedApplication] };
        let _: () = unsafe { msg_send![ns_app, setActivationPolicy: 0] }; // NSApplicationActivationPolicyRegular
        info!("NSApplication initialized in {:?}", start.elapsed());

        // Create app menu
        let start = Instant::now();
        if let Some(menu_setup) = self.menu_setup {
            let menu_bar = menu_setup(&self.title);
            menu_bar.build();
        } else {
            create_app_menu();
        }
        info!("App menu created in {:?}", start.elapsed());

        // Create Metal device and command queue
        let start = Instant::now();
        info!("Creating Metal device and command queue");
        let device = Device::system_default().expect("No Metal device found");
        let command_queue = device.new_command_queue();
        info!(
            "Metal device and command queue created in {:?}",
            start.elapsed()
        );

        // Create window
        let start = Instant::now();
        info!("Creating window: {}x{}", self.width, self.height);
        let window = Window::new(self.width, self.height, &self.title, &device);
        info!("Window created in {:?}", start.elapsed());

        // Create and initialize renderer
        let start = Instant::now();
        info!("Creating and initializing Metal renderer");
        let mut renderer = MetalRenderer::new(device.clone());
        if let Err(e) = renderer.initialize() {
            panic!("Failed to initialize renderer: {}", e);
        }
        info!("Metal renderer initialized in {:?}", start.elapsed());

        // Create layer manager
        let start = Instant::now();
        let _layer_manager = LayerManager::new();
        info!("Layer manager created in {:?}", start.elapsed());

        // Create text system
        let start = Instant::now();
        info!("Creating text system");
        let text_system = TextSystem::new(&device).expect("Failed to create text system");
        info!("Text system created in {:?}", start.elapsed());

        // Activate app and bring to front
        let start = Instant::now();
        let _: () = unsafe { msg_send![ns_app, activateIgnoringOtherApps: YES] };
        info!("App activated in {:?}", start.elapsed());

        info!("Total app build time: {:?}", build_start.elapsed());

        // Create entity store
        let entity_store = EntityStore::new();

        // Create task runner for background tasks
        let task_runner = TaskRunner::new();

        App {
            window,
            device,
            command_queue,
            renderer,
            layer_manager: _layer_manager,
            text_system,
            entity_store,
            task_runner,
            last_window_size: None,
            animation_frame_requested: false,
            start_time: Instant::now(),
        }
    }
}

impl App {
    fn run(mut self, layer_setup: Box<dyn FnOnce(&mut LayerManager)>) {
        let _run_span = info_span!("app_run").entered();

        // Set up layers
        {
            let _setup_span = info_span!("layer_setup_execution").entered();
            let start = Instant::now();
            info!("Setting up layers");
            layer_setup(&mut self.layer_manager);
            info!("Layer setup complete in {:?}", start.elapsed());
        }

        // Main render loop
        info!("Starting main render loop");
        let mut frame_count = 0u64;
        let loop_start = Instant::now();
        let mut first_frame_completed = false;

        loop {
            // Set task runner for this frame (allows spawn_task to work)
            set_task_runner(&mut self.task_runner);

            // Poll for completed background tasks
            let completed_tasks = self.task_runner.poll();
            if completed_tasks > 0 {
                debug!("Processed {} completed background tasks", completed_tasks);
            }

            // Use non-blocking event handling if animation frame was requested
            // or if there are pending background tasks
            let should_continue = if self.animation_frame_requested || self.task_runner.has_pending() {
                self.window.handle_events_non_blocking()
            } else {
                self.window.handle_events()
            };

            if !should_continue {
                clear_task_runner();
                break;
            }

            // Process input events
            let input_events = self.window.get_pending_input_events();
            for event in &input_events {
                self.layer_manager.handle_input(event);
            }

            let frame_start = Instant::now();
            let _frame_span = info_span!("frame", frame_number = frame_count).entered();
            self.render_frame();
            let frame_time = frame_start.elapsed();

            if !first_frame_completed {
                info!(
                    "First frame rendered in {:?} (total time since start: {:?})",
                    frame_time,
                    loop_start.elapsed()
                );
                first_frame_completed = true;
            }

            frame_count += 1;

            if frame_count % 100 == 0 {
                debug!(
                    "Rendered {} frames, last frame took {:?}",
                    frame_count, frame_time
                );
            }

            // Clear task runner at end of frame
            clear_task_runner();

            // Frame rate limiting: target 120 FPS (8.33ms per frame)
            if self.animation_frame_requested || self.task_runner.has_pending() {
                const TARGET_FRAME_TIME: std::time::Duration =
                    std::time::Duration::from_micros(8_333);
                if let Some(sleep_duration) = TARGET_FRAME_TIME.checked_sub(frame_time) {
                    std::thread::sleep(sleep_duration);
                }
            }
        }
    }

    fn render_frame(&mut self) {
        let frame_start = Instant::now();

        // Clear text system frame caches
        self.text_system.begin_frame();

        // Check if window size changed
        let current_size = self.window.size();
        if let Some(last_size) = self.last_window_size {
            if last_size != current_size {
                debug!("Window resized from {:?} to {:?}", last_size, current_size);
                // Mark all layers for rebuild on resize
                self.layer_manager.invalidate_all();
            }
        }
        self.last_window_size = Some(current_size);

        // Get the next drawable from the Metal layer
        let drawable = {
            let start = Instant::now();
            let _drawable_span = info_span!("get_next_drawable").entered();
            match self.window.metal_layer().next_drawable() {
                Some(drawable) => {
                    debug!("Next drawable acquired in {:?}", start.elapsed());
                    drawable
                }
                None => {
                    eprintln!("Failed to get next drawable");
                    return;
                }
            }
        };

        // Get window size and scale factor
        let start = Instant::now();
        let size = self.window.size();
        let scale_factor = self.window.scale_factor();
        debug!("Window size/scale retrieved in {:?}", start.elapsed());

        // Create command buffer
        let command_buffer = {
            let start = Instant::now();
            let _cmd_span = info_span!("create_command_buffer").entered();
            let buffer = self.command_queue.new_command_buffer();
            debug!("Command buffer created in {:?}", start.elapsed());
            buffer
        };

        // Render all layers using the layer manager
        {
            let _render_span = info_span!("layer_manager_render").entered();

            // Calculate elapsed time since app start for animations
            let elapsed_time = self.start_time.elapsed().as_secs_f32();

            // Render all layers and check if any requested animation frame
            self.animation_frame_requested = self.layer_manager.render(
                &mut self.renderer,
                &command_buffer,
                &drawable,
                (size.0, size.1).into(),
                &mut self.text_system,
                &mut self.entity_store,
                scale_factor,
                elapsed_time,
            );
        }

        // Present drawable and commit
        {
            let start = Instant::now();
            let _present_span = info_span!("present_and_commit").entered();
            command_buffer.present_drawable(&drawable);
            command_buffer.commit();
            debug!("Present and commit completed in {:?}", start.elapsed());
        }

        debug!("Total frame time: {:?}", frame_start.elapsed());
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
