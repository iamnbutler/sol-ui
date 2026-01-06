//! Initially, we will only support macOS via Metal.
//!
//! In the future, this will be where platform-specific code lives.

#[cfg(target_os = "macos")]
pub mod mac;

#[cfg(target_os = "macos")]
pub use mac::{Window, create_app_menu, setup_app_delegate, is_app_active};
