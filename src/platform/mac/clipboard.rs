//! macOS clipboard integration using NSPasteboard
//!
//! Provides copy/paste operations for sol-ui elements.
//!
//! # Usage
//! ```ignore
//! use sol_ui::platform::Clipboard;
//!
//! // Copy text to clipboard
//! Clipboard::copy("Hello, world!");
//!
//! // Paste from clipboard
//! if let Some(text) = Clipboard::paste() {
//!     println!("Pasted: {}", text);
//! }
//! ```
//!
//! The standard Edit menu (Cmd+C/V/X) is shown by `Menu::edit_menu()`.
//! Text input elements should call these methods when handling those shortcuts.

use cocoa::base::{id, nil};
use cocoa::foundation::NSString;
use objc::{class, msg_send, sel, sel_impl};

/// Clipboard access for copy/paste operations
pub struct Clipboard;

impl Clipboard {
    /// Copy text to the system clipboard
    pub fn copy(text: &str) -> bool {
        unsafe {
            let pasteboard: id = msg_send![class!(NSPasteboard), generalPasteboard];
            let _: i64 = msg_send![pasteboard, clearContents];

            let ns_string: id = NSString::alloc(nil).init_str(text);
            let ns_string_type: id = msg_send![class!(NSPasteboardType), string];

            let result: bool = msg_send![pasteboard, setString: ns_string forType: ns_string_type];
            result
        }
    }

    /// Paste text from the system clipboard
    pub fn paste() -> Option<String> {
        unsafe {
            let pasteboard: id = msg_send![class!(NSPasteboard), generalPasteboard];
            let ns_string_type: id = msg_send![class!(NSPasteboardType), string];

            let ns_string: id = msg_send![pasteboard, stringForType: ns_string_type];

            if ns_string == nil {
                return None;
            }

            let bytes: *const i8 = msg_send![ns_string, UTF8String];
            if bytes.is_null() {
                return None;
            }

            let c_str = std::ffi::CStr::from_ptr(bytes);
            c_str.to_str().ok().map(|s| s.to_string())
        }
    }

    /// Check if the clipboard contains text
    pub fn has_text() -> bool {
        unsafe {
            let pasteboard: id = msg_send![class!(NSPasteboard), generalPasteboard];
            let ns_string_type: id = msg_send![class!(NSPasteboardType), string];

            let types: id = msg_send![pasteboard, types];
            let contains: bool = msg_send![types, containsObject: ns_string_type];
            contains
        }
    }

    /// Clear the clipboard contents
    pub fn clear() {
        unsafe {
            let pasteboard: id = msg_send![class!(NSPasteboard), generalPasteboard];
            let _: i64 = msg_send![pasteboard, clearContents];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_roundtrip() {
        let test_text = "sol-ui clipboard test";
        assert!(Clipboard::copy(test_text));
        assert!(Clipboard::has_text());
        assert_eq!(Clipboard::paste(), Some(test_text.to_string()));
    }
}
