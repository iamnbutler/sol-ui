use cocoa::{
    base::{id, nil, NO, YES},
    foundation::NSString,
};
use objc::{
    class,
    declare::ClassDecl,
    msg_send,
    runtime::{Class, Object, Sel, BOOL},
    sel, sel_impl,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    ffi::c_void,
    ptr,
    sync::atomic::{AtomicU64, Ordering},
};

// Helper function to create NSString
unsafe fn ns_string(string: &str) -> id {
    let str: id = unsafe { NSString::alloc(nil).init_str(string) };
    unsafe { msg_send![str, autorelease] }
}

/// Keyboard modifier flags for menu shortcuts
#[derive(Debug, Clone, Copy, Default)]
pub struct KeyModifiers {
    pub cmd: bool,
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
}

impl KeyModifiers {
    pub fn cmd() -> Self {
        Self {
            cmd: true,
            ..Default::default()
        }
    }

    pub fn cmd_shift() -> Self {
        Self {
            cmd: true,
            shift: true,
            ..Default::default()
        }
    }

    pub fn cmd_alt() -> Self {
        Self {
            cmd: true,
            alt: true,
            ..Default::default()
        }
    }

    fn to_ns_modifier_mask(&self) -> u64 {
        let mut mask: u64 = 0;
        if self.cmd {
            mask |= 1 << 20; // NSEventModifierFlagCommand
        }
        if self.shift {
            mask |= 1 << 17; // NSEventModifierFlagShift
        }
        if self.alt {
            mask |= 1 << 19; // NSEventModifierFlagOption
        }
        if self.ctrl {
            mask |= 1 << 18; // NSEventModifierFlagControl
        }
        mask
    }
}

/// A keyboard shortcut for a menu item
#[derive(Debug, Clone)]
pub struct KeyboardShortcut {
    pub key: String,
    pub modifiers: KeyModifiers,
}

impl KeyboardShortcut {
    /// Create a shortcut with Cmd modifier (e.g., Cmd+N)
    pub fn cmd(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            modifiers: KeyModifiers::cmd(),
        }
    }

    /// Create a shortcut with Cmd+Shift modifiers (e.g., Cmd+Shift+S)
    pub fn cmd_shift(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            modifiers: KeyModifiers::cmd_shift(),
        }
    }

    /// Create a shortcut with Cmd+Alt modifiers
    pub fn cmd_alt(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            modifiers: KeyModifiers::cmd_alt(),
        }
    }

    /// Create a shortcut with custom modifiers
    pub fn with_modifiers(key: impl Into<String>, modifiers: KeyModifiers) -> Self {
        Self {
            key: key.into(),
            modifiers,
        }
    }
}

/// Callback type for menu item actions
pub type MenuAction = Box<dyn Fn() + Send + Sync + 'static>;

/// Unique ID for menu items
static NEXT_MENU_ITEM_ID: AtomicU64 = AtomicU64::new(1);

fn next_menu_item_id() -> u64 {
    NEXT_MENU_ITEM_ID.fetch_add(1, Ordering::SeqCst)
}

// Global registry for menu item callbacks
thread_local! {
    static MENU_ACTIONS: RefCell<HashMap<u64, MenuAction>> = RefCell::new(HashMap::new());
    static MENU_TARGET_CLASS: RefCell<Option<&'static Class>> = RefCell::new(None);
}

/// A menu item that can be added to a menu
pub enum MenuItem {
    /// A clickable item with a title and optional action
    Action {
        id: u64,
        title: String,
        shortcut: Option<KeyboardShortcut>,
        enabled: bool,
        checked: bool,
    },
    /// A visual separator line
    Separator,
    /// A submenu containing more items
    Submenu { title: String, menu: Menu },
}

impl MenuItem {
    /// Create a new action menu item
    pub fn action(title: impl Into<String>) -> MenuItemBuilder {
        MenuItemBuilder {
            id: next_menu_item_id(),
            title: title.into(),
            shortcut: None,
            enabled: true,
            checked: false,
            action: None,
        }
    }

    /// Create a separator
    pub fn separator() -> Self {
        MenuItem::Separator
    }

