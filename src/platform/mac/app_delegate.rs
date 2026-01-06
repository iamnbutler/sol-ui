//! NSApplicationDelegate implementation for macOS app lifecycle hooks
//!
//! Provides callbacks for:
//! - App launch completion
//! - App termination
//! - App activation/deactivation
//! - Window close behavior

use objc::{
    class,
    declare::ClassDecl,
    msg_send,
    runtime::{Class, Object, Sel, BOOL, YES},
    sel, sel_impl,
};
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

/// Global flag indicating the delegate class has been created
static mut APP_DELEGATE_CLASS: *const Class = null_mut();

/// Global flag for app active state
static APP_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Global pointer to the delegate instance for callbacks
static APP_DELEGATE_INSTANCE: AtomicPtr<Object> = AtomicPtr::new(null_mut());

/// Initialize the app delegate and set it on NSApplication
///
/// # Safety
/// Must be called after NSApplication is initialized
pub unsafe fn setup_app_delegate() {
    ensure_class_registered();

    unsafe {
        let delegate_class = APP_DELEGATE_CLASS;
        let delegate: *mut Object = msg_send![delegate_class, new];
        APP_DELEGATE_INSTANCE.store(delegate, Ordering::SeqCst);

        let app: *mut Object = msg_send![class!(NSApplication), sharedApplication];
        let _: () = msg_send![app, setDelegate: delegate];
    }
}

/// Check if the app is currently active (in foreground)
pub fn is_app_active() -> bool {
    APP_ACTIVE.load(Ordering::SeqCst)
}

/// Ensure the delegate class is registered
fn ensure_class_registered() {
    unsafe {
        if APP_DELEGATE_CLASS.is_null() {
            create_app_delegate_class();
        }
    }
}

/// Create the ToyUIAppDelegate Objective-C class
unsafe fn create_app_delegate_class() {
    // applicationDidFinishLaunching:
    extern "C" fn application_did_finish_launching(_: &Object, _: Sel, _: *mut Object) {
        tracing::info!("Application did finish launching");
        APP_ACTIVE.store(true, Ordering::SeqCst);
    }

    // applicationWillTerminate:
    extern "C" fn application_will_terminate(_: &Object, _: Sel, _: *mut Object) {
        tracing::info!("Application will terminate");
    }

    // applicationDidBecomeActive:
    extern "C" fn application_did_become_active(_: &Object, _: Sel, _: *mut Object) {
        tracing::debug!("Application did become active");
        APP_ACTIVE.store(true, Ordering::SeqCst);
    }

    // applicationWillResignActive:
    extern "C" fn application_will_resign_active(_: &Object, _: Sel, _: *mut Object) {
        tracing::debug!("Application will resign active");
        APP_ACTIVE.store(false, Ordering::SeqCst);
    }

    // applicationShouldTerminateAfterLastWindowClosed:
    extern "C" fn application_should_terminate_after_last_window_closed(
        _: &Object,
        _: Sel,
        _: *mut Object,
    ) -> BOOL {
        // Return YES to quit when the last window is closed
        YES
    }

    // applicationShouldTerminate:
    extern "C" fn application_should_terminate(
        _: &Object,
        _: Sel,
        _: *mut Object,
    ) -> u64 {
        // NSTerminateNow = 1
        tracing::info!("Application should terminate - allowing");
        1
    }

    // applicationWillFinishLaunching:
    extern "C" fn application_will_finish_launching(_: &Object, _: Sel, _: *mut Object) {
        tracing::debug!("Application will finish launching");
    }

    // applicationDidHide:
    extern "C" fn application_did_hide(_: &Object, _: Sel, _: *mut Object) {
        tracing::debug!("Application did hide");
    }

    // applicationDidUnhide:
    extern "C" fn application_did_unhide(_: &Object, _: Sel, _: *mut Object) {
        tracing::debug!("Application did unhide");
    }

    unsafe {
        let superclass = class!(NSObject);
        let mut decl = ClassDecl::new("ToyUIAppDelegate", superclass)
            .expect("Failed to create ToyUIAppDelegate class");

        // Register all delegate methods
        decl.add_method(
            sel!(applicationDidFinishLaunching:),
            application_did_finish_launching as extern "C" fn(&Object, Sel, *mut Object),
        );

        decl.add_method(
            sel!(applicationWillTerminate:),
            application_will_terminate as extern "C" fn(&Object, Sel, *mut Object),
        );

        decl.add_method(
            sel!(applicationDidBecomeActive:),
            application_did_become_active as extern "C" fn(&Object, Sel, *mut Object),
        );

        decl.add_method(
            sel!(applicationWillResignActive:),
            application_will_resign_active as extern "C" fn(&Object, Sel, *mut Object),
        );

        decl.add_method(
            sel!(applicationShouldTerminateAfterLastWindowClosed:),
            application_should_terminate_after_last_window_closed
                as extern "C" fn(&Object, Sel, *mut Object) -> BOOL,
        );

        decl.add_method(
            sel!(applicationShouldTerminate:),
            application_should_terminate as extern "C" fn(&Object, Sel, *mut Object) -> u64,
        );

        decl.add_method(
            sel!(applicationWillFinishLaunching:),
            application_will_finish_launching as extern "C" fn(&Object, Sel, *mut Object),
        );

        decl.add_method(
            sel!(applicationDidHide:),
            application_did_hide as extern "C" fn(&Object, Sel, *mut Object),
        );

        decl.add_method(
            sel!(applicationDidUnhide:),
            application_did_unhide as extern "C" fn(&Object, Sel, *mut Object),
        );

        APP_DELEGATE_CLASS = decl.register();
    }
}
