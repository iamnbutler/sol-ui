//! iOS platform implementation for sol-ui
//!
//! This module provides iOS-specific windowing, input handling, and Metal rendering support.
//! Designed to work on jailbroken iOS devices (iPhone 8, iOS 11-14).

use core_foundation::base::TCFType;
use core_foundation::string::CFString;
use core_graphics::geometry::{CGPoint, CGRect, CGSize};
use metal::{CommandQueue, Device, MetalLayer};
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Protocol, Sel, BOOL, YES, NO};
use objc::{class, msg_send, sel, sel_impl};
use std::cell::RefCell;
use std::ffi::c_void;
use std::ptr;
use std::sync::Arc;

use crate::layer::InputEvent;
use crate::platform::mac::metal_renderer::MetalRenderer;

// Static class references
static mut VIEW_CLASS: *const Class = ptr::null();
static mut WINDOW_DELEGATE_CLASS: *const Class = ptr::null();

thread_local! {
    static PENDING_EVENTS: RefCell<Vec<InputEvent>> = RefCell::new(Vec::new());
}

// ================================================================================================
// MARK: - Window Implementation
// ================================================================================================

/// iOS Window implementation using UIWindow and UIView with Metal layer
pub struct Window {
    ui_window: *mut Object,
    metal_view: *mut Object,
    metal_layer: MetalLayer,
    device: Device,
    command_queue: CommandQueue,
    renderer: MetalRenderer,
    size: CGSize,
}

impl Window {
    /// Create a new iOS window with Metal rendering support
    pub fn new(title: &str, width: f64, height: f64) -> Self {
        unsafe {
            // Ensure classes are registered
            ensure_classes_registered();

            // Get the main screen bounds
            let screen: *mut Object = msg_send![class!(UIScreen), mainScreen];
            let bounds: CGRect = msg_send![screen, bounds];

            // Use provided dimensions or fall back to screen size
            let window_rect = if width > 0.0 && height > 0.0 {
                CGRect::new(&CGPoint::new(0.0, 0.0), &CGSize::new(width, height))
            } else {
                bounds
            };

            // Create UIWindow
            let ui_window: *mut Object = msg_send![class!(UIWindow), alloc];
            let ui_window: *mut Object = msg_send![ui_window, initWithFrame:window_rect];

            // Create our custom Metal view
            let metal_view = Self::create_metal_view(window_rect);

            // Add view to window
            let _: () = msg_send![ui_window, addSubview:metal_view];

            // Set up Metal
            let device = Device::system_default()
                .expect("Metal is required but not available on this device");
            let command_queue = device.new_command_queue();

            // Get the CAMetalLayer from our view
            let layer: *mut Object = msg_send![metal_view, layer];
            let metal_layer = MetalLayer::from_ptr(layer as *mut _);

            // Configure the Metal layer
            let _: () = msg_send![layer, setDevice:device.as_ptr()];
            let _: () = msg_send![layer, setPixelFormat:80]; // MTLPixelFormatBGRA8Unorm = 80
            let _: () = msg_send![layer, setFramebufferOnly:NO];

            // Handle retina displays
            let scale: f64 = msg_send![screen, scale];
            let _: () = msg_send![layer, setContentsScale:scale];

            // Set drawable size
            let drawable_size = CGSize::new(window_rect.size.width * scale,
                                           window_rect.size.height * scale);
            let _: () = msg_send![layer, setDrawableSize:drawable_size];

            // Create renderer
            let renderer = MetalRenderer::new(&device);

            // Make window visible
            let _: () = msg_send![ui_window, makeKeyAndVisible];

            // Set window background color
            let white_color: *mut Object = msg_send![class!(UIColor), whiteColor];
            let _: () = msg_send![ui_window, setBackgroundColor:white_color];

            Window {
                ui_window,
                metal_view,
                metal_layer,
                device,
                command_queue,
                renderer,
                size: window_rect.size,
            }
        }
    }

    /// Create a custom UIView subclass with Metal layer support
    unsafe fn create_metal_view(frame: CGRect) -> *mut Object {
        let view: *mut Object = msg_send![VIEW_CLASS, alloc];
        let view: *mut Object = msg_send![view, initWithFrame:frame];

        // Store window pointer in view for callbacks
        let window_ptr = Box::into_raw(Box::new(view)) as *mut c_void;
        (*view).set_ivar("sol_ui_window", window_ptr);

        view
    }