    /// Create a submenu
    pub fn submenu(title: impl Into<String>, menu: Menu) -> Self {
        MenuItem::Submenu {
            title: title.into(),
            menu,
        }
    }
}

/// Builder for constructing menu items
pub struct MenuItemBuilder {
    id: u64,
    title: String,
    shortcut: Option<KeyboardShortcut>,
    enabled: bool,
    checked: bool,
    action: Option<MenuAction>,
}

impl MenuItemBuilder {
    /// Set the keyboard shortcut (e.g., Cmd+N)
    pub fn shortcut(mut self, shortcut: KeyboardShortcut) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    /// Set whether the item is enabled
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set whether the item shows a checkmark
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Set the action callback when the item is clicked
    pub fn on_action<F>(mut self, action: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.action = Some(Box::new(action));
        self
    }

    /// Build the menu item
    pub fn build(self) -> MenuItem {
        // Register the action callback if present
        if let Some(action) = self.action {
            MENU_ACTIONS.with(|actions| {
                actions.borrow_mut().insert(self.id, action);
            });
        }

        MenuItem::Action {
            id: self.id,
            title: self.title,
            shortcut: self.shortcut,
            enabled: self.enabled,
            checked: self.checked,
        }
    }
}

/// A menu containing multiple items
pub struct Menu {
    title: String,
    items: Vec<MenuItem>,
}

impl Menu {
    /// Create a new menu with the given title
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            items: Vec::new(),
        }
    }

    /// Add an item to the menu
    pub fn item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    /// Add multiple items to the menu
    pub fn items(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.items.extend(items);
        self
    }

    /// Add a separator to the menu
    pub fn separator(mut self) -> Self {
        self.items.push(MenuItem::Separator);
        self
    }

    /// Create the standard App menu (with app name, About, Quit, etc.)
    pub fn app_menu(app_name: impl Into<String>) -> Self {
        let app_name = app_name.into();
        Menu::new("")
            .item(
                MenuItem::action(format!("About {}", app_name))
                    .on_action(|| {
                        // Default about action - shows order front standard about panel
                        unsafe {
                            let app: id = msg_send![class!(NSApplication), sharedApplication];
                            let _: () = msg_send![app, orderFrontStandardAboutPanel: nil];
                        }
                    })
                    .build(),
            )
            .separator()
            .item(
                MenuItem::action(format!("Hide {}", app_name))
                    .shortcut(KeyboardShortcut::cmd("h"))
                    .on_action(|| unsafe {
                        let app: id = msg_send![class!(NSApplication), sharedApplication];
                        let _: () = msg_send![app, hide: nil];
                    })
                    .build(),
            )
            .item(
                MenuItem::action("Hide Others")
                    .shortcut(KeyboardShortcut::cmd_alt("h"))
                    .on_action(|| unsafe {
                        let app: id = msg_send![class!(NSApplication), sharedApplication];
                        let _: () = msg_send![app, hideOtherApplications: nil];
                    })
                    .build(),
            )
            .item(
                MenuItem::action("Show All")
                    .on_action(|| unsafe {
                        let app: id = msg_send![class!(NSApplication), sharedApplication];
                        let _: () = msg_send![app, unhideAllApplications: nil];
                    })
                    .build(),
            )
            .separator()
            .item(
                MenuItem::action(format!("Quit {}", app_name))
                    .shortcut(KeyboardShortcut::cmd("q"))
                    .on_action(|| unsafe {
                        let app: id = msg_send![class!(NSApplication), sharedApplication];
                        let _: () = msg_send![app, terminate: nil];
                    })
                    .build(),
            )
    }

    /// Create a standard Edit menu with undo, cut, copy, paste, etc.
    pub fn edit_menu() -> Self {
        Menu::new("Edit")
            .item(
                MenuItem::action("Undo")
                    .shortcut(KeyboardShortcut::cmd("z"))
                    .build(),
            )
            .item(
                MenuItem::action("Redo")
                    .shortcut(KeyboardShortcut::cmd_shift("z"))
                    .build(),
            )
            .separator()
            .item(
                MenuItem::action("Cut")
                    .shortcut(KeyboardShortcut::cmd("x"))
                    .build(),
            )
            .item(
                MenuItem::action("Copy")
                    .shortcut(KeyboardShortcut::cmd("c"))
                    .build(),
            )
            .item(
                MenuItem::action("Paste")
                    .shortcut(KeyboardShortcut::cmd("v"))
                    .build(),
            )
            .item(
                MenuItem::action("Select All")
                    .shortcut(KeyboardShortcut::cmd("a"))
                    .build(),
            )
    }

    /// Create a standard Window menu
    pub fn window_menu() -> Self {
        Menu::new("Window")
            .item(
                MenuItem::action("Minimize")
                    .shortcut(KeyboardShortcut::cmd("m"))
                    .build(),
            )
            .item(
                MenuItem::action("Zoom")
                    .build(),
            )
            .separator()
            .item(
                MenuItem::action("Bring All to Front")
                    .build(),
            )
    }

    /// Create a standard Help menu
    pub fn help_menu(app_name: impl Into<String>) -> Self {
        let app_name = app_name.into();
        Menu::new("Help").item(MenuItem::action(format!("{} Help", app_name)).build())
    }
}

