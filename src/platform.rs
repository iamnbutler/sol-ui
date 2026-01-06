//! Initially, we will only support macOS via Metal.
//!
//! In the future, this will be where platform-specific code lives.

#[cfg(target_os = "macos")]
pub mod mac;

#[cfg(target_os = "macos")]
pub use mac::{
    create_app_menu, create_standard_menu_bar, Clipboard, KeyModifiers, KeyboardShortcut, Menu,
    MenuBar, MenuItem, MenuItemBuilder, MenuModifiers, Window,
};
