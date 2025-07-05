use cocoa::base::{NO, YES, id, nil};
use cocoa::foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize, NSString};
use core_graphics::geometry::CGSize;

use metal::MetalLayer;
use objc::declare::ClassDecl;
use objc::runtime::{BOOL, Class, Object, Sel};
use objc::{class, msg_send, sel, sel_impl};
use std::ffi::c_void;
use std::ptr;
use std::sync::Arc;

// Helper function to create NSString
unsafe fn ns_string(string: &str) -> id {
    let str: id = unsafe { NSString::alloc(nil).init_str(string) };
    unsafe { msg_send![str, autorelease] }
}

#[repr(C)]
pub struct NSWindow {
    _private: [u8; 0],
}

#[repr(C)]
pub struct NSView {
    _private: [u8; 0],
}

#[repr(C)]
pub struct NSApplication {
    _private: [u8; 0],
}

// Window delegate to handle events
static mut WINDOW_DELEGATE_CLASS: *const Class = ptr::null();
static mut VIEW_CLASS: *const Class = ptr::null();

pub struct Window {
    ns_window: *mut Object,
    ns_view: *mut Object,
    metal_layer: MetalLayer,
}

impl Window {
    pub fn new(width: f64, height: f64, title: &str, device: &metal::Device) -> Arc<Self> {
        // Ensure classes are initialized
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
        let ns_view: *mut Object = unsafe { msg_send![VIEW_CLASS, alloc] };
        let ns_view: *mut Object = unsafe { msg_send![ns_view, initWithFrame: content_rect] };

        // Set up Metal layer
        let layer = MetalLayer::new();
        layer.set_device(device);
        layer.set_pixel_format(metal::MTLPixelFormat::BGRA8Unorm);
        layer.set_contents_scale(2.0); // Retina display
        layer.set_opaque(true);
        layer.set_presents_with_transaction(false);
        layer.set_framebuffer_only(true);
        layer.set_drawable_size(CGSize::new(width * 2.0, height * 2.0)); // Account for retina
        let _: () = unsafe { msg_send![layer.as_ref(), setFrame: content_rect] };

        // Configure additional layer properties for better performance
        unsafe {
            let _: () = msg_send![layer.as_ref(), setAllowsNextDrawableTimeout: NO];
            let _: () = msg_send![layer.as_ref(), setNeedsDisplayOnBoundsChange: YES];
        }

        // Set the layer on the view
        let layer_ref = layer.as_ref() as *const _ as *mut c_void;
        let _: () = unsafe { msg_send![ns_view, setLayer: layer_ref] };
        let _: () = unsafe { msg_send![ns_view, setWantsLayer: YES] };

        // Set view as content view
        let _: () = unsafe { msg_send![ns_window, setContentView: ns_view] };

        // Center and show window
        let _: () = unsafe { msg_send![ns_window, center] };
        let _: () = unsafe { msg_send![ns_window, makeKeyAndOrderFront: nil] };

        Arc::new(Window {
            ns_window,
            ns_view,
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
        let app = unsafe { NSApplication::shared() };

        loop {
            let event: *mut Object = unsafe {
                msg_send![
                    app,
                    nextEventMatchingMask: !0
                    untilDate: nil
                    inMode: ns_string("kCFRunLoopDefaultMode")
                    dequeue: YES
                ]
            };

            if event.is_null() {
                break;
            }

            let _: () = unsafe { msg_send![app, sendEvent: event] };
        }

        // Check if window is still valid
        let is_visible: BOOL = unsafe { msg_send![self.ns_window, isVisible] };
        is_visible == YES
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