/// Builder for the main menu bar
pub struct MenuBar {
    app_name: String,
    menus: Vec<Menu>,
}

impl MenuBar {
    /// Create a new menu bar with the given app name
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
            menus: Vec::new(),
        }
    }

    /// Add a menu to the menu bar
    pub fn menu(mut self, menu: Menu) -> Self {
        self.menus.push(menu);
        self
    }

    /// Add the standard app menu
    pub fn with_app_menu(self) -> Self {
        let app_name = self.app_name.clone();
        self.menu(Menu::app_menu(app_name))
    }

    /// Add a standard Edit menu
    pub fn with_edit_menu(self) -> Self {
        self.menu(Menu::edit_menu())
    }

    /// Add a standard Window menu
    pub fn with_window_menu(self) -> Self {
        self.menu(Menu::window_menu())
    }

    /// Add a standard Help menu
    pub fn with_help_menu(self) -> Self {
        let app_name = self.app_name.clone();
        self.menu(Menu::help_menu(app_name))
    }

    /// Build and set the menu bar
    pub fn build(self) {
        unsafe {
            ensure_menu_target_class();
        }

        let menubar: id = unsafe { msg_send![class!(NSMenu), new] };

        for menu in self.menus {
            let ns_menu = create_ns_menu(&menu);
            let menu_item: id = unsafe { msg_send![class!(NSMenuItem), new] };
            let _: () = unsafe { msg_send![menu_item, setSubmenu: ns_menu] };
            let _: () = unsafe { msg_send![menubar, addItem: menu_item] };
        }

        let app: id = unsafe { msg_send![class!(NSApplication), sharedApplication] };
        let _: () = unsafe { msg_send![app, setMainMenu: menubar] };
    }
}

/// Create an NSMenu from a Menu
fn create_ns_menu(menu: &Menu) -> id {
    let ns_menu: id = unsafe {
        let menu_obj: id = msg_send![class!(NSMenu), alloc];
        let title = ns_string(&menu.title);
        msg_send![menu_obj, initWithTitle: title]
    };

    for item in &menu.items {
        let ns_item = create_ns_menu_item(item);
        let _: () = unsafe { msg_send![ns_menu, addItem: ns_item] };
    }

    ns_menu
}

