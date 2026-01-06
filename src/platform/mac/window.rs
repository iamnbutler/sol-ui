use cocoa::{
    base::{NO, YES, id, nil},
    foundation::{NSAutoreleasePool, NSPoint, NSRect, NSSize, NSString},
};
use core_graphics::geometry::CGSize;

use crate::layer::{InputEvent, Key, Modifiers, MouseButton};
use glam::Vec2;
use metal::MetalLayer;
use objc::{
    class,
    declare::ClassDecl,
    msg_send,
    runtime::{BOOL, Class, Object, Sel},
    sel, sel_impl,
};
use std::{cell::RefCell, ffi::c_void, ptr, sync::Arc};

unsafe fn ns_string(string: &str) -> id {
    let str: id = unsafe { NSString::alloc(nil).init_str(string) };
    unsafe { msg_send![str, autorelease] }
}

#[allow(dead_code)] // This is a false positive
#[repr(C)]
pub struct NSWindow {
    _private: [u8; 0],
}

#[allow(dead_code)] // This is a false positive
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

thread_local! {
    static PENDING_EVENTS: RefCell<Vec<InputEvent>> = RefCell::new(Vec::new());
    static CURRENT_MODIFIERS: RefCell<Modifiers> = RefCell::new(Modifiers::new());
    /// When true, window close is intercepted and WindowCloseRequested event is emitted
    static CLOSE_CONFIRMATION_ENABLED: RefCell<bool> = RefCell::new(false);
    /// Set to true to allow window close to proceed (used after user confirms)
    static CLOSE_CONFIRMED: RefCell<bool> = RefCell::new(false);
}

#[allow(dead_code)] // dead ns_view is a false positive
pub struct Window {
    ns_window: *mut Object,
    ns_view: *mut Object,
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
        let ns_view: *mut Object = unsafe { msg_send![VIEW_CLASS, alloc] };
        let ns_view: *mut Object = unsafe { msg_send![ns_view, initWithFrame: content_rect] };

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
        let layer_ref = layer.as_ref() as *const _ as *mut c_void;
        let _: () = unsafe { msg_send![ns_view, setLayer: layer_ref] };
        let _: () = unsafe { msg_send![ns_view, setWantsLayer: YES] };

        // Set view as content view
        let _: () = unsafe { msg_send![ns_window, setContentView: ns_view] };

        // Center and show window
        let _: () = unsafe { msg_send![ns_window, center] };
        let _: () = unsafe { msg_send![ns_window, makeKeyAndOrderFront: nil] };

        // Enable mouse moved events
        let _: () = unsafe { msg_send![ns_window, setAcceptsMouseMovedEvents: YES] };

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
                1 => self.handle_mouse_down(event),   // NSEventTypeLeftMouseDown
                2 => self.handle_mouse_up(event),     // NSEventTypeLeftMouseUp
                3 => self.handle_mouse_down(event),   // NSEventTypeRightMouseDown
                4 => self.handle_mouse_up(event),     // NSEventTypeRightMouseUp
                5 => self.handle_mouse_moved(event),  // NSEventTypeMouseMoved
                6 => self.handle_mouse_moved(event),  // NSEventTypeLeftMouseDragged
                7 => self.handle_mouse_moved(event),  // NSEventTypeRightMouseDragged
                10 => self.handle_key_down(event),    // NSEventTypeKeyDown
                11 => self.handle_key_up(event),      // NSEventTypeKeyUp
                12 => self.handle_flags_changed(event), // NSEventTypeFlagsChanged
                22 => self.handle_scroll_wheel(event), // NSEventTypeScrollWheel
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

        // Get click count from macOS for double/triple click detection
        let click_count: u64 = unsafe { msg_send![event, clickCount] };

