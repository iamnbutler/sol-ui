use cocoa::base::{NO, YES, id, nil};
use cocoa::foundation::NSString;
use objc::{class, msg_send, sel, sel_impl};

// Helper function to create NSString
unsafe fn ns_string(string: &str) -> id {
    let str: id = NSString::alloc(nil).init_str(string);
    msg_send![str, autorelease]
}

pub fn create_app_menu() {
    let app_name = "Toy UI";

    // Create the menubar
    let menubar = unsafe {
        let menubar: id = msg_send![class!(NSMenu), new];
        menubar
    };

    // Create the app menu
    let app_menu_item = unsafe {
        let app_menu_item: id = msg_send![class!(NSMenuItem), new];
        app_menu_item
    };

    let app_menu = unsafe {
        let app_menu: id = msg_send![class!(NSMenu), new];
        app_menu
    };

    // Add Quit menu item
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

    // Add separator before quit
    let separator: id = unsafe { msg_send![class!(NSMenuItem), separatorItem] };

    // Add items to app menu
    unsafe {
        let _: () = msg_send![app_menu, addItem: separator];
        let _: () = msg_send![app_menu, addItem: quit_menu_item];
    }

    // Set the app menu
    unsafe {
        let _: () = msg_send![app_menu_item, setSubmenu: app_menu];
        let _: () = msg_send![menubar, addItem: app_menu_item];
    }

    // Create Edit menu with standard items
    let edit_menu_item = create_edit_menu();
    unsafe {
        let _: () = msg_send![menubar, addItem: edit_menu_item];
    }

    // Create Window menu
    let window_menu_item = create_window_menu();
    unsafe {
        let _: () = msg_send![menubar, addItem: window_menu_item];
    }

    // Set as main menu
    let app: id = unsafe { msg_send![class!(NSApplication), sharedApplication] };
    unsafe {
        let _: () = msg_send![app, setMainMenu: menubar];
    }
}

fn create_edit_menu() -> id {
    let edit_menu_item = unsafe {
        let edit_menu_item: id = msg_send![class!(NSMenuItem), new];
        edit_menu_item
    };

    let edit_menu = unsafe {
        let edit_menu: id = msg_send![class!(NSMenu), alloc];
        let title = ns_string("Edit");
        let edit_menu: id = msg_send![edit_menu, initWithTitle: title];
        edit_menu
    };

    // Add standard edit menu items
    add_menu_item(edit_menu, "Undo", sel!(undo:), "z");
    add_menu_item(edit_menu, "Redo", sel!(redo:), "Z");
    add_separator(edit_menu);
    add_menu_item(edit_menu, "Cut", sel!(cut:), "x");
    add_menu_item(edit_menu, "Copy", sel!(copy:), "c");
    add_menu_item(edit_menu, "Paste", sel!(paste:), "v");
    add_menu_item(edit_menu, "Select All", sel!(selectAll:), "a");

    unsafe {
        let _: () = msg_send![edit_menu_item, setSubmenu: edit_menu];
    }

    edit_menu_item
}

fn create_window_menu() -> id {
    let window_menu_item = unsafe {
        let window_menu_item: id = msg_send![class!(NSMenuItem), new];
        window_menu_item
    };

    let window_menu = unsafe {
        let window_menu: id = msg_send![class!(NSMenu), alloc];
        let title = ns_string("Window");
        let window_menu: id = msg_send![window_menu, initWithTitle: title];
        window_menu
    };

    // Add standard window menu items
    add_menu_item(window_menu, "Minimize", sel!(performMiniaturize:), "m");
    add_menu_item(window_menu, "Zoom", sel!(performZoom:), "");
    add_separator(window_menu);
    add_menu_item(window_menu, "Bring All to Front", sel!(arrangeInFront:), "");

    unsafe {
        let _: () = msg_send![window_menu_item, setSubmenu: window_menu];
    }

    window_menu_item
}

fn add_menu_item(menu: id, title: &str, action: objc::runtime::Sel, key_equivalent: &str) {
    let title = unsafe { ns_string(title) };
    let key = unsafe { ns_string(key_equivalent) };

    let menu_item: id = unsafe { msg_send![class!(NSMenuItem), alloc] };
    let menu_item: id = unsafe {
        msg_send![
            menu_item,
            initWithTitle:title
            action:action
            keyEquivalent:key
        ]
    };

    unsafe {
        let _: () = msg_send![menu, addItem: menu_item];
    }
}

fn add_separator(menu: id) {
    let separator: id = unsafe { msg_send![class!(NSMenuItem), separatorItem] };

    unsafe {
        let _: () = msg_send![menu, addItem: separator];
    }
}