/// Create an NSMenuItem from a MenuItem
fn create_ns_menu_item(item: &MenuItem) -> id {
    match item {
        MenuItem::Separator => unsafe { msg_send![class!(NSMenuItem), separatorItem] },
        MenuItem::Action {
            id,
            title,
            shortcut,
            enabled,
            checked,
        } => {
            let title = unsafe { ns_string(title) };
            let key = shortcut
                .as_ref()
                .map(|s| s.key.as_str())
                .unwrap_or("");
            let key_equiv = unsafe { ns_string(key) };

            let ns_item: id = unsafe {
                let item: id = msg_send![class!(NSMenuItem), alloc];
                msg_send![
                    item,
                    initWithTitle: title
                    action: sel!(menuItemAction:)
                    keyEquivalent: key_equiv
                ]
            };

            // Set key modifier mask if there's a shortcut
            if let Some(shortcut) = shortcut {
                let mask = shortcut.modifiers.to_ns_modifier_mask();
                let _: () = unsafe { msg_send![ns_item, setKeyEquivalentModifierMask: mask] };
            }

            // Set enabled state
            let _: () = unsafe { msg_send![ns_item, setEnabled: if *enabled { YES } else { NO }] };

            // Set checked state
            if *checked {
                let _: () = unsafe { msg_send![ns_item, setState: 1i64] }; // NSControlStateValueOn
            }

            // Store the action ID as the tag
            let _: () = unsafe { msg_send![ns_item, setTag: *id as i64] };

            // Set target to our menu target class
            MENU_TARGET_CLASS.with(|class_cell| {
                if let Some(cls) = *class_cell.borrow() {
                    let target: id = unsafe { msg_send![cls, sharedTarget] };
                    let _: () = unsafe { msg_send![ns_item, setTarget: target] };
                }
            });

            ns_item
        }
        MenuItem::Submenu { title, menu } => {
            let title = unsafe { ns_string(title) };
            let ns_item: id = unsafe {
                let item: id = msg_send![class!(NSMenuItem), alloc];
                msg_send![
                    item,
                    initWithTitle: title
                    action: nil
                    keyEquivalent: ns_string("")
                ]
            };

            let ns_submenu = create_ns_menu(menu);
            let _: () = unsafe { msg_send![ns_item, setSubmenu: ns_submenu] };

            ns_item
        }
    }
}

/// Ensure the menu target class is created
unsafe fn ensure_menu_target_class() {
    MENU_TARGET_CLASS.with(|class_cell| {
        if class_cell.borrow().is_none() {
            let cls = unsafe { create_menu_target_class() };
            *class_cell.borrow_mut() = Some(cls);
        }
    });
}

/// The menu target class singleton instance
static mut MENU_TARGET_INSTANCE: *mut Object = ptr::null_mut();

/// Create the Objective-C class that handles menu actions
unsafe fn create_menu_target_class() -> &'static Class {
    let superclass = class!(NSObject);
    let mut decl = ClassDecl::new("AceMenuTarget", superclass).unwrap();

    // Add instance variable to track singleton
    decl.add_ivar::<*mut c_void>("_dummy");

    // Class method to get shared instance
    extern "C" fn shared_target(_: &Class, _: Sel) -> id {
        unsafe {
            if MENU_TARGET_INSTANCE.is_null() {
                let cls = class!(AceMenuTarget);
                MENU_TARGET_INSTANCE = msg_send![cls, new];
            }
            MENU_TARGET_INSTANCE
        }
    }

    // Instance method to handle menu item clicks
    extern "C" fn menu_item_action(_: &Object, _: Sel, sender: id) {
        let tag: i64 = unsafe { msg_send![sender, tag] };
        let id = tag as u64;

        MENU_ACTIONS.with(|actions| {
            if let Some(action) = actions.borrow().get(&id) {
                action();
            }
        });
    }

    // Validation method for menu items
    extern "C" fn validate_menu_item(_: &Object, _: Sel, _menu_item: id) -> BOOL {
        // By default, enable all items
        // More sophisticated validation could check the action registry
        YES
    }

    unsafe {
        decl.add_class_method(
            sel!(sharedTarget),
            shared_target as extern "C" fn(&Class, Sel) -> id,
        );

        decl.add_method(
            sel!(menuItemAction:),
            menu_item_action as extern "C" fn(&Object, Sel, id),
        );

        decl.add_method(
            sel!(validateMenuItem:),
            validate_menu_item as extern "C" fn(&Object, Sel, id) -> BOOL,
        );
    }

    decl.register()
}

/// Legacy function for backwards compatibility
/// Creates a basic app menu with just Quit
pub fn create_app_menu() {
    MenuBar::new("Toy UI")
        .with_app_menu()
        .build();
}

/// Set up a full menu bar with standard menus
pub fn create_standard_menu_bar(app_name: impl Into<String>) {
    let app_name = app_name.into();
    MenuBar::new(&app_name)
        .with_app_menu()
        .with_edit_menu()
        .with_window_menu()
        .with_help_menu()
        .build();
}

// Re-exports for convenience
pub use KeyModifiers as MenuModifiers;
