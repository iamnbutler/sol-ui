use cocoa::base::{id, nil};
use cocoa::foundation::NSString;
use objc::{class, msg_send, sel, sel_impl};

// Helper function to create NSString
unsafe fn ns_string(string: &str) -> id {
    let str: id = unsafe { NSString::alloc(nil).init_str(string) };
    unsafe { msg_send![str, autorelease] }
}

pub fn create_app_menu() {
    let app_name = "Toy UI";

    let menubar = unsafe {
        let menubar: id = msg_send![class!(NSMenu), new];
        menubar
    };

    let app_menu_item = unsafe {
        let app_menu_item: id = msg_send![class!(NSMenuItem), new];
        app_menu_item
    };

    let app_menu = unsafe {
        let app_menu: id = msg_send![class!(NSMenu), new];
        app_menu
    };

    let quit_title = unsafe { ns_string(&format!("Quit {}", app_name)) };
    let quit_action = sel!(terminate:);
    let quit_key = unsafe { ns_string("q") };

    let quit_menu_item: id = unsafe { msg_send![class!(NSMenuItem), alloc] };
    let quit_menu_item: id = unsafe {
        msg_send![
            quit_menu_item,
            initWithTitle:quit_title
            action:quit_action
            keyEquivalent:quit_key
        ]
    };

    let separator: id = unsafe { msg_send![class!(NSMenuItem), separatorItem] };

    unsafe {
        let _: () = msg_send![app_menu, addItem: separator];
        let _: () = msg_send![app_menu, addItem: quit_menu_item];
    }

    unsafe {
        let _: () = msg_send![app_menu_item, setSubmenu: app_menu];
        let _: () = msg_send![menubar, addItem: app_menu_item];
    }

    let app: id = unsafe { msg_send![class!(NSApplication), sharedApplication] };
    unsafe {
        let _: () = msg_send![app, setMainMenu: menubar];
    }
}
