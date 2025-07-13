use cocoa::{
    base::{NO, YES, id, nil},
    foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize, NSString},
};
use core_graphics::geometry::CGSize;

use crate::layer::{InputEvent, MouseButton};
use metal::MetalLayer;
use objc::{
    class,
    declare::ClassDecl,
    msg_send,
    runtime::{BOOL, Class, Object, Sel},
    sel, sel_impl,
};
use std::{cell::RefCell, ptr, sync::Arc};

unsafe fn ns_string(string: &str) -> id {
    let str: id = unsafe { NSString::alloc(nil).init_str(string) };
    unsafe { msg_send![str, autorelease] }
}

// #[repr(C)]
// pub struct NSWindow {
//     _private: [u8; 0],
// }

// #[repr(C)]
// pub struct NSView {
//     _private: [u8; 0],
// }

#[repr(C)]
pub struct NSApplication {
    _private: [u8; 0],
}

// Window delegate to handle events
static mut WINDOW_DELEGATE_CLASS: *const Class = ptr::null();
static mut VIEW_CLASS: *const Class = ptr::null();

thread_local! {
    static PENDING_EVENTS: RefCell<Vec<InputEvent>> = RefCell::new(Vec::new());
}

pub struct Window {
    ns_window: *mut Object,
    // ns_view: *mut Object,
    metal_layer: MetalLayer,
}

impl Window {
    pub fn new(width: f64, height: f64, title: &str, device: &metal::Device) -> Arc<Self> {
        unsafe { ensure_classes_initialized() };

        let _pool = unsafe { NSAutoreleasePool::new(nil) };

        // Create window
        let ns_window: *mut Object = unsafe { msg_send![class!(NSWindow), alloc] };
        let content_rect = NSRect::new(
            NSPoint::new(100.0, 100.0),
            NSSize {
                width: width,
                height: height,
            },
        );
        let style_mask: u64 = 15; // Titled | Closable | Miniaturizable | Resizable
        let backing_store: u64 = 2; // Buffered

        let ns_window: *mut Object = unsafe {
            msg_send![
                ns_window,
                initWithContentRect:content_rect
                styleMask:style_mask
                backing:backing_store
                defer:NO
            ]
        };

        // Set title
        let title = unsafe { ns_string(title) };
        let _: () = unsafe { msg_send![ns_window, setTitle: title] };

        // Create delegate
        let delegate: *mut Object = unsafe { msg_send![WINDOW_DELEGATE_CLASS, new] };
        let _: () = unsafe { msg_send![ns_window, setDelegate: delegate] };

        // Create metal view
        // let ns_view: *mut Object = unsafe { msg_send![VIEW_CLASS, alloc] };
        // let ns_view: *mut Object = unsafe { msg_send![ns_view, initWithFrame: content_rect] };

        // Set up Metal layer
        let layer = MetalLayer::new();
        layer.set_device(device);
        layer.set_pixel_format(metal::MTLPixelFormat::BGRA8Unorm);

        // Get the actual backing scale factor from the window
        let scale_factor: f64 = unsafe { msg_send![ns_window, backingScaleFactor] };
        layer.set_contents_scale(scale_factor);

        layer.set_opaque(true);
        layer.set_presents_with_transaction(false);
        layer.set_framebuffer_only(true);
        layer.set_drawable_size(CGSize::new(width * scale_factor, height * scale_factor));
        let _: () = unsafe { msg_send![layer.as_ref(), setFrame: content_rect] };

        // Configure additional layer properties for better performance
        unsafe {
            let _: () = msg_send![layer.as_ref(), setAllowsNextDrawableTimeout: NO];
            let _: () = msg_send![layer.as_ref(), setNeedsDisplayOnBoundsChange: YES];
        }

        // Set the layer on the view
        // let layer_ref = layer.as_ref() as *const _ as *mut c_void;
        // let _: () = unsafe { msg_send![ns_view, setLayer: layer_ref] };
        // let _: () = unsafe { msg_send![ns_view, setWantsLayer: YES] };

        // Set view as content view
        // let _: () = unsafe { msg_send![ns_window, setContentView: ns_view] };

        // Center and show window
        let _: () = unsafe { msg_send![ns_window, center] };
        let _: () = unsafe { msg_send![ns_window, makeKeyAndOrderFront: nil] };

        // Enable mouse moved events
        let _: () = unsafe { msg_send![ns_window, setAcceptsMouseMovedEvents: YES] };

        Arc::new(Window {
            ns_window,
            // ns_view,
            metal_layer: layer,
        })
    }