    pub fn size(&self) -> (f64, f64) {
        (self.size.width, self.size.height)
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn command_queue(&self) -> &CommandQueue {
        &self.command_queue
    }

    pub fn metal_layer(&self) -> &MetalLayer {
        &self.metal_layer
    }

    pub fn renderer(&mut self) -> &mut MetalRenderer {
        &mut self.renderer
    }

    /// Process pending input events
    pub fn process_events(&mut self) -> Vec<InputEvent> {
        PENDING_EVENTS.with(|events| {
            let mut pending = events.borrow_mut();
            let result = pending.clone();
            pending.clear();
            result
        })
    }

    /// Run the main event loop (iOS-specific)
    pub fn run_event_loop<F>(&mut self, mut update_callback: F)
    where
        F: FnMut(&mut Window) -> bool + 'static,
    {
        // test_todo!("Implement iOS run loop - requires UIApplication integration")
        // For now, we'll use a simple polling loop for jailbroken environments
        unsafe {
            let app: *mut Object = msg_send![class!(UIApplication), sharedApplication];

            loop {
                // Create autorelease pool for this iteration
                let pool: *mut Object = msg_send![class!(NSAutoreleasePool), new];

                // Process any pending UI events
                let event: *mut Object = msg_send![app,
                    nextEventMatchingMask:0xFFFFFFFF
                    untilDate:ptr::null_mut::<Object>()
                    inMode:NSString_from_str("NSDefaultRunLoopMode")
                    dequeue:YES
                ];

                if event != ptr::null_mut() {
                    let _: () = msg_send![app, sendEvent:event];
                }

                // Call user update
                if !update_callback(self) {
                    let _: () = msg_send![pool, release];
                    break;
                }

                // Release autorelease pool
                let _: () = msg_send![pool, release];

                // Small sleep to prevent CPU spinning
                std::thread::sleep(std::time::Duration::from_millis(16)); // ~60 FPS
            }
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            let _: () = msg_send![self.metal_view, release];
            let _: () = msg_send![self.ui_window, release];
        }
    }
}

// ================================================================================================
// MARK: - Touch Input Handling
// ================================================================================================

/// Convert UITouch to our InputEvent
unsafe fn process_touches(touches: *mut Object, view: *mut Object, event_type: TouchEventType) {
    // test_todo!("Implement touch event processing with UITouch set iteration")

    // For now, simulate single touch
    let all_touches: *mut Object = msg_send![touches, allObjects];
    let count: usize = msg_send![all_touches, count];

    for i in 0..count {
        let touch: *mut Object = msg_send![all_touches, objectAtIndex:i];
        let location: CGPoint = msg_send![touch, locationInView:view];

        // Use touch pointer as unique ID
        let touch_id = touch as usize;

        let event = match event_type {
            TouchEventType::Begin => InputEvent::TouchDown {
                position: glam::Vec2::new(location.x as f32, location.y as f32),
                id: touch_id,
            },
            TouchEventType::Move => InputEvent::TouchMove {
                position: glam::Vec2::new(location.x as f32, location.y as f32),
                id: touch_id,
            },
            TouchEventType::End => InputEvent::TouchUp {
                position: glam::Vec2::new(location.x as f32, location.y as f32),
                id: touch_id,
            },
            TouchEventType::Cancel => InputEvent::TouchCancel {
                id: touch_id,
            },
        };

        PENDING_EVENTS.with(|events| {
            events.borrow_mut().push(event);
        });
    }
}

#[derive(Debug, Clone, Copy)]
enum TouchEventType {
    Begin,
    Move,
    End,
    Cancel,
}

// ================================================================================================
// MARK: - Objective-C Class Registration
// ================================================================================================

unsafe fn ensure_classes_registered() {
    if VIEW_CLASS.is_null() {
        register_view_class();
    }
    if WINDOW_DELEGATE_CLASS.is_null() {
        register_window_delegate_class();
    }
}

/// Register custom UIView subclass with Metal layer
unsafe fn register_view_class() {
    let superclass = class!(UIView);
    let mut decl = ClassDecl::new("SolUIMetalView", superclass).unwrap();

    // Add instance variable to store window pointer
    decl.add_ivar::<*mut c_void>("sol_ui_window");

    // Override layerClass to return CAMetalLayer
    extern "C" fn layer_class(_: &Class, _: Sel) -> *const Class {
        unsafe { class!(CAMetalLayer) }
    }
    decl.add_class_method(sel!(layerClass), layer_class as extern "C" fn(&Class, Sel) -> *const Class);

    // Touch event handlers
    extern "C" fn touches_began(this: &mut Object, _: Sel, touches: *mut Object, event: *mut Object) {
        unsafe {
            process_touches(touches, this as *mut Object, TouchEventType::Begin);
        }
    }

    extern "C" fn touches_moved(this: &mut Object, _: Sel, touches: *mut Object, event: *mut Object) {
        unsafe {
            process_touches(touches, this as *mut Object, TouchEventType::Move);
        }
    }

    extern "C" fn touches_ended(this: &mut Object, _: Sel, touches: *mut Object, event: *mut Object) {
        unsafe {
            process_touches(touches, this as *mut Object, TouchEventType::End);
        }
    }

    extern "C" fn touches_cancelled(this: &mut Object, _: Sel, touches: *mut Object, event: *mut Object) {
        unsafe {
            process_touches(touches, this as *mut Object, TouchEventType::Cancel);
        }
    }

    decl.add_method(sel!(touchesBegan:withEvent:), touches_began as extern "C" fn(&mut Object, Sel, *mut Object, *mut Object));
    decl.add_method(sel!(touchesMoved:withEvent:), touches_moved as extern "C" fn(&mut Object, Sel, *mut Object, *mut Object));
    decl.add_method(sel!(touchesEnded:withEvent:), touches_ended as extern "C" fn(&mut Object, Sel, *mut Object, *mut Object));
    decl.add_method(sel!(touchesCancelled:withEvent:), touches_cancelled as extern "C" fn(&mut Object, Sel, *mut Object, *mut Object));

    VIEW_CLASS = decl.register();
}

/// Register window delegate for lifecycle events
unsafe fn register_window_delegate_class() {
    let superclass = class!(NSObject);
    let mut decl = ClassDecl::new("SolUIWindowDelegate", superclass).unwrap();

    // Add UIApplicationDelegate protocol
    // test_todo!("Add UIApplicationDelegate protocol conformance")

    extern "C" fn application_did_finish_launching(_: &Object, _: Sel, _: *mut Object, _: *mut Object) -> BOOL {
        YES
    }

    extern "C" fn application_will_resign_active(_: &Object, _: Sel, _: *mut Object) {
        // Handle app going to background
    }

    extern "C" fn application_did_become_active(_: &Object, _: Sel, _: *mut Object) {
        // Handle app coming to foreground
    }

    decl.add_method(
        sel!(application:didFinishLaunchingWithOptions:),
        application_did_finish_launching as extern "C" fn(&Object, Sel, *mut Object, *mut Object) -> BOOL
    );
    decl.add_method(
        sel!(applicationWillResignActive:),
        application_will_resign_active as extern "C" fn(&Object, Sel, *mut Object)
    );
    decl.add_method(
        sel!(applicationDidBecomeActive:),
        application_did_become_active as extern "C" fn(&Object, Sel, *mut Object)
    );

    WINDOW_DELEGATE_CLASS = decl.register();
}

// ================================================================================================
// MARK: - Utility Functions
// ================================================================================================

/// Helper to create NSString
unsafe fn NSString_from_str(s: &str) -> *mut Object {
    let ns_string: *mut Object = msg_send![class!(NSString), alloc];
    let ns_string: *mut Object = msg_send![ns_string, initWithBytes:s.as_ptr()
                                                              length:s.len()
                                                            encoding:4]; // NSUTF8StringEncoding
    msg_send![ns_string, autorelease]
}

// ================================================================================================
// MARK: - Tests
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_touch_event_type() {
        // Simple test that doesn't require iOS runtime
        let event_type = TouchEventType::Begin;
        match event_type {
            TouchEventType::Begin => assert!(true),
            _ => assert!(false, "Wrong event type"),
        }
    }

    #[test]
    fn test_pending_events_initialization() {
        PENDING_EVENTS.with(|events| {
            assert_eq!(events.borrow().len(), 0);
        });
    }

    // test_todo!("Test Window creation with MobileTestContext")
    // test_todo!("Test Metal layer configuration with MobileTestContext")
    // test_todo!("Test touch event processing with MobileTestContext")
    // test_todo!("Test app lifecycle handling with MobileTestContext")
    // test_todo!("Test memory management and cleanup with MobileTestContext")
}