        PENDING_EVENTS.with(|events| {
            events.borrow_mut().push(InputEvent::MouseDown {
                position: glam::Vec2::new(location.0 as f32, location.1 as f32),
                button,
                click_count: click_count as u32,
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

    fn handle_key_down(&self, event: *mut Object) {
        let key_code: u16 = unsafe { msg_send![event, keyCode] };
        let is_repeat: bool = unsafe { msg_send![event, isARepeat] };
        let key = Key::from_keycode(key_code);

        // Get the character from the event
        let character = self.get_character_from_event(event);

        // Get current modifiers
        let modifiers = self.get_modifiers_from_event(event);

        PENDING_EVENTS.with(|events| {
            events.borrow_mut().push(InputEvent::KeyDown {
                key,
                modifiers,
                character,
                is_repeat,
            });
        });
    }

    fn handle_key_up(&self, event: *mut Object) {
        let key_code: u16 = unsafe { msg_send![event, keyCode] };
        let key = Key::from_keycode(key_code);

        // Get current modifiers
        let modifiers = self.get_modifiers_from_event(event);

        PENDING_EVENTS.with(|events| {
            events.borrow_mut().push(InputEvent::KeyUp { key, modifiers });
        });
    }

    fn handle_flags_changed(&self, event: *mut Object) {
        let modifiers = self.get_modifiers_from_event(event);

        // Update the stored modifiers
        CURRENT_MODIFIERS.with(|current| {
            *current.borrow_mut() = modifiers;
        });

        PENDING_EVENTS.with(|events| {
            events.borrow_mut().push(InputEvent::ModifiersChanged { modifiers });
        });
    }

    fn get_modifiers_from_event(&self, event: *mut Object) -> Modifiers {
        let flags: u64 = unsafe { msg_send![event, modifierFlags] };

        // macOS modifier flag masks
        const NS_EVENT_MODIFIER_FLAG_CAPS_LOCK: u64 = 1 << 16;
        const NS_EVENT_MODIFIER_FLAG_SHIFT: u64 = 1 << 17;
        const NS_EVENT_MODIFIER_FLAG_CONTROL: u64 = 1 << 18;
        const NS_EVENT_MODIFIER_FLAG_OPTION: u64 = 1 << 19;
        const NS_EVENT_MODIFIER_FLAG_COMMAND: u64 = 1 << 20;

        Modifiers {
            shift: flags & NS_EVENT_MODIFIER_FLAG_SHIFT != 0,
            ctrl: flags & NS_EVENT_MODIFIER_FLAG_CONTROL != 0,
            alt: flags & NS_EVENT_MODIFIER_FLAG_OPTION != 0,
            cmd: flags & NS_EVENT_MODIFIER_FLAG_COMMAND != 0,
            caps_lock: flags & NS_EVENT_MODIFIER_FLAG_CAPS_LOCK != 0,
        }
    }

    fn get_character_from_event(&self, event: *mut Object) -> Option<char> {
        unsafe {
            // Get the characters string from the event
            let characters: *mut Object = msg_send![event, characters];
            if characters.is_null() {
                return None;
            }

            let length: usize = msg_send![characters, length];
            if length == 0 {
                return None;
            }

            // Get the first character
            let char_code: u16 = msg_send![characters, characterAtIndex: 0usize];

            // Convert to char, filtering out control characters (except for some special cases)
            char::from_u32(char_code as u32).filter(|c| {
                // Allow printable characters and common whitespace
                !c.is_control() || *c == '\t' || *c == '\n' || *c == '\r'
            })
        }
    }

    fn handle_scroll_wheel(&self, event: *mut Object) {
        let location = self.get_mouse_location(event);

        // Get scroll deltas (macOS provides these in "line" units for trackpad)
        let delta_x: f64 = unsafe { msg_send![event, scrollingDeltaX] };
        let delta_y: f64 = unsafe { msg_send![event, scrollingDeltaY] };

        // Check if this is a precise scroll event (trackpad) or imprecise (mouse wheel)
        let is_precise: bool = unsafe { msg_send![event, hasPreciseScrollingDeltas] };

        // For mouse wheel (imprecise), multiply by a factor to get reasonable pixel values
        // For trackpad (precise), the values are already in pixels
        let multiplier = if is_precise { 1.0 } else { 10.0 };

        let delta = glam::Vec2::new(
            (delta_x * multiplier) as f32,
            (delta_y * multiplier) as f32,
        );

        // Only emit event if there's actual scrolling
        if delta.x.abs() > 0.0 || delta.y.abs() > 0.0 {
            PENDING_EVENTS.with(|events| {
                events.borrow_mut().push(InputEvent::ScrollWheel {
                    position: glam::Vec2::new(location.0 as f32, location.1 as f32),
                    delta,
                });
            });
        }
    }

    /// Get the current modifier state
    #[allow(dead_code)]
    pub fn current_modifiers(&self) -> Modifiers {
        CURRENT_MODIFIERS.with(|m| *m.borrow())
    }

    // ===================
    // Window Management
    // ===================

    /// Set the window title
    pub fn set_title(&self, title: &str) {
        let title = unsafe { ns_string(title) };
        let _: () = unsafe { msg_send![self.ns_window, setTitle: title] };
    }

    /// Get the current window title
    pub fn title(&self) -> String {
        unsafe {
            let title: *mut Object = msg_send![self.ns_window, title];
            if title.is_null() {
                return String::new();
            }
            let utf8: *const i8 = msg_send![title, UTF8String];
            if utf8.is_null() {
                return String::new();
            }
            std::ffi::CStr::from_ptr(utf8)
                .to_string_lossy()
                .into_owned()
        }
    }

    /// Minimize the window
    pub fn minimize(&self) {
        let _: () = unsafe { msg_send![self.ns_window, miniaturize: nil] };
    }

    /// Check if the window is minimized
    pub fn is_minimized(&self) -> bool {
        let minimized: BOOL = unsafe { msg_send![self.ns_window, isMiniaturized] };
        minimized == YES
    }

    /// Restore the window from minimized state
    pub fn restore(&self) {
        let _: () = unsafe { msg_send![self.ns_window, deminiaturize: nil] };
    }

    /// Maximize the window (zoom to fill screen)
    pub fn maximize(&self) {
        let _: () = unsafe { msg_send![self.ns_window, zoom: nil] };
    }

    /// Check if the window is maximized (zoomed)
    pub fn is_maximized(&self) -> bool {
        let zoomed: BOOL = unsafe { msg_send![self.ns_window, isZoomed] };
        zoomed == YES
    }

    /// Enter fullscreen mode
    pub fn enter_fullscreen(&self) {
        let is_fullscreen = self.is_fullscreen();
        if !is_fullscreen {
            let _: () = unsafe { msg_send![self.ns_window, toggleFullScreen: nil] };
        }
    }

    /// Exit fullscreen mode
    pub fn exit_fullscreen(&self) {
        let is_fullscreen = self.is_fullscreen();
        if is_fullscreen {
            let _: () = unsafe { msg_send![self.ns_window, toggleFullScreen: nil] };
        }
    }

    /// Toggle fullscreen mode
    pub fn toggle_fullscreen(&self) {
        let _: () = unsafe { msg_send![self.ns_window, toggleFullScreen: nil] };
    }

    /// Check if the window is in fullscreen mode
    pub fn is_fullscreen(&self) -> bool {
        let style_mask: u64 = unsafe { msg_send![self.ns_window, styleMask] };
        // NSWindowStyleMaskFullScreen = 1 << 14
        const NS_FULLSCREEN_MASK: u64 = 1 << 14;
        (style_mask & NS_FULLSCREEN_MASK) != 0
    }

    /// Get the window position (origin of the frame in screen coordinates)
    pub fn position(&self) -> (f32, f32) {
        let frame: NSRect = unsafe { msg_send![self.ns_window, frame] };
        (frame.origin.x as f32, frame.origin.y as f32)
    }

    /// Set the window position
    pub fn set_position(&self, x: f32, y: f32) {
        let origin = NSPoint::new(x as f64, y as f64);
        let _: () = unsafe { msg_send![self.ns_window, setFrameOrigin: origin] };
    }

    /// Set the window size (content area size)
    pub fn set_size(&self, width: f32, height: f32) {
        let size = NSSize::new(width as f64, height as f64);
        let _: () = unsafe { msg_send![self.ns_window, setContentSize: size] };

        // Also update the metal layer drawable size
        let scale_factor: f64 = unsafe { msg_send![self.ns_window, backingScaleFactor] };
        self.metal_layer.set_drawable_size(CGSize::new(
            width as f64 * scale_factor,
            height as f64 * scale_factor,
        ));
    }

    /// Set both position and size at once
    pub fn set_frame(&self, x: f32, y: f32, width: f32, height: f32) {
        let frame = NSRect::new(
            NSPoint::new(x as f64, y as f64),
            NSSize::new(width as f64, height as f64),
        );
        let _: () = unsafe { msg_send![self.ns_window, setFrame: frame display: YES] };

        // Also update the metal layer drawable size
        let scale_factor: f64 = unsafe { msg_send![self.ns_window, backingScaleFactor] };
        self.metal_layer.set_drawable_size(CGSize::new(
            width as f64 * scale_factor,
            height as f64 * scale_factor,
        ));
    }

    /// Get the full window frame (including title bar) in screen coordinates
    pub fn frame(&self) -> (f32, f32, f32, f32) {
        let frame: NSRect = unsafe { msg_send![self.ns_window, frame] };
        (
            frame.origin.x as f32,
            frame.origin.y as f32,
            frame.size.width as f32,
            frame.size.height as f32,
        )
    }

    /// Center the window on the screen
    pub fn center(&self) {
        let _: () = unsafe { msg_send![self.ns_window, center] };
    }

    /// Check if the window has focus (is key window)
    pub fn is_focused(&self) -> bool {
        let is_key: BOOL = unsafe { msg_send![self.ns_window, isKeyWindow] };
        is_key == YES
    }

    /// Request focus for this window
    pub fn focus(&self) {
        let _: () = unsafe { msg_send![self.ns_window, makeKeyAndOrderFront: nil] };
    }

    /// Order the window to the front without making it key
    pub fn order_front(&self) {
        let _: () = unsafe { msg_send![self.ns_window, orderFront: nil] };
    }

    /// Order the window to the back
    pub fn order_back(&self) {
        let _: () = unsafe { msg_send![self.ns_window, orderBack: nil] };
    }

    /// Close the window
    pub fn close(&self) {
        let _: () = unsafe { msg_send![self.ns_window, close] };
    }

    // ===================
    // Close Confirmation
    // ===================

    /// Enable close confirmation. When enabled, attempting to close the window
    /// will emit a WindowCloseRequested event instead of closing immediately.
    /// Call `confirm_close()` to actually close the window.
    pub fn set_close_confirmation(&self, enabled: bool) {
        CLOSE_CONFIRMATION_ENABLED.with(|c| *c.borrow_mut() = enabled);
    }

    /// Check if close confirmation is enabled
    pub fn close_confirmation_enabled(&self) -> bool {
        CLOSE_CONFIRMATION_ENABLED.with(|c| *c.borrow())
    }

    /// Confirm the close and actually close the window.
    /// Only has effect when close confirmation is enabled.
    pub fn confirm_close(&self) {
        CLOSE_CONFIRMED.with(|c| *c.borrow_mut() = true);
        self.close();
    }

    // ===================
    // Position/Size Persistence
    // ===================

    /// Save the current window frame to user defaults with the given key
    pub fn save_frame(&self, key: &str) {
        let key_str = unsafe { ns_string(key) };
        let _: () = unsafe { msg_send![self.ns_window, saveFrameUsingName: key_str] };
    }

    /// Restore the window frame from user defaults with the given key.
    /// Returns true if a saved frame was found and applied.
    pub fn restore_frame(&self, key: &str) -> bool {
        let key_str = unsafe { ns_string(key) };
        let result: BOOL = unsafe { msg_send![self.ns_window, setFrameUsingName: key_str] };
        result == YES
    }

    /// Enable automatic frame saving with the given key.
    /// The window will automatically save its frame when moved/resized.
    pub fn set_frame_autosave_name(&self, name: &str) -> bool {
        let name_str = unsafe { ns_string(name) };
        let result: BOOL = unsafe { msg_send![self.ns_window, setFrameAutosaveName: name_str] };
        result == YES
    }

    /// Get the current frame autosave name, if any
    pub fn frame_autosave_name(&self) -> Option<String> {
        unsafe {
            let name: *mut Object = msg_send![self.ns_window, frameAutosaveName];
            if name.is_null() {
                return None;
            }
            let length: usize = msg_send![name, length];
            if length == 0 {
                return None;
            }
            let utf8: *const i8 = msg_send![name, UTF8String];
            if utf8.is_null() {
                return None;
            }
            Some(
                std::ffi::CStr::from_ptr(utf8)
                    .to_string_lossy()
                    .into_owned(),
            )
        }
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

    // windowShouldClose: - handle close confirmation
    extern "C" fn window_should_close(_: &Object, _: Sel, _: *mut Object) -> BOOL {
        let confirmation_enabled = CLOSE_CONFIRMATION_ENABLED.with(|c| *c.borrow());
        let close_confirmed = CLOSE_CONFIRMED.with(|c| *c.borrow());

        if confirmation_enabled && !close_confirmed {
            // Emit close requested event instead of closing
            PENDING_EVENTS.with(|events| {
                events.borrow_mut().push(InputEvent::WindowCloseRequested);
            });
            NO // Prevent close
        } else {
            // Reset confirmed flag for next time
            CLOSE_CONFIRMED.with(|c| *c.borrow_mut() = false);
            YES // Allow close
        }
    }

    unsafe {
        decl.add_method(
            sel!(windowShouldClose:),
            window_should_close as extern "C" fn(&Object, Sel, *mut Object) -> BOOL,
        );
    }

    // windowWillClose: - terminate app when window closes
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

    // windowDidBecomeKey: - window gained focus
    extern "C" fn window_did_become_key(_: &Object, _: Sel, _: *mut Object) {
        PENDING_EVENTS.with(|events| {
            events.borrow_mut().push(InputEvent::WindowFocused);
        });
    }

    unsafe {
        decl.add_method(
            sel!(windowDidBecomeKey:),
            window_did_become_key as extern "C" fn(&Object, Sel, *mut Object),
        );
    }

    // windowDidResignKey: - window lost focus
    extern "C" fn window_did_resign_key(_: &Object, _: Sel, _: *mut Object) {
        PENDING_EVENTS.with(|events| {
            events.borrow_mut().push(InputEvent::WindowBlurred);
        });
    }

    unsafe {
        decl.add_method(
            sel!(windowDidResignKey:),
            window_did_resign_key as extern "C" fn(&Object, Sel, *mut Object),
        );
    }

    // windowDidResize: - window was resized
    extern "C" fn window_did_resize(_: &Object, _: Sel, notification: *mut Object) {
        unsafe {
            let window: *mut Object = msg_send![notification, object];
            let content_rect: NSRect = msg_send![window, contentLayoutRect];
            PENDING_EVENTS.with(|events| {
                events.borrow_mut().push(InputEvent::WindowResized {
                    size: Vec2::new(content_rect.size.width as f32, content_rect.size.height as f32),
                });
            });
        }
    }

    unsafe {
        decl.add_method(
            sel!(windowDidResize:),
            window_did_resize as extern "C" fn(&Object, Sel, *mut Object),
        );
    }

    // windowDidMove: - window was moved
    extern "C" fn window_did_move(_: &Object, _: Sel, notification: *mut Object) {
        unsafe {
            let window: *mut Object = msg_send![notification, object];
            let frame: NSRect = msg_send![window, frame];
            PENDING_EVENTS.with(|events| {
                events.borrow_mut().push(InputEvent::WindowMoved {
                    position: Vec2::new(frame.origin.x as f32, frame.origin.y as f32),
                });
            });
        }
    }

    unsafe {
        decl.add_method(
            sel!(windowDidMove:),
            window_did_move as extern "C" fn(&Object, Sel, *mut Object),
        );
    }

    // windowDidMiniaturize: - window was minimized
    extern "C" fn window_did_miniaturize(_: &Object, _: Sel, _: *mut Object) {
        PENDING_EVENTS.with(|events| {
            events.borrow_mut().push(InputEvent::WindowMinimized);
        });
    }

    unsafe {
        decl.add_method(
            sel!(windowDidMiniaturize:),
            window_did_miniaturize as extern "C" fn(&Object, Sel, *mut Object),
        );
    }

    // windowDidDeminiaturize: - window was restored from minimized
    extern "C" fn window_did_deminiaturize(_: &Object, _: Sel, _: *mut Object) {
        PENDING_EVENTS.with(|events| {
            events.borrow_mut().push(InputEvent::WindowRestored);
        });
    }

    unsafe {
        decl.add_method(
            sel!(windowDidDeminiaturize:),
            window_did_deminiaturize as extern "C" fn(&Object, Sel, *mut Object),
        );
    }

    // windowDidEnterFullScreen: - entered fullscreen
    extern "C" fn window_did_enter_fullscreen(_: &Object, _: Sel, _: *mut Object) {
        PENDING_EVENTS.with(|events| {
            events.borrow_mut().push(InputEvent::WindowEnteredFullscreen);
        });
    }

    unsafe {
        decl.add_method(
            sel!(windowDidEnterFullScreen:),
            window_did_enter_fullscreen as extern "C" fn(&Object, Sel, *mut Object),
        );
    }

    // windowDidExitFullScreen: - exited fullscreen
    extern "C" fn window_did_exit_fullscreen(_: &Object, _: Sel, _: *mut Object) {
        PENDING_EVENTS.with(|events| {
            events.borrow_mut().push(InputEvent::WindowExitedFullscreen);
        });
    }

    unsafe {
        decl.add_method(
            sel!(windowDidExitFullScreen:),
            window_did_exit_fullscreen as extern "C" fn(&Object, Sel, *mut Object),
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