    pub fn metal_layer(&self) -> &MetalLayer {
        &self.metal_layer
    }

    pub fn size(&self) -> (f32, f32) {
        let frame: NSRect = unsafe { msg_send![self.ns_window, contentLayoutRect] };
        (frame.size.width as f32, frame.size.height as f32)
    }

    pub fn handle_events(&self) -> bool {
        self.handle_events_internal(true)
    }

    pub fn handle_events_non_blocking(&self) -> bool {
        self.handle_events_internal(false)
    }

    pub fn get_pending_input_events(&self) -> Vec<InputEvent> {
        PENDING_EVENTS.with(|events| {
            let mut events_ref = events.borrow_mut();
            let result = events_ref.clone();
            events_ref.clear();
            result
        })
    }

    fn handle_events_internal(&self, blocking: bool) -> bool {
        let app = unsafe { NSApplication::shared() };

        loop {
            let event: *mut Object = unsafe {
                if blocking {
                    msg_send![
                        app,
                        nextEventMatchingMask: !0
                        untilDate: nil
                        inMode: ns_string("kCFRunLoopDefaultMode")
                        dequeue: YES
                    ]
                } else {
                    // Non-blocking: return immediately if no events
                    msg_send![
                        app,
                        nextEventMatchingMask: !0
                        untilDate: {
                            let past: *mut Object = msg_send![class!(NSDate), distantPast];
                            past
                        }
                        inMode: ns_string("kCFRunLoopDefaultMode")
                        dequeue: YES
                    ]
                }
            };

            if event.is_null() {
                break;
            }

            // Get event type
            let event_type: u64 = unsafe { msg_send![event, type] };

            // Handle different event types
            match event_type {
                1 => self.handle_mouse_down(event),  // NSEventTypeLeftMouseDown
                2 => self.handle_mouse_up(event),    // NSEventTypeLeftMouseUp
                3 => self.handle_mouse_down(event),  // NSEventTypeRightMouseDown
                4 => self.handle_mouse_up(event),    // NSEventTypeRightMouseUp
                5 => self.handle_mouse_moved(event), // NSEventTypeMouseMoved
                6 => self.handle_mouse_moved(event), // NSEventTypeLeftMouseDragged
                7 => self.handle_mouse_moved(event), // NSEventTypeRightMouseDragged
                _ => {}
            }

            let _: () = unsafe { msg_send![app, sendEvent: event] };
        }

        // Check if window is still valid
        let is_visible: BOOL = unsafe { msg_send![self.ns_window, isVisible] };
        is_visible == YES
    }

    pub fn scale_factor(&self) -> f32 {
        let scale: f64 = unsafe { msg_send![self.ns_window, backingScaleFactor] };
        scale as f32
    }

    fn handle_mouse_moved(&self, event: *mut Object) {
        let location = self.get_mouse_location(event);
        PENDING_EVENTS.with(|events| {
            events.borrow_mut().push(InputEvent::MouseMove {
                position: glam::Vec2::new(location.0 as f32, location.1 as f32),
            });
        });
    }

    fn handle_mouse_down(&self, event: *mut Object) {
        let location = self.get_mouse_location(event);
        let event_type: u64 = unsafe { msg_send![event, type] };
        let button = if event_type == 1 {
            MouseButton::Left
        } else if event_type == 3 {
            MouseButton::Right
        } else {
            MouseButton::Middle
        };

        PENDING_EVENTS.with(|events| {
            events.borrow_mut().push(InputEvent::MouseDown {
                position: glam::Vec2::new(location.0 as f32, location.1 as f32),
                button,
            });
        });
    }

    fn handle_mouse_up(&self, event: *mut Object) {
        let location = self.get_mouse_location(event);
        let event_type: u64 = unsafe { msg_send![event, type] };
        let button = if event_type == 2 {
            MouseButton::Left
        } else if event_type == 4 {
            MouseButton::Right
        } else {
            MouseButton::Middle
        };

        PENDING_EVENTS.with(|events| {
            events.borrow_mut().push(InputEvent::MouseUp {
                position: glam::Vec2::new(location.0 as f32, location.1 as f32),
                button,
            });
        });
    }

    fn get_mouse_location(&self, event: *mut Object) -> (f64, f64) {
        // Get location in window coordinates
        let window_point: NSPoint = unsafe { msg_send![event, locationInWindow] };

        // Get content view bounds
        let content_view: *mut Object = unsafe { msg_send![self.ns_window, contentView] };
        let bounds: NSRect = unsafe { msg_send![content_view, bounds] };

        // Flip Y coordinate (macOS has origin at bottom-left, we want top-left)
        let x = window_point.x;
        let y = bounds.size.height - window_point.y;

        (x, y)
    }
}

unsafe fn ensure_classes_initialized() {
    if unsafe { WINDOW_DELEGATE_CLASS.is_null() } {
        unsafe { create_window_delegate_class() };
    }
    if unsafe { VIEW_CLASS.is_null() } {
        unsafe { create_view_class() };
    }
}

unsafe fn create_window_delegate_class() {
    let superclass = class!(NSObject);
    let mut decl = ClassDecl::new("ToyUIWindowDelegate", superclass).unwrap();

    // Add windowShouldClose method
    extern "C" fn window_should_close(_: &Object, _: Sel, _: *mut Object) -> BOOL {
        YES
    }

    unsafe {
        decl.add_method(
            sel!(windowShouldClose:),
            window_should_close as extern "C" fn(&Object, Sel, *mut Object) -> BOOL,
        );
    }

    // Add windowWillClose method
    extern "C" fn window_will_close(_: &Object, _: Sel, _: *mut Object) {
        let app = unsafe { NSApplication::shared() };
        let _: () = unsafe { msg_send![app, terminate: nil] };
    }

    unsafe {
        decl.add_method(
            sel!(windowWillClose:),
            window_will_close as extern "C" fn(&Object, Sel, *mut Object),
        );
    }

    unsafe {
        WINDOW_DELEGATE_CLASS = decl.register();
    }
}

unsafe fn create_view_class() {
    let superclass = class!(NSView);
    let mut decl = ClassDecl::new("ToyUIMetalView", superclass).unwrap();

    // Make the view layer-backed
    extern "C" fn wants_layer(_: &Object, _: Sel) -> BOOL {
        YES
    }

    extern "C" fn layer_class(_: &Class, _: Sel) -> *const Class {
        class!(CAMetalLayer)
    }

    unsafe {
        decl.add_method(
            sel!(wantsLayer),
            wants_layer as extern "C" fn(&Object, Sel) -> BOOL,
        );
    }

    unsafe {
        decl.add_class_method(
            sel!(layerClass),
            layer_class as extern "C" fn(&Class, Sel) -> *const Class,
        );
    }

    // Add mouse tracking
    extern "C" fn update_tracking_areas(this: &mut Object, _: Sel) {
        unsafe {
            // Call super
            let superclass = class!(NSView);
            let _: () = msg_send![super(this, superclass), updateTrackingAreas];

            // Remove existing tracking areas
            let tracking_areas: *mut Object = msg_send![this, trackingAreas];
            let count: usize = msg_send![tracking_areas, count];
            for i in 0..count {
                let area: *mut Object = msg_send![tracking_areas, objectAtIndex: i];
                let _: () = msg_send![this, removeTrackingArea: area];
            }

            // Add new tracking area
            let bounds: NSRect = msg_send![this, bounds];
            let options: u64 = 0x01 | 0x02 | 0x20 | 0x100; // NSTrackingMouseEnteredAndExited | NSTrackingMouseMoved | NSTrackingActiveInKeyWindow | NSTrackingInVisibleRect
            let tracking_area: *mut Object = msg_send![class!(NSTrackingArea), alloc];
            let tracking_area: *mut Object = msg_send![
                tracking_area,
                initWithRect:bounds
                options:options
                owner:this as *const Object
                userInfo:nil
            ];
            let _: () = msg_send![this, addTrackingArea: tracking_area];
        }
    }

    // Mouse entered view
    extern "C" fn mouse_entered(_: &Object, _: Sel, _: *mut Object) {
        // Mouse entered the view
    }

    // Mouse exited view
    extern "C" fn mouse_exited(_: &Object, _: Sel, _: *mut Object) {
        PENDING_EVENTS.with(|events| {
            events.borrow_mut().push(InputEvent::MouseLeave);
        });
    }

    unsafe {
        decl.add_method(
            sel!(updateTrackingAreas),
            update_tracking_areas as extern "C" fn(&mut Object, Sel),
        );
        decl.add_method(
            sel!(mouseEntered:),
            mouse_entered as extern "C" fn(&Object, Sel, *mut Object),
        );
        decl.add_method(
            sel!(mouseExited:),
            mouse_exited as extern "C" fn(&Object, Sel, *mut Object),
        );
    }

    unsafe {
        VIEW_CLASS = decl.register();
    }
}

// Helper to get shared NSApplication instance
impl NSApplication {
    unsafe fn shared() -> *mut Object {
        msg_send![class!(NSApplication), sharedApplication]
    }
}
